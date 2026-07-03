//! Desktop notifications (best-effort).
//!
//! Linux (freedesktop/D-Bus) and macOS (native) via `notify-rust`. Windows is a
//! no-op for now — the WinRT toast backend needs a registered AppUserModelID,
//! and notifications are best-effort anyway. Freedesktop icon names are Linux-only.

/// Notifies of a pending action that requires user intervention.
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn pending(kind: &str, description: &str) {
    let mut n = notify_rust::Notification::new();
    n.summary(&format!("jobRabbit — action needed: {kind}"))
        .body(description);
    #[cfg(target_os = "linux")]
    {
        n.icon("dialog-warning")
            .timeout(notify_rust::Timeout::Milliseconds(8000));
    }
    let _ = n.show();
}

/// Generic notification (e.g., end of agent execution).
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn info(summary: &str, body: &str) {
    let mut n = notify_rust::Notification::new();
    n.summary(summary).body(body);
    #[cfg(target_os = "linux")]
    {
        n.icon("dialog-information");
    }
    let _ = n.show();
}

/// No-op notifications on Windows (best-effort feature).
#[cfg(target_os = "windows")]
pub fn pending(_kind: &str, _description: &str) {}

/// No-op notifications on Windows (best-effort feature).
#[cfg(target_os = "windows")]
pub fn info(_summary: &str, _body: &str) {}
