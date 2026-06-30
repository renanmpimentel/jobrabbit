//! Persistence derived from agent events.
//!
//! [`apply_event`] is the single source of truth for transforming an
//! [`AppEvent`] into database writes. The UI reaction (logging, switching tabs,
//! updating status) is returned via [`EventOutcome`] so each frontend
//! (TUI or web) can decide how to present it.

use crate::core::AgentStatus;
use crate::db::Db;
use crate::event::AppEvent;

/// Suggested tab indices (mirror `tui::TABS`). TUI uses to focus;
/// the web frontend can ignore or map to its own navigation.
pub mod tab {
    pub const PROFILE: usize = 1;
    pub const SESSION: usize = 4;
    pub const ATS: usize = 6;
}

/// What the UI should do after [`apply_event`] persists an event.
#[derive(Debug, Default, Clone)]
pub struct EventOutcome {
    /// Lines for the session log (ready for display).
    pub logs: Vec<String>,
    /// Tab the UI should focus on (index of `tui::TABS`), if any.
    pub focus_tab: Option<usize>,
    /// New agent status, if changed.
    pub status: Option<AgentStatus>,
    /// `true` if the UI should reload its data from the database.
    pub refresh: bool,
    /// Desktop notification to emit: `(kind, description)`.
    pub notify: Option<(String, String)>,
}

impl EventOutcome {
    fn log(&mut self, line: impl Into<String>) {
        self.logs.push(line.into());
    }
}

