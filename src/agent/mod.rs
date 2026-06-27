//! Agent orchestration: assembles `claude` arguments, spawns via PTY,
//! parses stream-json + NDJSON protocol and emits [`AppEvent`]s.
//!
//! The agent **does not touch the DB** — it only emits events. Persistence is done
//! by the TUI loop, which is the exclusive owner of the SQLite connection.

pub mod prompts;
pub mod protocol;
pub mod pty;
pub mod stream;

use anyhow::Result;
use std::path::PathBuf;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::event::AppEvent;
use stream::{parse_line, StreamEvent};

/// Configuration for invoking `claude`.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Claude Code binary (usually "claude").
    pub claude_bin: String,
    /// Agent working directory.
    pub cwd: Option<PathBuf>,
    /// Enable Claude in Chrome integration (flag `--chrome`).
    pub chrome: bool,
    /// Skip permissions (headless autonomous mode).
    pub bypass_permissions: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            claude_bin: "claude".to_string(),
            cwd: None,
            chrome: true,
            bypass_permissions: true,
        }
    }
}

/// Summary of session results.
#[derive(Debug, Clone, Default)]
pub struct RunSummary {
    pub result: Option<String>,
    pub num_turns: Option<u32>,
    pub cost_usd: Option<f64>,
    pub is_error: bool,
}

/// Assembles arguments for a headless session with stream-json.
pub fn build_args(cfg: &AgentConfig, prompt: &str, resume_session: Option<&str>) -> Vec<String> {
    let mut args = vec![
        "-p".to_string(),
        prompt.to_string(),
        "--output-format".to_string(),
        "stream-json".to_string(),
        "--verbose".to_string(),
    ];
    if cfg.chrome {
        // Use the Claude in Chrome extension (real Chrome already logged in).
        args.push("--chrome".to_string());
    }
    if cfg.bypass_permissions {
        // Autonomous agent: tools (browser) run without interactive prompts.
        args.push("--permission-mode".to_string());
        args.push("bypassPermissions".to_string());
    }
    // Force use of real Chrome, avoiding Playwright MCP (own/logged-out browser).
    args.push("--disallowedTools".to_string());
    args.push("mcp__plugin_playwright_playwright".to_string());
    if let Some(sid) = resume_session {
        args.push("--resume".to_string());
        args.push(sid.to_string());
    }
    args
}

/// Runs a `claude` session from start to finish, emitting content events.
///
/// Does not emit `AgentStarted`/`AgentFinished` (that is the responsibility of whoever orchestrates
/// one or more sessions). Returns the session summary.
pub async fn run_session(
    cfg: &AgentConfig,
    prompt: &str,
    resume_session: Option<&str>,
    events: &UnboundedSender<AppEvent>,
) -> Result<RunSummary> {
    let args = build_args(cfg, prompt, resume_session);
    let (mut handle, rx) = match pty::spawn(&cfg.claude_bin, &args, cfg.cwd.as_deref()) {
        Ok(v) => v,
        Err(e) => {
            let _ = events.send(AppEvent::AgentError(format!("claude spawn failed: {e}")));
            return Err(e);
        }
    };

    let summary = process_stream(rx, events).await;
    let _ = handle.wait();
    Ok(summary)
}

