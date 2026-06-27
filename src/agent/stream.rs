//! Parser for `claude`'s `--output-format stream-json` NDJSON.
//!
//! Each stdout line is an independent JSON object, discriminated by the
//! `type` field. Real format captured in Phase 0 spike — see
//! `docs/stream-json-format.md` and the fixture `tests/fixtures/stream-json-hello.ndjson`.
//!
//! The parser is **tolerant**: any empty line, non-JSON, or unknown type becomes
//! [`StreamEvent::Unknown`] instead of breaking the flow.

use serde::Deserialize;

/// A typed stream-json event.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    /// System events (session init, hooks, etc.).
    System(SystemEvent),
    /// Assistant message (text and/or tool calls).
    Assistant(MessageEvent),
    /// "User" message — typically tool results.
    User(MessageEvent),
    /// Terminal event of the turn.
    Result(ResultEvent),
    /// Rate limit information.
    RateLimitEvent(RateLimitEvent),
    /// Any other type (tolerance for format evolution).
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SystemEvent {
    #[serde(default)]
    pub subtype: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
}

impl SystemEvent {
    /// `true` if it's the session initialization event.
    pub fn is_init(&self) -> bool {
        self.subtype.as_deref() == Some("init")
    }

    /// `true` if it's hook noise (to be ignored in UI).
    pub fn is_hook(&self) -> bool {
        matches!(self.subtype.as_deref(), Some(s) if s.starts_with("hook_"))
    }
}

/// Message (assistant or user). They share the same envelope.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageEvent {
    #[serde(default)]
    pub session_id: Option<String>,
    pub message: Message,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub content: Vec<ContentBlock>,
    #[serde(default)]
    pub usage: Option<Usage>,
}

