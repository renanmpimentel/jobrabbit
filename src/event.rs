//! Application event bus.
//!
//! The core (agent, idle, etc.) emits [`AppEvent`]; the TUI consumes. Keeps the UI
//! decoupled from the `claude` stream format (see [`crate::agent::stream`]).

use crate::agent::stream::StreamEvent;
use crate::db::models::NewJob;
use serde::{Deserialize, Serialize};

/// Events that circulate between core and TUI.
///
/// Serializes to JSON (externally tagged representation, e.g.,
/// `{"AgentText":"..."}`) to be transmitted by the web backend's SSE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppEvent {
    /// Periodic render/update tick.
    Tick,

    /// An agent execution started (the UI creates the session in the DB).
    AgentStarted,
    /// `claude`'s `session_id` (captured in the init event).
    AgentSessionId(String),
    /// Text produced by the assistant (for the Session log).
    AgentText(String),
    /// The agent called a tool (e.g. Chrome navigation).
    AgentToolUse {
        name: String,
        input: serde_json::Value,
    },
    /// The agent reported a job (NDJSON protocol).
    AgentJobFound(NewJob),
    /// The agent reported an application.
    AgentApplication {
        url: String,
        status: String,
        cv: Option<String>,
        cover: Option<String>,
        screenshot: Option<String>,
    },
    /// The agent reported a pending action (captcha/field/login/answer_needed).
    AgentPending {
        url: Option<String>,
        kind: String,
        description: String,
        field_key: Option<String>,
    },
    /// The agent inferred/learned a screening answer.
    AgentAnswer {
        key: String,
        label: String,
        value: String,
    },
    /// The agent generated a feedback analysis.
    AgentFeedback {
        summary: String,
        suggestions: String,
    },
    /// The agent evaluated the resume (ATS tab).
    AgentCvReview {
        score: u8,
        target: String,
        report: String,
        keywords: Vec<crate::db::models::Keyword>,
    },
    /// The agent generated an improved version of the resume (ATS tab).
    AgentCvImproved { content: String, target: String },
    /// The agent built a profile (import from CV/LinkedIn).
    AgentProfile {
        background: String,
        cv_base: String,
        variants: Vec<(String, String)>,
    },
    /// Raw line from stream-json (debug/raw log).
    AgentRaw(String),
    /// Execution finished.
    AgentFinished {
        result: Option<String>,
        num_turns: Option<u32>,
        cost_usd: Option<f64>,
        is_error: bool,
    },
    /// Error in the agent (spawn, IO, etc.).
    AgentError(String),

    /// User reached idle threshold (auto-execution trigger).
    IdleReached,

    /// Shut down the application.
    Quit,
}

impl AppEvent {
    /// Converts an item from the agent protocol to the corresponding event.
    pub fn from_agent_output(out: crate::agent::protocol::AgentOutput) -> Self {
        use crate::agent::protocol::AgentOutput;
        match out {
            AgentOutput::Job(j) => AppEvent::AgentJobFound(j),
            AgentOutput::Application {
                url,
                status,
                cv,
                cover,
                screenshot,
            } => AppEvent::AgentApplication {
                url,
                status,
                cv,
                cover,
                screenshot,
            },
            AgentOutput::Pending {
                url,
                kind,
                description,
                field_key,
            } => AppEvent::AgentPending {
                url,
                kind,
                description,
                field_key,
            },
            AgentOutput::Answer { key, label, value } => {
                AppEvent::AgentAnswer { key, label, value }
            }
            AgentOutput::CvReview {
                score,
                target,
                report,
                keywords,
            } => AppEvent::AgentCvReview {
                score,
                target,
                report,
                keywords,
            },
            AgentOutput::CvImproved { content, target } => {
                AppEvent::AgentCvImproved { content, target }
            }
            AgentOutput::Feedback {
                summary,
                suggestions,
            } => AppEvent::AgentFeedback {
                summary,
                suggestions,
            },
            AgentOutput::Profile {
                background,
                cv_base,
                variants,
            } => AppEvent::AgentProfile {
                background,
                cv_base,
                variants,
            },
        }
    }
}

/// Converts a stream-json event to the corresponding [`AppEvent`].
///
/// Returns `None` for events without direct interest to the UI (hooks, rate
/// limit, raw `user`/tool_result, unknown).
pub fn app_event_from_stream(ev: &StreamEvent) -> Option<AppEvent> {
    match ev {
        StreamEvent::Assistant(_) => {
            if let Some(text) = ev.assistant_text() {
                return Some(AppEvent::AgentText(text));
            }
            // no text: might be just a tool_use
            if let Some((name, input)) = ev.tool_uses().into_iter().next() {
                return Some(AppEvent::AgentToolUse { name, input });
            }
            None
        }
        StreamEvent::Result(r) => Some(AppEvent::AgentFinished {
            result: r.result.clone(),
            num_turns: r.num_turns,
            cost_usd: r.total_cost_usd,
            is_error: r.is_error,
        }),
        // hooks, init, user/tool_result, rate limit, unknown → no UI event
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::stream::parse_line;

    #[test]
    fn assistant_text_becomes_agent_text() {
        let ev = parse_line(
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"hello"}]}}"#,
        )
        .unwrap();
        match app_event_from_stream(&ev) {
            Some(AppEvent::AgentText(t)) => assert_eq!(t, "hello"),
            other => panic!("expected AgentText, got {other:?}"),
        }
    }

    #[test]
    fn result_becomes_agent_finished() {
        let ev = parse_line(
            r#"{"type":"result","subtype":"success","is_error":false,"result":"done","num_turns":2}"#,
        )
        .unwrap();
        match app_event_from_stream(&ev) {
            Some(AppEvent::AgentFinished {
                result,
                num_turns,
                is_error,
                ..
            }) => {
                assert_eq!(result.as_deref(), Some("done"));
                assert_eq!(num_turns, Some(2));
                assert!(!is_error);
            }
            other => panic!("expected AgentFinished, got {other:?}"),
        }
    }

    #[test]
    fn hook_does_not_emit_event() {
        let ev = parse_line(r#"{"type":"system","subtype":"hook_started"}"#).unwrap();
        assert!(app_event_from_stream(&ev).is_none());
    }
}
