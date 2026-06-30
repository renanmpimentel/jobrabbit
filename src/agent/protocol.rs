//! NDJSON protocol between agent (`claude`) and jobRabbit.
//!
//! The prompts (see [`super::prompts`]) instruct the agent to emit, in the response text,
//! one JSON line per item with a discriminator `type` field:
//!
//! - `{"type":"job","title","company","url","source","description","fit_score"}`
//! - `{"type":"application","url","status","cv","cover"}`  (status: applied|skipped|failed)
//! - `{"type":"pending","url","kind","description"}`       (kind: captcha|required_field|login)
//!
//! Lines that don't match the protocol are treated as normal human text.

use crate::db::models::NewJob;

#[derive(Debug, Clone, PartialEq)]
pub enum AgentOutput {
    Job(NewJob),
    Application {
        url: String,
        status: String,
        cv: Option<String>,
        cover: Option<String>,
        screenshot: Option<String>,
    },
    Pending {
        url: Option<String>,
        kind: String,
        description: String,
        /// For `kind="answer_needed"`: the answer key that the user should fill.
        field_key: Option<String>,
    },
    Answer {
        key: String,
        label: String,
        value: String,
    },
    Feedback {
        summary: String,
        suggestions: String,
    },
    CvReview {
        score: u8,
        target: String,
        report: String,
    },
    /// Improved (rewritten/optimized for ATS) version of the resume.
    CvImproved {
        content: String,
        target: String,
    },
    Profile {
        background: String,
        cv_base: String,
        /// Suggested search variants: `(label, query)`.
        variants: Vec<(String, String)>,
    },
}

fn str_field(v: &serde_json::Value, key: &str) -> String {
    v.get(key)
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string()
}

