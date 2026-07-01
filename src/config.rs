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
    /// Use the Claude in Chrome extension (real logged-in Chrome) via the `--chrome` flag.
    pub use_chrome: bool,
    /// Skip permission prompts in the headless agent (`--permission-mode bypassPermissions`).
    pub bypass_permissions: bool,
    /// Apply mode: "review" | "autonomous" | "hybrid".
    pub apply_mode: String,
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
            use_chrome: true,
            bypass_permissions: true,
            apply_mode: "review".to_string(),
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
        Ok(serde_json::from_str(&data)?)
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
    fn missing_file_uses_default() {
        let path = std::env::temp_dir().join("jobrabbit_nope_xyz/settings.json");
        assert!(Settings::load_from(&path).is_err());
    }
}
