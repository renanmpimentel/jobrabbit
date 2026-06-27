//! Idle detection (X11 / Wayland-D-Bus) via user-idle.
//!
//! Best-effort: in environments without a graphical server (e.g., build container) the
//! queries fail and [`current_idle`] returns `None`.

use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

use crate::event::AppEvent;

/// Time since the user's last activity, if detectable.
pub fn current_idle() -> Option<Duration> {
    user_idle::UserIdle::get_time()
        .ok()
        .map(|t| Duration::from_secs(t.as_seconds()))
}

/// Task that watches for idleness and emits [`AppEvent::IdleReached`] once
/// when the threshold is crossed; rearms when the user interacts again.
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