/// Persists an [`AppEvent`] in the database and describes the UI reaction via [`EventOutcome`].
///
/// `session_id` is the current session id in the database (state maintained by the
/// frontend): `AgentStarted` creates it, `AgentFinished` consumes it.
///
/// Events purely from UI/loop (`Tick`, `Quit`, `IdleReached`) return an
/// empty outcome — the frontend handles it.
pub fn apply_event(db: &Db, ev: &AppEvent, session_id: &mut Option<i64>) -> EventOutcome {
    let mut out = EventOutcome::default();
    match ev {
        AppEvent::Tick | AppEvent::Quit | AppEvent::IdleReached => {}

        AppEvent::AgentStarted => {
            *session_id = db.start_session(None).ok();
            out.status = Some(AgentStatus::Running);
            out.log("▶ agent execution started");
        }
        AppEvent::AgentSessionId(sid) => {
            if let Some(id) = *session_id {
                let _ = db.set_session_claude_id(id, sid);
            }
        }
        AppEvent::AgentText(t) => out.log(t.clone()),
        AppEvent::AgentToolUse { name, input } => {
            out.log(friendly_tool(name, input));
        }
        AppEvent::AgentJobFound(job) => {
            let title = job.title.clone();
            let fit = job.fit_score.unwrap_or(0.0);
            if db.upsert_job(job).is_ok() {
                out.log(format!("＋ job: {title} (fit {fit:.2})"));
                out.refresh = true;
            }
        }
        AppEvent::AgentApplication {
            url,
            status,
            cv,
            cover,
            screenshot,
        } => {
            if let Ok(Some(job_id)) = db.job_id_by_url(url) {
                // Does an application exist? Update status (preserve CV/letter); otherwise insert.
                match db.application_for_job(job_id) {
                    Ok(Some(_)) => {
                        let _ = db.set_application_status(job_id, status);
                    }
                    _ => {
                        let _ = db.add_application(job_id, status, cv.as_deref(), cover.as_deref());
                    }
                }
                // If screenshot path is provided, store it.
                if let Some(shot) = screenshot {
                    if !shot.trim().is_empty() {
                        let _ = db.set_application_screenshot(job_id, shot);
                    }
                }
                // Review mode: 'ready' becomes an approval item in the Pending tab.
                if status == "ready" {
                    let title = db
                        .get_job(job_id)
                        .ok()
                        .flatten()
                        .map(|j| format!("{} @ {}", j.title, j.company))
                        .unwrap_or_else(|| url.clone());
                    let _ = db.add_pending(
                        Some(job_id),
                        "approval",
                        &format!("Approve application: {title}"),
                        Some(url),
                    );
                }
                // Application confirmed: remove the corresponding approval item.
                if status == "applied" {
                    resolve_approval_for(db, job_id);
                }
                if status == "failed" {
                    out.log(
                        "✖ application NOT completed — see the session log; the approval item was kept for retry.",
                    );
                }
                let icon = match status.as_str() {
                    "ready" => "📝",
                    "dry_run" => "🧪",
                    "applied" => "✅",
                    "failed" => "✖",
                    _ => "•",
                };
                out.log(format!("{icon} application [{status}]: {url}"));
                out.refresh = true;
            } else {
                out.log(format!("⚠ application with unknown job: {url}"));
            }
        }
        AppEvent::AgentPending {
            url,
            kind,
            description,
            field_key,
        } => {
            let job_id = url
                .as_deref()
                .and_then(|u| db.job_id_by_url(u).ok().flatten());
            let _ = db.add_pending_full(
                job_id,
                kind,
                description,
                url.as_deref(),
                field_key.as_deref(),
            );
            let icon = match kind.as_str() {
                "answer_needed" => "❓",
                "login" => "🔑",
                "captcha" => "🧩",
                _ => "⛔",
            };
            out.log(format!("{icon} pending [{kind}]: {description}"));
            out.notify = Some((kind.clone(), description.clone()));
            out.refresh = true;
        }
        AppEvent::AgentAnswer { key, label, value } => {
            let _ = db.set_answer(key, label, value);
            out.log(format!("🗂 learned answer: {key} = {value}"));
        }
        AppEvent::AgentFeedback {
            summary,
            suggestions,
        } => {
            let _ = db.add_feedback(summary, suggestions);
            out.log(format!("📊 feedback: {summary}"));
            out.refresh = true;
        }
        AppEvent::AgentCvReview {
            score,
            target,
            report,
        } => {
            let _ = db.add_cv_review(*score as i64, target, report);
            out.log(format!(
                "📋 CV evaluation: score {score}/100 (target: {target})"
            ));
            out.focus_tab = Some(tab::ATS);
            out.refresh = true;
        }
        AppEvent::AgentCvImproved { content, target } => {
            let _ = db.add_cv_version(target, content);
            out.log(format!(
                "✨ improved CV version generated ({} chars, target: {target})",
                content.len()
            ));
            out.focus_tab = Some(tab::ATS);
            out.refresh = true;
        }
        AppEvent::AgentProfile {
            background,
            cv_base,
            variants,
        } => {
            // background+cv replace; variants are added without duplicating label.
            let _ = db.save_profile(background, cv_base);
            let existentes: std::collections::HashSet<String> = db
                .list_variants()
                .unwrap_or_default()
                .into_iter()
                .map(|v| v.label.to_lowercase())
                .collect();
            let mut add = 0;
            for (label, query) in variants {
                if !existentes.contains(&label.to_lowercase())
                    && db.add_variant(label, query).is_ok()
                {
                    add += 1;
                }
            }
            out.log(format!(
                "👤 profile imported ({} chars of CV, +{add} variant(s))",
                cv_base.len()
            ));
            out.focus_tab = Some(tab::PROFILE);
            out.refresh = true;
        }
        AppEvent::AgentRaw(_) => {} // detalhe cru fica no log de arquivo
        AppEvent::AgentFinished {
            result,
            num_turns,
            cost_usd,
            is_error,
        } => {
            out.status = Some(if *is_error {
                AgentStatus::Error("execution with errors".into())
            } else {
                AgentStatus::Idle
            });
            if let Some(id) = session_id.take() {
                let _ = db.finish_session(
                    id,
                    result.as_deref(),
                    num_turns.map(|n| n as i64),
                    *cost_usd,
                    None,
                );
            }
            if let Some(r) = result {
                out.log(format!("✔ {r}"));
            }
            out.log(format!(
                "— end (turns: {}, cost: ${:.4})",
                num_turns.unwrap_or(0),
                cost_usd.unwrap_or(0.0)
            ));
            out.refresh = true;
        }
        AppEvent::AgentError(e) => {
            out.status = Some(AgentStatus::Error(e.clone()));
            out.log(format!("✖ error: {e}"));
        }
    }
    out
}

