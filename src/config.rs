//! Configuration and paths (XDG) for jobRabbit.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::locale::Locale;

/// App data directory (XDG data dir), e.g. ~/.local/share/jobrabbit
pub fn data_dir() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("dev", "jobrabbit", "jobrabbit")
        .context("failed to resolve the project directories")?;
    Ok(dirs.data_dir().to_path_buf())
}

/// Path to the SQLite database.
pub fn db_path() -> Result<PathBuf> {
    Ok(data_dir()?.join("jobrabbit.db"))
}

/// Path to the settings file (JSON).
pub fn settings_path() -> Result<PathBuf> {
    Ok(data_dir()?.join("settings.json"))
}

/// Takes the single-instance lock (`jobrabbit.lock` in the data dir). The
/// returned handle must be kept alive for the whole process — the OS advisory
/// lock is released when it drops, so a crash never leaves a stale lock.
pub fn acquire_instance_lock() -> Result<std::fs::File> {
    use fs4::fs_std::FileExt;

    let dir = data_dir()?;
    std::fs::create_dir_all(&dir).context("failed to create the data directory")?;
    let path = dir.join("jobrabbit.lock");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&path)
        .with_context(|| format!("failed to open the lock file at {}", path.display()))?;
    if !file
        .try_lock_exclusive()
        .with_context(|| format!("failed to lock {}", path.display()))?
    {
        anyhow::bail!(
            "another jobRabbit instance is already running (data dir: {}).\n\
             Close it first — two instances cannot share the same database.",
            dir.display()
        );
    }
    Ok(file)
}

/// Browser automation backend used by the agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BrowserBackend {
    /// Claude in Chrome extension (the user's real, logged-in Chrome) via `--chrome`.
    #[default]
    Chrome,
    /// Playwright MCP (Claude Code's playwright plugin) with a persistent profile —
    /// no extension required; the user logs in once in the Playwright browser.
    Playwright,
}

impl BrowserBackend {
    /// Stable identifier, matching the serialized form.
    pub fn as_str(self) -> &'static str {
        match self {
            BrowserBackend::Chrome => "chrome",
            BrowserBackend::Playwright => "playwright",
        }
    }

    /// The other backend (used by the TUI cycle toggle).
    pub fn next(self) -> Self {
        match self {
            BrowserBackend::Chrome => BrowserBackend::Playwright,
            BrowserBackend::Playwright => BrowserBackend::Chrome,
        }
    }
}

/// User settings, persisted in `settings.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Claude Code binary.
    pub claude_bin: String,
    /// Idle seconds before the agent is triggered automatically.
    pub idle_threshold_secs: u64,
    /// If `true`, the agent runs on its own when the user is idle.
    pub auto_run_on_idle: bool,
    /// Browser backend: Claude in Chrome extension or Playwright MCP (no extension).
    pub browser_backend: BrowserBackend,
    /// Skip permission prompts in the headless agent (`--permission-mode bypassPermissions`).
    pub bypass_permissions: bool,
    /// Apply mode: "review" | "autonomous" | "hybrid".
    pub apply_mode: String,
    /// Safety master: when `true` (default), the agent NEVER fills or submits a form on a job
    /// site without an explicit human review+approval of the generated content (CV/cover letter).
    /// Takes precedence over `apply_mode` (holds even in autonomous/hybrid). The user's own
    /// identity data (answer bank) is still filled without extra approval.
    pub require_human_review: bool,
    /// In "hybrid" mode, auto-apply only when fit >= this threshold; below it goes to review.
    pub hybrid_threshold: f64,
    /// Dry-run: simulates everything (search/evaluate/generate) but NEVER submits an application.
    pub dry_run: bool,
    /// If `true`, only consider jobs in the active locale's language; skip the rest.
    pub language_filter: bool,
    /// Work model filter: "remote" | "onsite" | "hybrid". Only search for jobs matching this model.
    pub work_model: String,
    /// UI / agent language. English by default.
    pub locale: Locale,
    /// Path to a résumé file (PDF/DOCX) to UPLOAD when a site requires it.
    /// Filled in automatically when a CV is imported from a file.
    pub cv_file_path: String,
    /// The candidate's LinkedIn URL (default for profile import).
    pub linkedin_url: String,
}