/// A content block within a message.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text {
        text: String,
    },
    ToolUse {
        #[serde(default)]
        id: String,
        name: String,
        #[serde(default)]
        input: serde_json::Value,
    },
    ToolResult {
        #[serde(default)]
        content: serde_json::Value,
        #[serde(default)]
        is_error: bool,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResultEvent {
    #[serde(default)]
    pub subtype: Option<String>,
    #[serde(default)]
    pub is_error: bool,
    #[serde(default)]
    pub result: Option<String>,
    #[serde(default)]
    pub num_turns: Option<u32>,
    #[serde(default)]
    pub total_cost_usd: Option<f64>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Usage {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitEvent {
    #[serde(default)]
    pub rate_limit_info: serde_json::Value,
}

impl StreamEvent {
    /// The `session_id` of the event, if present.
    pub fn session_id(&self) -> Option<&str> {
        match self {
            StreamEvent::System(e) => e.session_id.as_deref(),
            StreamEvent::Assistant(e) | StreamEvent::User(e) => e.session_id.as_deref(),
            StreamEvent::Result(e) => e.session_id.as_deref(),
            StreamEvent::RateLimitEvent(_) | StreamEvent::Unknown => None,
        }
    }

    /// Concatenates the text of `Text` blocks from an assistant message.
    pub fn assistant_text(&self) -> Option<String> {
        if let StreamEvent::Assistant(e) = self {
            let mut out = String::new();
            for block in &e.message.content {
                if let ContentBlock::Text { text } = block {
                    out.push_str(text);
                }
            }
            if out.is_empty() {
                None
            } else {
                Some(out)
            }
        } else {
            None
        }
    }

    /// Tool calls `(name, input)` in an assistant message.
    pub fn tool_uses(&self) -> Vec<(String, serde_json::Value)> {
        let mut out = Vec::new();
        if let StreamEvent::Assistant(e) = self {
            for block in &e.message.content {
                if let ContentBlock::ToolUse { name, input, .. } = block {
                    out.push((name.clone(), input.clone()));
                }
            }
        }
        out
    }

    /// `true` if it's the terminal event of the turn.
    pub fn is_terminal(&self) -> bool {
        matches!(self, StreamEvent::Result(_))
    }

    /// `true` if it's Claude Code hook noise (ignore in UI).
    pub fn is_hook_noise(&self) -> bool {
        matches!(self, StreamEvent::System(e) if e.is_hook())
    }
}

/// Parses **one line** of stream-json.
///
/// Returns `None` for empty/whitespace-only lines. Non-JSON or unexpected format lines
/// return `Some(StreamEvent::Unknown)` (never breaks).
pub fn parse_line(line: &str) -> Option<StreamEvent> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    match serde_json::from_str::<StreamEvent>(trimmed) {
        Ok(ev) => Some(ev),
        Err(err) => {
            tracing::debug!(%err, line = trimmed, "unrecognized stream-json line");
            Some(StreamEvent::Unknown)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../tests/fixtures/stream-json-hello.ndjson");

    fn parse_all(input: &str) -> Vec<StreamEvent> {
        input.lines().filter_map(parse_line).collect()
    }

    #[test]
    fn empty_line_becomes_none() {
        assert!(parse_line("").is_none());
        assert!(parse_line("   ").is_none());
    }

    #[test]
    fn non_json_line_becomes_unknown() {
        let ev = parse_line("this is not json").unwrap();
        assert!(matches!(ev, StreamEvent::Unknown));
    }

    #[test]
    fn unknown_type_becomes_unknown() {
        let ev = parse_line(r#"{"type":"something_new","x":1}"#).unwrap();
        assert!(matches!(ev, StreamEvent::Unknown));
    }

    #[test]
    fn parses_system_init() {
        let line = r#"{"type":"system","subtype":"init","session_id":"abc","model":"claude-opus-4-8","cwd":"/tmp"}"#;
        let ev = parse_line(line).unwrap();
        match &ev {
            StreamEvent::System(s) => {
                assert!(s.is_init());
                assert!(!s.is_hook());
            }
            other => panic!("expected System, got {other:?}"),
        }
        assert_eq!(ev.session_id(), Some("abc"));
    }

    #[test]
    fn parses_assistant_text() {
        let line = r#"{"type":"assistant","session_id":"s1","message":{"role":"assistant","content":[{"type":"text","text":"hi"}]}}"#;
        let ev = parse_line(line).unwrap();
        assert_eq!(ev.assistant_text().as_deref(), Some("hi"));
        assert!(!ev.is_terminal());
    }

    #[test]
    fn parses_tool_use() {
        let line = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","id":"t1","name":"Bash","input":{"command":"ls"}}]}}"#;
        let ev = parse_line(line).unwrap();
        let tools = ev.tool_uses();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].0, "Bash");
        assert_eq!(tools[0].1["command"], "ls");
    }

    #[test]
    fn parses_terminal_result() {
        let line = r#"{"type":"result","subtype":"success","is_error":false,"result":"hi","num_turns":1,"total_cost_usd":0.08,"session_id":"s1","usage":{"output_tokens":4}}"#;
        let ev = parse_line(line).unwrap();
        assert!(ev.is_terminal());
        match ev {
            StreamEvent::Result(r) => {
                assert!(!r.is_error);
                assert_eq!(r.result.as_deref(), Some("hi"));
                assert_eq!(r.num_turns, Some(1));
                assert_eq!(r.usage.unwrap().output_tokens, 4);
            }
            other => panic!("expected Result, got {other:?}"),
        }
    }

    #[test]
    fn fixture_real_parses_without_panic_and_has_key_events() {
        let events = parse_all(FIXTURE);
        assert!(!events.is_empty(), "fixture should produce events");

        // No known event should fall into Unknown (format is stable).
        let unknowns = events
            .iter()
            .filter(|e| matches!(e, StreamEvent::Unknown))
            .count();
        assert_eq!(unknowns, 0, "real fixture should not have Unknown events");

        // There should be an init with session_id.
        let init = events.iter().find_map(|e| match e {
            StreamEvent::System(s) if s.is_init() => Some(s),
            _ => None,
        });
        assert!(init.is_some(), "should have a system/init event");
        assert!(init.unwrap().session_id.is_some());

        // There should be the assistant text from the captured fixture.
        let texto = events.iter().find_map(|e| e.assistant_text());
        assert_eq!(texto.as_deref(), Some("oi"));

        // There should be exactly one terminal event (result).
        let terminais = events.iter().filter(|e| e.is_terminal()).count();
        assert_eq!(terminais, 1, "should have 1 terminal result event");
    }

    #[test]
    fn hooks_are_marked_as_noise() {
        let line = r#"{"type":"system","subtype":"hook_started","hook_name":"X"}"#;
        let ev = parse_line(line).unwrap();
        assert!(ev.is_hook_noise());
    }
}