/// Consumes stream-json lines, emits events and returns the summary.
///
/// Separated from [`run_session`] to be testable with a fake line stream.
pub async fn process_stream(
    mut rx: UnboundedReceiver<String>,
    events: &UnboundedSender<AppEvent>,
) -> RunSummary {
    let mut summary = RunSummary::default();

    while let Some(line) = rx.recv().await {
        let _ = events.send(AppEvent::AgentRaw(line.clone()));

        let Some(ev) = parse_line(&line) else {
            continue;
        };

        match &ev {
            StreamEvent::System(s) if s.is_init() => {
                if let Some(sid) = &s.session_id {
                    let _ = events.send(AppEvent::AgentSessionId(sid.clone()));
                }
            }
            StreamEvent::Assistant(_) => {
                // Assistant text may contain protocol lines (jobs,
                // applications, pending) and/or human narration.
                if let Some(text) = ev.assistant_text() {
                    for l in text.lines() {
                        if let Some(out) = protocol::parse(l) {
                            let _ = events.send(AppEvent::from_agent_output(out));
                        } else if !l.trim().is_empty() {
                            let _ = events.send(AppEvent::AgentText(l.to_string()));
                        }
                    }
                }
                for (name, input) in ev.tool_uses() {
                    let _ = events.send(AppEvent::AgentToolUse { name, input });
                }
            }
            StreamEvent::Result(r) => {
                summary = RunSummary {
                    result: r.result.clone(),
                    num_turns: r.num_turns,
                    cost_usd: r.total_cost_usd,
                    is_error: r.is_error,
                };
            }
            _ => {}
        }

        if ev.is_terminal() {
            break;
        }
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc::unbounded_channel;

    #[test]
    fn build_args_basic() {
        let cfg = AgentConfig::default();
        let a = build_args(&cfg, "hi", None);
        assert!(a.contains(&"-p".to_string()));
        assert!(a.contains(&"hi".to_string()));
        assert!(a
            .windows(2)
            .any(|w| w == ["--output-format", "stream-json"]));
        assert!(a.contains(&"--verbose".to_string()));
        assert!(!a.contains(&"--resume".to_string()));
        // by default: Chrome on, permission bypass, Playwright disabled
        assert!(a.contains(&"--chrome".to_string()));
        assert!(a
            .windows(2)
            .any(|w| w == ["--permission-mode", "bypassPermissions"]));
        assert!(a
            .windows(2)
            .any(|w| w == ["--disallowedTools", "mcp__plugin_playwright_playwright"]));
    }

    #[test]
    fn build_args_no_chrome_no_bypass() {
        let cfg = AgentConfig {
            chrome: false,
            bypass_permissions: false,
            ..Default::default()
        };
        let a = build_args(&cfg, "hi", None);
        assert!(!a.contains(&"--chrome".to_string()));
        assert!(!a.contains(&"bypassPermissions".to_string()));
    }

    #[test]
    fn build_args_with_resume() {
        let cfg = AgentConfig::default();
        let a = build_args(&cfg, "continue", Some("sess-123"));
        assert!(a.windows(2).any(|w| w == ["--resume", "sess-123"]));
    }

    #[tokio::test]
    async fn process_stream_on_real_claude_capture() {
        // Fixture captured from real `claude` emitting the protocol (Phase 4).
        const FIXTURE: &str = include_str!("../../tests/fixtures/stream-json-protocol.ndjson");
        let (line_tx, line_rx) = unbounded_channel::<String>();
        for l in FIXTURE.lines() {
            if !l.trim().is_empty() {
                line_tx.send(l.to_string()).unwrap();
            }
        }
        drop(line_tx);

        let (ev_tx, mut ev_rx) = unbounded_channel::<AppEvent>();
        let summary = process_stream(line_rx, &ev_tx).await;
        assert!(!summary.is_error);

        let mut jobs = 0;
        let mut pendings = 0;
        while let Ok(ev) = ev_rx.try_recv() {
            match ev {
                AppEvent::AgentJobFound(_) => jobs += 1,
                AppEvent::AgentPending { .. } => pendings += 1,
                _ => {}
            }
        }
        assert_eq!(jobs, 1, "should extract 1 job from real capture");
        assert_eq!(pendings, 1, "should extract 1 pending from real capture");
    }

    #[tokio::test]
    async fn process_stream_emits_jobs_pending_and_summary() {
        // Fake stream: init, assistant with 1 job + narration + 1 pending, result.
        let (line_tx, line_rx) = unbounded_channel::<String>();
        line_tx
            .send(r#"{"type":"system","subtype":"init","session_id":"sess-1"}"#.into())
            .unwrap();
        line_tx
            .send(
                r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Searching for jobs...\n{\"type\":\"job\",\"title\":\"Dev\",\"url\":\"https://x/1\",\"fit_score\":0.9}\n{\"type\":\"pending\",\"kind\":\"captcha\",\"description\":\"resolve\",\"url\":\"https://x/1\"}"}]}}"#.into(),
            )
            .unwrap();
        line_tx
            .send(
                r#"{"type":"result","subtype":"success","is_error":false,"result":"ok","num_turns":2,"total_cost_usd":0.05}"#.into(),
            )
            .unwrap();
        drop(line_tx);

        let (ev_tx, mut ev_rx) = unbounded_channel::<AppEvent>();
        let summary = process_stream(line_rx, &ev_tx).await;

        assert_eq!(summary.result.as_deref(), Some("ok"));
        assert_eq!(summary.num_turns, Some(2));

        let mut session_id = None;
        let mut jobs = 0;
        let mut pendings = 0;
        let mut texts = 0;
        while let Ok(ev) = ev_rx.try_recv() {
            match ev {
                AppEvent::AgentSessionId(s) => session_id = Some(s),
                AppEvent::AgentJobFound(_) => jobs += 1,
                AppEvent::AgentPending { .. } => pendings += 1,
                AppEvent::AgentText(_) => texts += 1,
                _ => {}
            }
        }
        assert_eq!(session_id.as_deref(), Some("sess-1"));
        assert_eq!(jobs, 1, "should emit 1 job");
        assert_eq!(pendings, 1, "should emit 1 pending");
        assert!(texts >= 1, "should emit human narration");
    }
}