/// Supported apply modes.
pub const APPLY_MODES: [&str; 3] = ["review", "autonomous", "hybrid"];

impl Default for Settings {
    fn default() -> Self {
        Self {
            claude_bin: "claude".to_string(),
            idle_threshold_secs: 300,
            auto_run_on_idle: false,
            browser_backend: BrowserBackend::Chrome,
            bypass_permissions: true,
            apply_mode: "review".to_string(),
            require_human_review: true,
            hybrid_threshold: 0.9,
            dry_run: false,
            language_filter: false,
            work_model: "remote".to_string(),
            locale: Locale::En,
            cv_file_path: String::new(),
            linkedin_url: String::new(),
        }
    }

    // (new fields inherit these defaults via #[serde(default)] for older settings files)
}

impl Settings {
    /// Advance to the next apply mode (cycle).
    pub fn cycle_apply_mode(&mut self) {
        let i = APPLY_MODES
            .iter()
            .position(|m| *m == self.apply_mode)
            .unwrap_or(0);
        self.apply_mode = APPLY_MODES[(i + 1) % APPLY_MODES.len()].to_string();
    }

    /// Advance to the next UI / agent locale (cycle).
    pub fn cycle_locale(&mut self) {
        self.locale = self.locale.next();
    }

    /// Switch between browser backends (cycle).
    pub fn cycle_browser_backend(&mut self) {
        self.browser_backend = self.browser_backend.next();
    }
}

impl Settings {
    /// Load from the default path; returns `Default` if it doesn't exist / on error.
    pub fn load() -> Self {
        settings_path()
            .ok()
            .and_then(|p| Self::load_from(&p).ok())
            .unwrap_or_default()
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        let data = std::fs::read_to_string(path)?;
        let mut settings: Settings = serde_json::from_str(&data)?;
        // Migration: files written before `browser_backend` carried `use_chrome`.
        let raw: serde_json::Value = serde_json::from_str(&data)?;
        if raw.get("browser_backend").is_none()
            && raw.get("use_chrome").and_then(|v| v.as_bool()) == Some(false)
        {
            settings.browser_backend = BrowserBackend::Playwright;
        }
        Ok(settings)
    }

    /// Save to the default path (best-effort).
    pub fn save(&self) -> Result<()> {
        let path = settings_path()?;
        self.save_to(&path)
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_roundtrip() {
        let dir = std::env::temp_dir().join("jobrabbit_test_settings");
        let path = dir.join("settings.json");
        let _ = std::fs::remove_file(&path);

        let s = Settings {
            auto_run_on_idle: true,
            idle_threshold_secs: 120,
            ..Default::default()
        };
        s.save_to(&path).unwrap();

        let loaded = Settings::load_from(&path).unwrap();
        assert_eq!(loaded, s);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn legacy_use_chrome_false_migrates_to_playwright() {
        let dir = std::env::temp_dir().join("jobrabbit_test_settings_migration");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings-legacy-false.json");
        std::fs::write(&path, r#"{"use_chrome": false}"#).unwrap();
        let loaded = Settings::load_from(&path).unwrap();
        assert_eq!(loaded.browser_backend, BrowserBackend::Playwright);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn legacy_use_chrome_true_migrates_to_chrome() {
        let dir = std::env::temp_dir().join("jobrabbit_test_settings_migration");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings-legacy-true.json");
        std::fs::write(&path, r#"{"use_chrome": true}"#).unwrap();
        let loaded = Settings::load_from(&path).unwrap();
        assert_eq!(loaded.browser_backend, BrowserBackend::Chrome);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn browser_backend_takes_precedence_over_legacy_flag() {
        let dir = std::env::temp_dir().join("jobrabbit_test_settings_migration");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings-both.json");
        std::fs::write(
            &path,
            r#"{"use_chrome": false, "browser_backend": "chrome"}"#,
        )
        .unwrap();
        let loaded = Settings::load_from(&path).unwrap();
        assert_eq!(loaded.browser_backend, BrowserBackend::Chrome);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn missing_file_uses_default() {
        let path = std::env::temp_dir().join("jobrabbit_nope_xyz/settings.json");
        assert!(Settings::load_from(&path).is_err());
    }
}
