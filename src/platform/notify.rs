//! Desktop notifications (freedesktop/D-Bus) via notify-rust.
//!
//! Best-effort: in environments without D-Bus/notification daemon (e.g., inside a
//! build container), calls simply fail silently.

/// Notifies of a pending action that requires user intervention.
pub fn pending(kind: &str, description: &str) {
    let _ = notify_rust::Notification::new()
        .summary(&format!("jobRabbit — action needed: {kind}"))
        .body(description)
        .icon("dialog-warning")
        .timeout(notify_rust::Timeout::Milliseconds(8000))
        .show();
}

/// Generic notification (e.g., end of agent execution).
pub fn info(summary: &str, body: &str) {
    let _ = notify_rust::Notification::new()
        .summary(summary)
        .body(body)
        .icon("dialog-information")
        .show();
}