/// Translates an agent tool call into a friendly log line.
///
/// The raw tools (e.g., `mcp__claude-in-chrome__get_page_text`) don't tell
/// anything to the user; here they become readable actions ("👀 reading the page", etc.).
fn friendly_tool(name: &str, input: &serde_json::Value) -> String {
    let short = name.rsplit("__").next().unwrap_or(name);
    let url = input.get("url").and_then(|v| v.as_str());
    let action = input.get("action").and_then(|v| v.as_str());
    match short {
        "navigate" => match url {
            Some(u) => format!("🌐 opening {}", short_url(u)),
            None => "🌐 navigating".into(),
        },
        "computer" => match action {
            Some("screenshot") => "📸 viewing the page".into(),
            Some("left_click") | Some("right_click") | Some("click") | Some("mouse_click") => {
                "🖱️ clicking".into()
            }
            Some("type") => "⌨️ typing".into(),
            Some("key") | Some("keypress") => "⌨️ using the keyboard".into(),
            Some("scroll") => "🖱️ scrolling the page".into(),
            _ => "🖱️ interacting with the page".into(),
        },
        "get_page_text" | "read_page" | "get_dom" => "👀 reading the page".into(),
        "form_input" | "fill_form" => "⌨️ filling the form".into(),
        "file_upload" | "upload_image" => "📎 uploading file".into(),
        "find" => "🔎 searching on the page".into(),
        "navigate_back" => "↩️ going back".into(),
        "read_console_messages" => "🐛 reading the console".into(),
        "read_network_requests" => "📡 inspecting the network".into(),
        s if s.starts_with("tabs_create") => "🗂️ opening a tab".into(),
        s if s.starts_with("tabs_close") => "🗂️ closing the tab".into(),
        s if s.starts_with("tabs_context") => "🗂️ checking the tabs".into(),
        "WebFetch" | "WebSearch" => match url {
            Some(u) => format!("🔎 searching on the web: {}", short_url(u)),
            None => "🔎 searching on the web".into(),
        },
        "Bash" => {
            let cmd = input.get("command").and_then(|v| v.as_str()).unwrap_or("");
            format!("💻 {}", truncate(cmd, 60))
        }
        "Read" => "📄 reading a file".into(),
        "Write" | "Edit" | "MultiEdit" => "✏️ writing a file".into(),
        // Unknown: uses a short and clean name, without the `mcp__...__` prefix.
        other => format!("🔧 {}", other.replace('_', " ")),
    }
}

/// Shortens a URL to `host/path…` (no scheme, no long querystring).
fn short_url(u: &str) -> String {
    let s = u
        .strip_prefix("https://")
        .or_else(|| u.strip_prefix("http://"))
        .unwrap_or(u);
    let s = s.split('?').next().unwrap_or(s);
    truncate(s.trim_end_matches('/'), 50)
}

fn truncate(s: &str, max: usize) -> String {
    let s = s.trim();
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max).collect();
        format!("{cut}…")
    }
}

/// Resolves (marks as completed) the pending approval item for a job.
pub fn resolve_approval_for(db: &Db, job_id: i64) {
    if let Ok(list) = db.list_pending(false) {
        for p in list
            .iter()
            .filter(|p| p.kind == "approval" && p.job_id == Some(job_id))
        {
            let _ = db.resolve_pending(p.id);
        }
    }
}
