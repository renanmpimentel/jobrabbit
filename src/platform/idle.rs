//! Idle detection for the optional auto-run-on-idle feature.
//!
//! Linux only for now (X11 / Wayland via `user-idle`). On macOS/Windows it is a
//! no-op: [`current_idle`] returns `None` and [`watch`] exits immediately, so the
//! feature simply never triggers there. Best-effort even on Linux: in a headless
//! environment the queries fail and `current_idle` returns `None`.

use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

use crate::event::AppEvent;

/// Time since the user's last activity, if detectable (Linux only).
pub fn current_idle() -> Option<Duration> {
    #[cfg(target_os = "linux")]
    {
        user_idle::UserIdle::get_time()
            .ok()
            .map(|t| Duration::from_secs(t.as_seconds()))
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

/// Task that watches for idleness and emits [`AppEvent::IdleReached`] once when
/// the threshold is crossed; rearms when the user interacts again. (Linux only.)
#[cfg(target_os = "linux")]
pub async fn watch(threshold: Duration, tx: UnboundedSender<AppEvent>) {
    // Without a graphical session (e.g., container/headless) the X11/Wayland queries
    // from `user-idle` can fail abruptly. In those cases, we don't watch.
    if std::env::var_os("DISPLAY").is_none() && std::env::var_os("WAYLAND_DISPLAY").is_none() {
        tracing::info!("idle watch disabled (no DISPLAY/WAYLAND_DISPLAY)");
        return;
    }
    let mut fired = false;
    let mut tick = tokio::time::interval(Duration::from_secs(5));
    loop {
        tick.tick().await;
        if let Some(idle) = current_idle() {
            if idle >= threshold && !fired {
                fired = true;
                if tx.send(AppEvent::IdleReached).is_err() {
                    break; // app exited
                }
            } else if idle < threshold {
                fired = false;
            }
        }
    }
}

/// No-op idle watch on non-Linux platforms (auto-run-on-idle is Linux-only for now).
#[cfg(not(target_os = "linux"))]
pub async fn watch(_threshold: Duration, _tx: UnboundedSender<AppEvent>) {}
