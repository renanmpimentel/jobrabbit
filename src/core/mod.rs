//! Presentation-agnostic core.
//!
//! Brings together logic that **does not depend** on the UI layer (TUI or web):
//! - [`persist`]: persists agent [`crate::event::AppEvent`] in the database.
//! - [`actions`]: assembles prompts and orchestrates agent execution.
//!
//! Both TUI (ratatui) and web backend (axum) consume this module,
//! reacting to [`EventOutcome`](persist::EventOutcome) with their own UI.

pub mod actions;
pub mod doctor;
pub mod persist;

/// Agent execution state. Shared by the TUI and web.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AgentStatus {
    Idle,
    Running,
    Error(String),
}

impl AgentStatus {
    /// Short label for status bar / headers.
    pub fn label(&self) -> String {
        match self {
            AgentStatus::Idle => "● idle".to_string(),
            AgentStatus::Running => "▶ running".to_string(),
            AgentStatus::Error(e) => format!("✖ error: {e}"),
        }
    }
}