fn opt_str(v: &serde_json::Value, key: &str) -> Option<String> {
    v.get(key)
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

/// Tries to interpret a line as a protocol item. `None` if not.
pub fn parse(line: &str) -> Option<AgentOutput> {
    let trimmed = line.trim();
    if !trimmed.starts_with('{') {
        return None;
    }
    let v: serde_json::Value = serde_json::from_str(trimmed).ok()?;
    match v.get("type")?.as_str()? {
        "job" => {
            let url = str_field(&v, "url");
            let title = str_field(&v, "title");
            if url.is_empty() || title.is_empty() {
                return None;
            }
            Some(AgentOutput::Job(NewJob {
                title,
                company: str_field(&v, "company"),
                url,
                source: str_field(&v, "source"),
                description: str_field(&v, "description"),
                fit_score: v.get("fit_score").and_then(|x| x.as_f64()),
            }))
        }
        "application" => {
            let url = str_field(&v, "url");
            if url.is_empty() {
                return None;
            }
            Some(AgentOutput::Application {
                url,
                status: {
                    let s = str_field(&v, "status");
                    if s.is_empty() {
                        "applied".to_string()
                    } else {
                        s
                    }
                },
                cv: opt_str(&v, "cv"),
                cover: opt_str(&v, "cover"),
                screenshot: opt_str(&v, "screenshot"),
            })
        }
        "pending" => {
            let kind = str_field(&v, "kind");
            if kind.is_empty() {
                return None;
            }
            Some(AgentOutput::Pending {
                url: opt_str(&v, "url"),
                kind,
                description: str_field(&v, "description"),
                field_key: opt_str(&v, "field_key"),
            })
        }
        "answer" => {
            let key = str_field(&v, "key");
            if key.is_empty() {
                return None;
            }
            Some(AgentOutput::Answer {
                key,
                label: str_field(&v, "label"),
                value: str_field(&v, "value"),
            })
        }
        "feedback" => {
            let summary = str_field(&v, "summary");
            if summary.is_empty() {
                return None;
            }
            Some(AgentOutput::Feedback {
                summary,
                suggestions: str_field(&v, "suggestions"),
            })
        }
        "cv_review" => {
            let report = str_field(&v, "report");
            if report.is_empty() {
                return None;
            }
            let score = v
                .get("score")
                .and_then(|x| x.as_i64())
                .unwrap_or(0)
                .clamp(0, 100) as u8;
            Some(AgentOutput::CvReview {
                score,
                target: str_field(&v, "target"),
                report,
            })
        }
        "cv_improved" => {
            let content = str_field(&v, "content");
            if content.is_empty() {
                return None;
            }
            Some(AgentOutput::CvImproved {
                content,
                target: str_field(&v, "target"),
            })
        }
        "profile" => {
            let background = str_field(&v, "background");
            let cv_base = str_field(&v, "cv_base");
            if background.is_empty() && cv_base.is_empty() {
                return None;
            }
            let variants = v
                .get("variants")
                .and_then(|x| x.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|it| {
                            let label = str_field(it, "label");
                            let query = str_field(it, "query");
                            if label.is_empty() || query.is_empty() {
                                None
                            } else {
                                Some((label, query))
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();
            Some(AgentOutput::Profile {
                background,
                cv_base,
                variants,
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_job() {
        let l = r#"{"type":"job","title":"Dev","company":"Acme","url":"https://x/1","source":"linkedin","description":"d","fit_score":0.8}"#;
        match parse(l).unwrap() {
            AgentOutput::Job(j) => {
                assert_eq!(j.title, "Dev");
                assert_eq!(j.url, "https://x/1");
                assert_eq!(j.fit_score, Some(0.8));
            }
            other => panic!("expected Job, got {other:?}"),
        }
    }

    #[test]
    fn job_without_url_or_title_is_invalid() {
        assert!(parse(r#"{"type":"job","title":"Dev"}"#).is_none());
        assert!(parse(r#"{"type":"job","url":"https://x"}"#).is_none());
    }

    #[test]
    fn parses_application_with_default_status() {
        let l = r#"{"type":"application","url":"https://x/1","cv":"my cv"}"#;
        match parse(l).unwrap() {
            AgentOutput::Application {
                url, status, cv, screenshot, ..
            } => {
                assert_eq!(url, "https://x/1");
                assert_eq!(status, "applied");
                assert_eq!(cv.as_deref(), Some("my cv"));
                assert!(screenshot.is_none());
            }
            other => panic!("expected Application, got {other:?}"),
        }
    }

    #[test]
    fn parses_application_with_screenshot() {
        let l = r#"{"type":"application","url":"https://x/1","status":"applied","screenshot":"/tmp/screenshot.png"}"#;
        match parse(l).unwrap() {
            AgentOutput::Application {
                url, status, screenshot, ..
            } => {
                assert_eq!(url, "https://x/1");
                assert_eq!(status, "applied");
                assert_eq!(screenshot.as_deref(), Some("/tmp/screenshot.png"));
            }
            other => panic!("expected Application, got {other:?}"),
        }
    }

    #[test]
    fn parses_pending() {
        let l =
            r#"{"type":"pending","url":"https://x/1","kind":"captcha","description":"resolve"}"#;
        match parse(l).unwrap() {
            AgentOutput::Pending {
                url,
                kind,
                description,
                field_key,
            } => {
                assert_eq!(url.as_deref(), Some("https://x/1"));
                assert_eq!(kind, "captcha");
                assert_eq!(description, "resolve");
                assert!(field_key.is_none());
            }
            other => panic!("expected Pending, got {other:?}"),
        }
    }

    #[test]
    fn parses_answer_and_pending_with_field_key() {
        let a = r#"{"type":"answer","key":"english_level","label":"English","value":"advanced"}"#;
        match parse(a).unwrap() {
            AgentOutput::Answer { key, value, .. } => {
                assert_eq!(key, "english_level");
                assert_eq!(value, "advanced");
            }
            other => panic!("expected Answer, got {other:?}"),
        }
        assert!(parse(r#"{"type":"answer","value":"x"}"#).is_none());

        let p = r#"{"type":"pending","kind":"answer_needed","description":"Salary expectation?","field_key":"salary_expectation"}"#;
        match parse(p).unwrap() {
            AgentOutput::Pending {
                kind, field_key, ..
            } => {
                assert_eq!(kind, "answer_needed");
                assert_eq!(field_key.as_deref(), Some("salary_expectation"));
            }
            other => panic!("expected Pending, got {other:?}"),
        }
    }

    #[test]
    fn parses_feedback() {
        let l = r#"{"type":"feedback","summary":"good fit","suggestions":"- broaden search"}"#;
        match parse(l).unwrap() {
            AgentOutput::Feedback {
                summary,
                suggestions,
            } => {
                assert_eq!(summary, "good fit");
                assert!(suggestions.contains("broaden"));
            }
            other => panic!("expected Feedback, got {other:?}"),
        }
        assert!(parse(r#"{"type":"feedback","suggestions":"x"}"#).is_none());
    }

    #[test]
    fn parses_profile() {
        let l = r#"{"type":"profile","background":"Backend dev 8 years","cv_base":"CV...","variants":[{"label":"Senior Remote","query":"senior rust remote"},{"label":"X","query":""}]}"#;
        match parse(l).unwrap() {
            AgentOutput::Profile {
                background,
                cv_base,
                variants,
            } => {
                assert_eq!(background, "Backend dev 8 years");
                assert_eq!(cv_base, "CV...");
                // the 2nd variant (empty query) is discarded
                assert_eq!(
                    variants,
                    vec![("Senior Remote".into(), "senior rust remote".into())]
                );
            }
            other => panic!("expected Profile, got {other:?}"),
        }
        // without background and without cv_base → invalid
        assert!(parse(r#"{"type":"profile","variants":[]}"#).is_none());
    }

    #[test]
    fn parses_cv_review() {
        let l = r#"{"type":"cv_review","score":82,"target":"Eng Manager","report":"Strengths - accomplishments"}"#;
        match parse(l).unwrap() {
            AgentOutput::CvReview {
                score,
                target,
                report,
            } => {
                assert_eq!(score, 82);
                assert_eq!(target, "Eng Manager");
                assert!(report.contains("accomplishments"));
            }
            other => panic!("expected CvReview, got {other:?}"),
        }
        // no report → invalid; score outside range is clamped
        assert!(parse(r#"{"type":"cv_review","score":50}"#).is_none());
        if let AgentOutput::CvReview { score, .. } =
            parse(r#"{"type":"cv_review","score":250,"report":"x"}"#).unwrap()
        {
            assert_eq!(score, 100);
        }
    }

    #[test]
    fn parses_cv_improved() {
        let l = r##"{"type":"cv_improved","target":"general","content":"# John\n- did X"}"##;
        match parse(l).unwrap() {
            AgentOutput::CvImproved { content, target } => {
                assert_eq!(target, "general");
                assert!(content.contains("John"));
            }
            other => panic!("expected CvImproved, got {other:?}"),
        }
        // no content → invalid
        assert!(parse(r#"{"type":"cv_improved","target":"x"}"#).is_none());
    }

    #[test]
    fn human_text_is_not_protocol() {
        assert!(parse("Analyzing jobs...").is_none());
        assert!(parse(r#"{"type":"something"}"#).is_none());
        assert!(parse("").is_none());
    }
}
