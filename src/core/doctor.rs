//! Environment diagnostic ("doctor"): checks dependencies and configuration.
//!
//! Shared logic between CLI (`--doctor`) and web (`GET /api/doctor`).
//! Each [`Check`] has a status and, when something is wrong, a correction hint.

use serde::Serialize;

use crate::config::Settings;
use crate::db::Db;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum Status {
    Ok,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Serialize)]
pub struct Check {
    pub name: String,
    pub status: Status,
    pub detail: String,
    /// How to fix, when `status != Ok`.
    pub hint: Option<String>,
}

impl Check {
    fn ok(name: &str, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: Status::Ok,
            detail: detail.into(),
            hint: None,
        }
    }
    fn warn(name: &str, detail: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: Status::Warn,
            detail: detail.into(),
            hint: Some(hint.into()),
        }
    }
    fn fail(name: &str, detail: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: Status::Fail,
            detail: detail.into(),
            hint: Some(hint.into()),
        }
    }
}

/// First binary found in PATH among the candidates.
fn first_in_path(cands: &[&str]) -> Option<String> {
    cands
        .iter()
        .find_map(|b| crate::platform::which(b).map(|p| p.display().to_string()))
}

/// Runs all environment/configuration checks.
pub fn run(db: &Db, settings: &Settings) -> Vec<Check> {
    let mut checks = Vec::new();

    // 1) Claude Code CLI — mandatory.
    match crate::platform::which(&settings.claude_bin) {
        Some(p) => checks.push(Check::ok(
            "Claude Code CLI",
            format!("`{}` found in {}", settings.claude_bin, p.display()),
        )),
        None => checks.push(Check::fail(
            "Claude Code CLI",
            format!("`{}` is not in PATH", settings.claude_bin),
            "Install Claude Code and authenticate (`claude`); or adjust the path in the Config tab.",
        )),
    }

    // 2) Google Chrome — required for Claude in Chrome integration.
    match first_in_path(&[
        "google-chrome",
        "google-chrome-stable",
        "chromium",
        "chromium-browser",
        "chrome",
    ]) {
        Some(p) => checks.push(Check::ok(
            "Google Chrome",
            format!("found in {p} — confirm that the Claude in Chrome extension is installed and logged in"),
        )),
        None if settings.use_chrome => checks.push(Check::warn(
            "Google Chrome",
            "browser not found in PATH",
            "Install Google Chrome and the Claude in Chrome extension (automation depends on logged-in Chrome).",
        )),
        None => checks.push(Check::ok("Google Chrome", "not required (Use Claude in Chrome is off)")),
    }

    // 3) Graphical session — Chrome and automatic browser opening depend on this.
    if std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some() {
        checks.push(Check::ok("Graphical session", "DISPLAY/Wayland detected"));
    } else {
        checks.push(Check::warn(
            "Graphical session",
            "no DISPLAY/WAYLAND_DISPLAY",
            "Run on your desktop (with graphical environment) for the UI to open and Chrome to work.",
        ));
    }

    // 4) xdg-open — used to open jobs/browser.
    if crate::platform::which("xdg-open").is_some() {
        checks.push(Check::ok("Open links (xdg-open)", "available"));
    } else {
        checks.push(Check::warn(
            "Open links (xdg-open)",
            "`xdg-open` not found",
            "Install `xdg-utils` to open jobs and the browser automatically.",
        ));
    }

    // 5) Candidate profile.
    let profile = db.get_profile().unwrap_or_default();
    if profile.background.trim().is_empty() && profile.cv_base.trim().is_empty() {
        checks.push(Check::warn(
            "Candidate profile",
            "background and base CV empty",
            "Fill in the Profile or import your CV/LinkedIn (Profile tab).",
        ));
    } else {
        checks.push(Check::ok(
            "Candidate profile",
            format!("{} chars of base CV", profile.cv_base.len()),
        ));
    }

    // 6) Active search variants.
    let ativas = db
        .list_variants()
        .unwrap_or_default()
        .into_iter()
        .filter(|v| v.enabled)
        .count();
    if ativas == 0 {
        checks.push(Check::warn(
            "Search variants",
            "no active variant",
            "Add/enable at least one search variant in the Profile tab.",
        ));
    } else {
        checks.push(Check::ok("Search variants", format!("{ativas} active")));
    }

    // 7) CV file for upload (optional).
    let cv_path = settings.cv_file_path.trim();
    if cv_path.is_empty() {
        checks.push(Check::ok(
            "CV file (upload)",
            "not set (optional — only needed when the site requires upload)",
        ));
    } else if std::path::Path::new(cv_path).is_file() {
        checks.push(Check::ok("CV file (upload)", cv_path.to_string()));
    } else {
        checks.push(Check::warn(
            "CV file (upload)",
            format!("the configured path does not exist: {cv_path}"),
            "Point to a valid PDF/DOCX in the Config tab (or import a CV from file).",
        ));
    }

    // 8) Database / data directory.
    match crate::config::data_dir() {
        Ok(dir) => checks.push(Check::ok(
            "Database",
            format!("accessible in {}", dir.display()),
        )),
        Err(e) => checks.push(Check::fail(
            "Database",
            format!("could not resolve data directory: {e}"),
            "Check permissions of XDG data directory (~/.local/share/jobrabbit).",
        )),
    }

    checks
}

/// Summary: `(ok, warn, fail)`.
pub fn summary(checks: &[Check]) -> (usize, usize, usize) {
    let mut t = (0, 0, 0);
    for c in checks {
        match c.status {
            Status::Ok => t.0 += 1,
            Status::Warn => t.1 += 1,
            Status::Fail => t.2 += 1,
        }
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;

    #[test]
    fn run_cobre_checks_essenciais() {
        let db = Db::open_in_memory().unwrap();
        let settings = Settings::default();
        let checks = run(&db, &settings);
        let nomes: Vec<&str> = checks.iter().map(|c| c.name.as_str()).collect();
        assert!(nomes.contains(&"Claude Code CLI"));
        assert!(nomes.contains(&"Search variants"));
        assert!(nomes.contains(&"Database"));
        // the summary adds up to the total checks
        let (ok, warn, fail) = summary(&checks);
        assert_eq!(ok + warn + fail, checks.len());
        // in-memory database always accessible
        let db_check = checks.iter().find(|c| c.name == "Database").unwrap();
        assert_eq!(db_check.status, Status::Ok);
        // empty profile and no variants → warnings
        assert!(checks
            .iter()
            .any(|c| c.name == "Search variants" && c.status == Status::Warn));
    }
}
