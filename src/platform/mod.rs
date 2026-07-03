//! Desktop integrations: idle detection, notifications, keyring, link opening.
//!
//! Cross-platform: Linux (freedesktop/D-Bus/X11), macOS and Windows. Platform
//! differences are handled per-OS inside each submodule; the public API is the same.

pub mod idle;
pub mod notify;
pub mod secrets;

use std::path::{Path, PathBuf};

/// Locates an executable in PATH (like `which`/`where`). Accepts absolute paths.
///
/// On Windows, bare names are also tried with the executable suffixes from
/// `PATHEXT` (defaulting to `.exe;.cmd;.bat`).
pub fn which(bin: &str) -> Option<PathBuf> {
    if Path::new(bin).is_absolute() {
        let p = PathBuf::from(bin);
        return p.is_file().then_some(p);
    }
    let candidates = |dir: &Path| -> Vec<PathBuf> {
        let direct = dir.join(bin);
        #[cfg(windows)]
        {
            // Only append extensions when the name has none.
            if Path::new(bin).extension().is_none() {
                let pathext =
                    std::env::var("PATHEXT").unwrap_or_else(|_| ".EXE;.CMD;.BAT".to_string());
                let mut v = vec![direct.clone()];
                for ext in pathext.split(';').filter(|e| !e.is_empty()) {
                    v.push(dir.join(format!("{bin}{}", ext.to_ascii_lowercase())));
                }
                return v;
            }
        }
        vec![direct]
    };
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .flat_map(|dir| candidates(&dir))
            .find(|p| p.is_file())
    })
}

/// `true` if the Claude Code binary is available.
pub fn claude_available(bin: &str) -> bool {
    which(bin).is_some()
}

/// Opens a URL in the user's default browser.
///
/// Uses the platform opener (`xdg-open` on Linux, `open` on macOS,
/// `cmd /c start` on Windows). Best-effort: errors are silenced (e.g., running
/// without a graphical environment).
pub fn open_url(url: &str) {
    #[cfg(target_os = "linux")]
    let mut cmd = {
        let mut c = std::process::Command::new("xdg-open");
        c.arg(url);
        c
    };
    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = std::process::Command::new("open");
        c.arg(url);
        c
    };
    #[cfg(target_os = "windows")]
    let mut cmd = {
        // `start` is a cmd builtin; the empty "" is the window title placeholder.
        let mut c = std::process::Command::new("cmd");
        c.args(["/C", "start", "", url]);
        c
    };
    let _ = cmd
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}
