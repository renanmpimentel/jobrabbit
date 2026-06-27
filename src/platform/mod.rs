//! Linux-specific integrations: idle detection, notifications, keyring.
//!
//! Phase 0 stub — implementation in Phase 5.

pub mod idle;
pub mod notify;
pub mod secrets;

use std::path::PathBuf;

/// Locates an executable in PATH (like `which`). Accepts absolute paths.
pub fn which(bin: &str) -> Option<PathBuf> {
    if bin.contains('/') {
        let p = PathBuf::from(bin);
        return p.is_file().then_some(p);
    }
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|dir| dir.join(bin))
            .find(|p| p.is_file())
    })
}

/// `true` if the Claude Code binary is available.
pub fn claude_available(bin: &str) -> bool {
    which(bin).is_some()
}

/// Opens a URL in the user's default browser (xdg-open).
///
/// Best-effort: errors are silenced (e.g., running without graphical environment).
pub fn open_url(url: &str) {
    let _ = std::process::Command::new("xdg-open")
        .arg(url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}
