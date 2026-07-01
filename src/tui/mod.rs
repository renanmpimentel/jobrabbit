//! TUI layer (ratatui/crossterm): state, render loop, input, screens.

pub mod atstab;
#[path = "config.rs"]
pub mod configtab;
pub mod dashboard;
pub mod editor;
pub mod feedback;
pub mod pending;
pub mod profile;
pub mod sessionlog;

use anyhow::Result;
use std::time::Duration;

use crossterm::{
    event::{self, Event as CtEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame, Terminal,
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::config::Settings;
use crate::db::models::{Application, Feedback, PendingAction, SearchVariant, Stats};
use crate::db::Db;
use crate::event::AppEvent;
use crate::platform;
use editor::TextEditor;

/// TUI tabs (order matters: matches the 1..6 indices/shortcuts).
pub const TABS: [&str; 7] = [
    "Dashboard",
    "Profile",
    "Pending",
    "Feedback",
    "Session",
    "Config",
    "ATS",
];

/// Settings editable in the Config tab (list order).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingKind {
    LinkedinUrl,
    CvPath,
    ClaudeBin,
    ApplyMode,
    RequireHumanReview,
    LanguageFilter,
    Locale,
    HybridThreshold,
    DryRun,
    UseChrome,
    BypassPermissions,
    IdleThreshold,
    AutoRunIdle,
}

pub const SETTING_KINDS: [SettingKind; 13] = [
    SettingKind::LinkedinUrl,
    SettingKind::CvPath,
    SettingKind::ClaudeBin,
    SettingKind::ApplyMode,
    SettingKind::RequireHumanReview,
    SettingKind::LanguageFilter,
    SettingKind::Locale,
    SettingKind::HybridThreshold,
    SettingKind::DryRun,
    SettingKind::UseChrome,
    SettingKind::BypassPermissions,
    SettingKind::IdleThreshold,
    SettingKind::AutoRunIdle,
];

impl SettingKind {
    pub fn label(self) -> &'static str {
        match self {
            SettingKind::LinkedinUrl => "LinkedIn URL",
            SettingKind::CvPath => "CV path (upload)",
            SettingKind::ClaudeBin => "claude binary",
            SettingKind::ApplyMode => "Apply mode",
            SettingKind::RequireHumanReview => "Human review before filling",
            SettingKind::LanguageFilter => "Filter jobs by language",
            SettingKind::Locale => "Language",
            SettingKind::HybridThreshold => "Hybrid threshold (fit)",
            SettingKind::DryRun => "Dry-run (simulation)",
            SettingKind::UseChrome => "Use Claude in Chrome",
            SettingKind::BypassPermissions => "Skip permissions (autonomous)",
            SettingKind::IdleThreshold => "Idle after (seconds)",
            SettingKind::AutoRunIdle => "Auto-run when idle",
        }
    }

    pub fn is_bool(self) -> bool {
        matches!(
            self,
            SettingKind::DryRun
                | SettingKind::UseChrome
                | SettingKind::BypassPermissions
                | SettingKind::AutoRunIdle
                | SettingKind::LanguageFilter
                | SettingKind::RequireHumanReview
        )
    }

    /// `true` if changed by cycling/toggling in place (bools + enum settings),
    /// rather than free-text editing.
    pub fn is_cycle(self) -> bool {
        self.is_bool() || matches!(self, SettingKind::ApplyMode | SettingKind::Locale)
    }

    /// `true` if editable via text (text or number).
    pub fn is_text(self) -> bool {
        matches!(
            self,
            SettingKind::LinkedinUrl
                | SettingKind::CvPath
                | SettingKind::ClaudeBin
                | SettingKind::HybridThreshold
                | SettingKind::IdleThreshold
        )
    }

    pub fn value_str(self, s: &Settings) -> String {
        match self {
            SettingKind::LinkedinUrl => s.linkedin_url.clone(),
            SettingKind::CvPath => s.cv_file_path.clone(),
            SettingKind::ClaudeBin => s.claude_bin.clone(),
            SettingKind::ApplyMode => s.apply_mode.clone(),
            SettingKind::RequireHumanReview => bool_str(s.require_human_review),
            SettingKind::LanguageFilter => bool_str(s.language_filter),
            SettingKind::Locale => s.locale.label().to_string(),
            SettingKind::HybridThreshold => format!("{:.2}", s.hybrid_threshold),
            SettingKind::DryRun => bool_str(s.dry_run),
            SettingKind::UseChrome => bool_str(s.use_chrome),
            SettingKind::BypassPermissions => bool_str(s.bypass_permissions),
            SettingKind::IdleThreshold => s.idle_threshold_secs.to_string(),
            SettingKind::AutoRunIdle => bool_str(s.auto_run_on_idle),
        }
    }

    fn toggle(self, s: &mut Settings) {
        match self {
            SettingKind::DryRun => s.dry_run = !s.dry_run,
            SettingKind::UseChrome => s.use_chrome = !s.use_chrome,
            SettingKind::BypassPermissions => s.bypass_permissions = !s.bypass_permissions,
            SettingKind::AutoRunIdle => s.auto_run_on_idle = !s.auto_run_on_idle,
            SettingKind::LanguageFilter => s.language_filter = !s.language_filter,
            SettingKind::RequireHumanReview => s.require_human_review = !s.require_human_review,
            SettingKind::Locale => s.cycle_locale(),
            SettingKind::ApplyMode => s.cycle_apply_mode(),
            _ => {}
        }
    }

    fn apply_text(self, s: &mut Settings, txt: &str) {
        let txt = txt.trim();
        match self {
            SettingKind::LinkedinUrl => s.linkedin_url = txt.to_string(),
            SettingKind::CvPath => s.cv_file_path = txt.to_string(),
            SettingKind::ClaudeBin => {
                if !txt.is_empty() {
                    s.claude_bin = txt.to_string();
                }
            }
            SettingKind::HybridThreshold => {
                if let Ok(v) = txt.parse::<f64>() {
                    s.hybrid_threshold = v.clamp(0.0, 1.0);
                }
            }
            SettingKind::IdleThreshold => {
                if let Ok(v) = txt.parse::<u64>() {
                    s.idle_threshold_secs = v;
                }
            }
            _ => {}
        }
    }
}

fn bool_str(b: bool) -> String {
    if b {
        "yes".to_string()
    } else {
        "no".to_string()
    }
}

/// Edit target in the Config tab.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigTarget {
    Setting(SettingKind),
    Answer { key: String, label: String },
}

pub use crate::core::AgentStatus;

impl AgentStatus {
    fn color(&self) -> Color {
        match self {
            AgentStatus::Idle => Color::Green,
            AgentStatus::Running => Color::Yellow,
            AgentStatus::Error(_) => Color::Red,
        }
    }
}

/// Field currently being edited (Profile tab).
#[derive(Debug, Clone, PartialEq)]
pub enum EditField {
    Background,
    Cv,
    NewVariantLabel,
    NewVariantQuery,
    ImportPath,
    ImportLinkedin,
    AnswerInput,
    ConfigInput,
    AtsTarget,
}

/// Application state.
pub struct App {
    pub should_quit: bool,
    pub active: usize,
    pub agent_status: AgentStatus,

    // Editing (Profile tab).
    pub editing: Option<EditField>,
    pub editor: TextEditor,
    pub variant_sel: usize,
    pub new_variant_label: String,

    // Agent / pending actions.
    pub current_session_id: Option<i64>,
    pub pending_sel: usize,
    pub settings: Settings,
    /// Answer being edited: (pending_id, field_key, label).
    pub answer_input: Option<(i64, String, String)>,

    // Config tab.
    pub config_sel: usize,
    pub config_edit: Option<ConfigTarget>,
    pub answers: Vec<crate::db::models::Answer>,

    // ATS tab (CV evaluation).
    pub jobs: Vec<crate::db::models::Job>,
    pub cv_review: Option<crate::db::models::CvReview>,
    pub ats_job_sel: usize,

    // Cached data (updated from DB).
    pub stats: Stats,
    pub variants: Vec<SearchVariant>,
    pub pending: Vec<PendingAction>,
    pub applications: Vec<Application>,
    pub feedback: Vec<Feedback>,
    pub profile_background: String,
    pub profile_cv: String,

    // Session log.
    pub log: Vec<String>,
    pub log_scroll: u16,
    pub log_follow: bool,

    // Dashboard: applications per day (last 7 days, ascending).
    pub apps_per_day: Vec<(String, u64)>,
}

impl App {
    pub fn new(db: &Db) -> Result<Self> {
        let mut app = Self {
            should_quit: false,
            active: 0,
            agent_status: AgentStatus::Idle,
            editing: None,
            editor: TextEditor::default(),
            variant_sel: 0,
            new_variant_label: String::new(),
            current_session_id: None,
            pending_sel: 0,
            settings: Settings::load(),
            answer_input: None,
            config_sel: 0,
            config_edit: None,
            answers: Vec::new(),
            jobs: Vec::new(),
            cv_review: None,
            ats_job_sel: 0,
            stats: Stats::default(),
            variants: Vec::new(),
            pending: Vec::new(),
            applications: Vec::new(),
            feedback: Vec::new(),
            profile_background: String::new(),
            profile_cv: String::new(),
            log: Vec::new(),
            log_scroll: 0,
            log_follow: true,
            apps_per_day: Vec::new(),
        };
        app.refresh(db)?;
        app.push_log("jobRabbit ready. Session tab shows the agent in real time.".into());
        app.push_log(
            "⚖ Check the Terms of Service of each job site. Usage is your responsibility.".into(),
        );
        if !platform::claude_available(&app.settings.claude_bin) {
            app.push_log(format!(
                "✖ `{}` not found in PATH — install/authenticate Claude Code to run the agent.",
                app.settings.claude_bin
            ));
        } else {
            app.push_log("✔ claude found. Set up Profile (tab 2) and press [r] to run.".into());
        }
        Ok(app)
    }

    /// Checks if `claude` is available; logs and notifies if missing.
    fn ensure_claude(&mut self) -> bool {
        if platform::claude_available(&self.settings.claude_bin) {
            return true;
        }
        let msg = format!(
            "`{}` not found in PATH. Install Claude Code and authenticate.",
            self.settings.claude_bin
        );
        self.push_log(format!("✖ {msg}"));
        self.agent_status = AgentStatus::Error("claude missing".into());
        false
    }

    /// Reloads data from DB.
    pub fn refresh(&mut self, db: &Db) -> Result<()> {
        self.stats = db.stats()?;
        self.variants = db.list_variants()?;
        self.pending = db.list_pending(false)?;
        self.applications = db.list_applications()?;
        self.feedback = db.list_feedback()?;
        self.answers = db.get_answers()?;
        self.jobs = db.list_jobs()?;
        self.cv_review = db.latest_cv_review()?;
        let p = db.get_profile()?;
        self.profile_background = p.background;
        self.profile_cv = p.cv_base;
        self.apps_per_day = Self::load_apps_per_day(db);
        Ok(())
    }

    /// Series of applications from the last 7 days (zero-filled), ascending.
    fn load_apps_per_day(db: &Db) -> Vec<(String, u64)> {
        let map = db.applications_per_day().unwrap_or_default();
        let today = chrono::Utc::now().date_naive();
        (0..7)
            .rev()
            .map(|i| {
                let d = today - chrono::Duration::days(i);
                let key = d.format("%Y-%m-%d").to_string();
                let label = d.format("%d/%m").to_string();
                (label, *map.get(&key).unwrap_or(&0) as u64)
            })
            .collect()
    }

    fn push_log(&mut self, line: String) {
        // limits buffer to prevent unbounded growth
        const MAX: usize = 5000;
        for l in line.split('\n') {
            self.log.push(l.to_string());
        }
        if self.log.len() > MAX {
            let drop = self.log.len() - MAX;
            self.log.drain(0..drop);
        }
    }

    fn next_tab(&mut self) {
        self.active = (self.active + 1) % TABS.len();
    }

    fn prev_tab(&mut self) {
        self.active = (self.active + TABS.len() - 1) % TABS.len();
    }

    /// Handles a key press. `db` is used to persist edits.
    pub fn on_key(&mut self, key: KeyEvent, tx: &UnboundedSender<AppEvent>, db: &Db) {
        // In editing mode, all keys go to the editor.
        if self.editing.is_some() {
            self.on_key_editing(key, tx, db);
            return;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Tab | KeyCode::Right => self.next_tab(),
            KeyCode::BackTab | KeyCode::Left => self.prev_tab(),
            KeyCode::Char(c @ '1'..='7') => {
                self.active = (c as usize) - ('1' as usize);
            }
            KeyCode::Char('r') => self.trigger_run(tx, db),
            KeyCode::Char('p') => {
                self.settings.cycle_apply_mode();
                let _ = self.settings.save();
                self.push_log(format!("⚙ application mode: {}", self.settings.apply_mode));
            }
            KeyCode::Char('y') => {
                self.settings.dry_run = !self.settings.dry_run;
                let _ = self.settings.save();
                self.push_log(format!(
                    "🧪 dry-run (simulation): {}",
                    if self.settings.dry_run { "on" } else { "off" }
                ));
            }
            KeyCode::Char('i') => {
                self.settings.auto_run_on_idle = !self.settings.auto_run_on_idle;
                let _ = self.settings.save();
                self.push_log(format!(
                    "⏾ auto-run when idle: {}",
                    if self.settings.auto_run_on_idle {
                        "on"
                    } else {
                        "off"
                    }
                ));
            }
            KeyCode::Char('d') => {
                // Demo: injects events into channel to validate UI flow
                // without running `claude` for real.
                let _ = tx.send(AppEvent::AgentStarted);
                let _ = tx.send(AppEvent::AgentText(
                    "Analyzing job: Senior Rust Engineer @ Acme (fit 0.87)".into(),
                ));
                let _ = tx.send(AppEvent::AgentToolUse {
                    name: "Bash".into(),
                    input: serde_json::json!({"command": "open job in Chrome"}),
                });
                let _ = tx.send(AppEvent::AgentFinished {
                    result: Some("Application submitted (demo).".into()),
                    num_turns: Some(3),
                    cost_usd: Some(0.12),
                    is_error: false,
                });
            }
            // Keys specific to active tab.
            _ => match self.active {
                1 => self.on_key_profile(key, db),
                2 => self.on_key_pending(key, tx, db),
                3 => self.on_key_feedback(key, tx, db),
                4 => self.on_key_sessionlog(key),
                5 => self.on_key_config(key, db),
                6 => self.on_key_ats(key, tx, db),
                _ => {}
            },
        }
    }

    /// Keys for Pending tab (navigate, open URL, resolve).
    fn on_key_pending(&mut self, key: KeyEvent, tx: &UnboundedSender<AppEvent>, db: &Db) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.pending_sel = self.pending_sel.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.pending.is_empty() {
                    self.pending_sel = (self.pending_sel + 1).min(self.pending.len() - 1);
                }
            }
            KeyCode::Enter | KeyCode::Char('o') => {
                if let Some(p) = self.pending.get(self.pending_sel) {
                    if let Some(url) = &p.url {
                        platform::open_url(url);
                        self.push_log(format!("↗ opening {url}"));
                    }
                }
            }
            KeyCode::Char('a') => {
                // Contextual action: approve application OR answer question.
                match self.pending.get(self.pending_sel).map(|p| p.kind.clone()) {
                    Some(k) if k == "answer_needed" => self.start_answer_input(),
                    Some(k) if k == "approval" => self.approve_selected_pending(tx, db),
                    _ => self.push_log(
                        "ℹ nothing to approve/answer here (use [o] open, [space] resolve)".into(),
                    ),
                }
            }
            KeyCode::Char(' ') => {
                if let Some(p) = self.pending.get(self.pending_sel) {
                    let _ = db.resolve_pending(p.id);
                    let _ = self.refresh(db);
                    if self.pending_sel >= self.pending.len() {
                        self.pending_sel = self.pending.len().saturating_sub(1);
                    }
                }
            }
            _ => {}
        }
    }

    /// Resolves the approval item for a job (when application is confirmed).
    fn resolve_approval_for(&self, db: &Db, job_id: i64) {
        if let Ok(list) = db.list_pending(false) {
            for p in list
                .iter()
                .filter(|p| p.kind == "approval" && p.job_id == Some(job_id))
            {
                let _ = db.resolve_pending(p.id);
            }
        }
    }

    /// Opens the editor to answer the selected screening question (`answer_needed`).
    fn start_answer_input(&mut self) {
        let Some(p) = self.pending.get(self.pending_sel).cloned() else {
            return;
        };
        let field_key = p.field_key.clone().unwrap_or_else(|| "answer".to_string());
        self.editor = TextEditor::new("");
        self.answer_input = Some((p.id, field_key, p.description.clone()));
        self.editing = Some(EditField::AnswerInput);
    }

    /// Approves the selected item (if `approval`): triggers the real submission
    /// of the application using the already-prepared CV/cover letter.
    fn approve_selected_pending(&mut self, tx: &UnboundedSender<AppEvent>, db: &Db) {
        let Some(p) = self.pending.get(self.pending_sel).cloned() else {
            return;
        };
        // Assembles the submission prompt (ATS-aware) in core; validates type/job/application.
        let prompt = match crate::core::actions::approval_prompt(db, &self.settings, &p) {
            Ok(prompt) => prompt,
            Err(e) => {
                self.push_log(format!("ℹ {e}"));
                return;
            }
        };
        if let Some(job) = p.job_id.and_then(|id| db.get_job(id).ok().flatten()) {
            self.push_log(format!(
                "✅ approved — submitting for real: {} @ {}",
                job.title, job.company
            ));
            self.push_log(format!(
                "🧭 platform: {} — following playbook",
                crate::ats::detect(&job.url).name()
            ));
        }
        if self.settings.cv_file_path.is_empty() {
            self.push_log(
                "ℹ no CV file for upload — set by importing a CV from file (--import-cv) if the site requires."
                    .into(),
            );
        }
        // DO NOT resolve the item here: it only goes away when the application is CONFIRMED
        // (applied event). On block/failure, the item stays for retry.
        self.spawn_session(prompt, "end of application attempt", tx);
    }

    /// Triggers an agent run for all active variants.
    fn trigger_run(&mut self, tx: &UnboundedSender<AppEvent>, db: &Db) {
        if self.agent_status == AgentStatus::Running {
            self.push_log("⚠ agent is already running".into());
            return;
        }
        let prompts = match crate::core::actions::search_prompts(db, &self.settings) {
            Ok(p) => p,
            Err(e) => {
                self.push_log(format!("✖ {e}"));
                return;
            }
        };
        if !self.ensure_claude() {
            return;
        }
        self.active = 4; // shows the Session tab
        let dry = self.settings.dry_run;
        self.push_log(format!(
            "▶ mode: {}{}",
            self.settings.apply_mode,
            if dry { " + dry-run (simulation)" } else { "" }
        ));
        let n = prompts.len();
        crate::core::actions::spawn_run(
            crate::core::actions::agent_config(&self.settings),
            prompts,
            format!("{n} variant(s) processed"),
            tx.clone(),
        );
    }

    /// Keys for Session tab (log scroll).
    fn on_key_sessionlog(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                self.log_follow = false;
                self.log_scroll = self.log_scroll.saturating_sub(1);
            }
            KeyCode::Down => {
                self.log_follow = false;
                self.log_scroll = self.log_scroll.saturating_add(1);
            }
            KeyCode::Char('f') => self.log_follow = !self.log_follow,
            _ => {}
        }
    }

    /// Keys for Profile tab (navigation/editing of variants and fields).
    fn on_key_profile(&mut self, key: KeyEvent, db: &Db) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.variant_sel = self.variant_sel.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.variants.is_empty() {
                    self.variant_sel = (self.variant_sel + 1).min(self.variants.len() - 1);
                }
            }
            KeyCode::Char('e') => {
                self.editor = TextEditor::new(&self.profile_background);
                self.editing = Some(EditField::Background);
            }
            KeyCode::Char('c') => {
                self.editor = TextEditor::new(&self.profile_cv);
                self.editing = Some(EditField::Cv);
            }
            KeyCode::Char('a') => {
                self.new_variant_label.clear();
                self.editor = TextEditor::new("");
                self.editing = Some(EditField::NewVariantLabel);
            }
            KeyCode::Char('m') => {
                let prefill = if self.settings.cv_file_path.is_empty() {
                    String::new()
                } else {
                    self.settings.cv_file_path.clone()
                };
                self.editor = TextEditor::new(&prefill);
                self.editing = Some(EditField::ImportPath);
            }
            KeyCode::Char('l') => {
                let prefill = if self.settings.linkedin_url.is_empty() {
                    "https://www.linkedin.com/in/".to_string()
                } else {
                    self.settings.linkedin_url.clone()
                };
                self.editor = TextEditor::new(&prefill);
                self.editing = Some(EditField::ImportLinkedin);
            }
            KeyCode::Char(' ') => {
                if let Some(v) = self.variants.get(self.variant_sel) {
                    let _ = db.set_variant_enabled(v.id, !v.enabled);
                    let _ = self.refresh(db);
                }
            }
            KeyCode::Char('x') => {
                if let Some(v) = self.variants.get(self.variant_sel) {
                    let _ = db.delete_variant(v.id);
                    let _ = self.refresh(db);
                    if self.variant_sel > 0 && self.variant_sel >= self.variants.len() {
                        self.variant_sel = self.variants.len().saturating_sub(1);
                    }
                }
            }
            _ => {}
        }
    }

    /// Keys in editing mode (routed to [`TextEditor`]).
    fn on_key_editing(&mut self, key: KeyEvent, tx: &UnboundedSender<AppEvent>, db: &Db) {
        // Esc cancels any edit.
        if key.code == KeyCode::Esc {
            self.editing = None;
            self.answer_input = None;
            self.config_edit = None;
            return;
        }
        let field = self.editing.clone().unwrap();
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            // Save multiline fields with Ctrl+S.
            KeyCode::Char('s') if ctrl => match field {
                EditField::Background => {
                    let _ = db.save_profile(self.editor.content(), &self.profile_cv);
                    let _ = self.refresh(db);
                    self.editing = None;
                }
                EditField::Cv => {
                    let _ = db.save_profile(&self.profile_background, self.editor.content());
                    let _ = self.refresh(db);
                    self.editing = None;
                }
                _ => {}
            },
            KeyCode::Enter => match field {
                // Multiline: Enter inserts new line.
                EditField::Background | EditField::Cv => self.editor.newline(),
                // Variant: Enter advances/saves.
                EditField::NewVariantLabel => {
                    self.new_variant_label = self.editor.content().to_string();
                    self.editor = TextEditor::new("");
                    self.editing = Some(EditField::NewVariantQuery);
                }
                EditField::NewVariantQuery => {
                    let label = self.new_variant_label.trim();
                    let query = self.editor.content().trim();
                    if !label.is_empty() && !query.is_empty() {
                        let _ = db.add_variant(label, query);
                        let _ = self.refresh(db);
                    }
                    self.editing = None;
                }
                EditField::ImportPath => {
                    let path = self.editor.content().trim().to_string();
                    self.editing = None;
                    if !path.is_empty() {
                        self.import_cv_file(&path, tx, db);
                    }
                }
                EditField::ImportLinkedin => {
                    let url = self.editor.content().trim().to_string();
                    self.editing = None;
                    if !url.is_empty() {
                        self.import_linkedin(&url, tx, db);
                    }
                }
                EditField::AtsTarget => {
                    let target = self.editor.content().trim().to_string();
                    self.editing = None;
                    if !target.is_empty() {
                        self.trigger_cv_review(Some(target), tx, db);
                    }
                }
                EditField::ConfigInput => {
                    let value = self.editor.content().to_string();
                    match self.config_edit.take() {
                        Some(ConfigTarget::Setting(kind)) => {
                            kind.apply_text(&mut self.settings, &value);
                            let _ = self.settings.save();
                        }
                        Some(ConfigTarget::Answer { key, label }) => {
                            let _ = db.set_answer(&key, &label, value.trim());
                            let _ = self.refresh(db);
                        }
                        None => {}
                    }
                    self.editing = None;
                }
                EditField::AnswerInput => {
                    let value = self.editor.content().trim().to_string();
                    if let Some((pending_id, field_key, label)) = self.answer_input.take() {
                        if !value.is_empty() {
                            let _ = db.set_answer(&field_key, &label, &value);
                            let _ = db.resolve_pending(pending_id);
                            self.push_log(format!("✔ answer saved: {field_key} = {value}"));
                            let _ = self.refresh(db);
                            if self.pending_sel >= self.pending.len() {
                                self.pending_sel = self.pending.len().saturating_sub(1);
                            }
                        }
                    }
                    self.editing = None;
                }
            },
            KeyCode::Backspace => self.editor.backspace(),
            KeyCode::Left => self.editor.left(),
            KeyCode::Right => self.editor.right(),
            KeyCode::Home => self.editor.home(),
            KeyCode::End => self.editor.end(),
            KeyCode::Char(c) if !ctrl => self.editor.insert(c),
            _ => {}
        }
    }

    /// Total of actionable items in Config tab (settings + answers).
    fn config_len(&self) -> usize {
        SETTING_KINDS.len() + self.answers.len()
    }

    /// Keys for Config tab (navigate/edit settings and answers).
    fn on_key_config(&mut self, key: KeyEvent, _db: &Db) {
        let n = self.config_len();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.config_sel = self.config_sel.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if n > 0 {
                    self.config_sel = (self.config_sel + 1).min(n - 1);
                }
            }
            KeyCode::Char(' ') => {
                // Direct toggle for bool / enum settings.
                if let Some(kind) = self.selected_setting() {
                    if kind.is_cycle() {
                        kind.toggle(&mut self.settings);
                        let _ = self.settings.save();
                    }
                }
            }
            KeyCode::Enter | KeyCode::Char('e') => self.start_config_edit(),
            _ => {}
        }
    }

    /// Setting at the selected position (None if the selection is in the answer bank).
    fn selected_setting(&self) -> Option<SettingKind> {
        SETTING_KINDS.get(self.config_sel).copied()
    }

    /// Starts editing the selected Config item.
    fn start_config_edit(&mut self) {
        if let Some(kind) = self.selected_setting() {
            if kind.is_cycle() {
                kind.toggle(&mut self.settings);
                let _ = self.settings.save();
            } else {
                self.editor = TextEditor::new(&kind.value_str(&self.settings));
                self.config_edit = Some(ConfigTarget::Setting(kind));
                self.editing = Some(EditField::ConfigInput);
            }
            return;
        }
        // Otherwise, it's an answer from the bank.
        let idx = self.config_sel - SETTING_KINDS.len();
        if let Some(ans) = self.answers.get(idx) {
            self.editor = TextEditor::new(&ans.value);
            self.config_edit = Some(ConfigTarget::Answer {
                key: ans.key.clone(),
                label: ans.label.clone(),
            });
            self.editing = Some(EditField::ConfigInput);
        }
    }

    /// Keys for ATS tab (CV evaluation).
    fn on_key_ats(&mut self, key: KeyEvent, tx: &UnboundedSender<AppEvent>, db: &Db) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.ats_job_sel = self.ats_job_sel.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.jobs.is_empty() {
                    self.ats_job_sel = (self.ats_job_sel + 1).min(self.jobs.len() - 1);
                }
            }
            KeyCode::Char('g') => self.trigger_cv_review(None, tx, db),
            KeyCode::Enter => {
                if let Some(job) = self.jobs.get(self.ats_job_sel) {
                    let target = format!("{} @ {}\n{}", job.title, job.company, job.description);
                    self.trigger_cv_review(Some(target), tx, db);
                } else {
                    self.push_log(
                        "ℹ no jobs — use [g] general evaluation or [t] paste description".into(),
                    );
                }
            }
            KeyCode::Char('t') => {
                self.editor = TextEditor::new("");
                self.editing = Some(EditField::AtsTarget);
            }
            _ => {}
        }
    }

    /// CV text to evaluate: file (if present) else `cv_base` from profile.
    /// Triggers ATS evaluation of CV (general or against a target).
    fn trigger_cv_review(
        &mut self,
        target: Option<String>,
        tx: &UnboundedSender<AppEvent>,
        db: &Db,
    ) {
        let prompt =
            match crate::core::actions::cv_review_prompt(db, &self.settings, target.as_deref()) {
                Ok(p) => p,
                Err(e) => {
                    self.push_log(format!("✖ {e}"));
                    return;
                }
            };
        self.push_log(format!(
            "📋 evaluating CV ({})",
            if target.is_some() {
                "against target job"
            } else {
                "general"
            }
        ));
        self.spawn_session(prompt, "CV evaluation completed", tx);
    }

    /// Keys for Feedback tab (generate analysis).
    fn on_key_feedback(&mut self, key: KeyEvent, tx: &UnboundedSender<AppEvent>, db: &Db) {
        if key.code == KeyCode::Char('g') {
            self.trigger_feedback(tx, db);
        }
    }

    /// Triggers ONE agent session with the given `prompt`, monitoring in Session tab.
    ///
    /// Does claude preflight, sends `AgentStarted`, spawns `run_session` and emits
    /// `AgentFinished` at end. Base for feedback and imports.
    fn spawn_session(&mut self, prompt: String, done_msg: &str, tx: &UnboundedSender<AppEvent>) {
        if self.agent_status == AgentStatus::Running {
            self.push_log("⚠ agent is already running".into());
            return;
        }
        if !self.ensure_claude() {
            return;
        }
        self.active = 4; // Session tab (to monitor)
        crate::core::actions::spawn_run(
            crate::core::actions::agent_config(&self.settings),
            vec![prompt],
            done_msg.to_string(),
            tx.clone(),
        );
    }

    /// Triggers an agent session to generate feedback.
    fn trigger_feedback(&mut self, tx: &UnboundedSender<AppEvent>, db: &Db) {
        let prompt = crate::core::actions::feedback_prompt(db, self.settings.locale);
        self.spawn_session(prompt, "feedback analysis generated", tx);
    }

    /// Imports profile from a CV file (PDF/DOCX/TXT).
    fn import_cv_file(&mut self, path: &str, tx: &UnboundedSender<AppEvent>, db: &Db) {
        let _ = db; // persistence happens via AgentProfile
        match crate::core::actions::import_cv_prompt(path, self.settings.locale) {
            Ok((prompt, text)) => {
                self.push_log(format!(
                    "📄 importing CV from {path} ({} chars)",
                    text.len()
                ));
                // Saves the file for upload in applications that require it.
                self.settings.cv_file_path = path.to_string();
                let _ = self.settings.save();
                self.spawn_session(prompt, "profile imported from CV", tx);
            }
            Err(e) => {
                self.push_log(format!("✖ {e}"));
            }
        }
    }

    /// Imports profile by navigating LinkedIn (via Chrome).
    fn import_linkedin(&mut self, url: &str, tx: &UnboundedSender<AppEvent>, _db: &Db) {
        self.push_log(format!("🔗 importing profile from LinkedIn: {url}"));
        // Saves as default for next imports.
        self.settings.linkedin_url = url.to_string();
        let _ = self.settings.save();
        let prompt = crate::core::actions::import_linkedin_prompt(url, self.settings.locale);
        self.spawn_session(prompt, "profile imported from LinkedIn", tx);
    }

    /// Applies an [`AppEvent`] from core. UI owns the DB: all
    /// persistence derived from agent execution happens here.
    pub fn on_app_event(&mut self, ev: AppEvent, db: &Db) {
        if let AppEvent::Quit = ev {
            self.should_quit = true;
            return;
        }
        // All persistence derived from agent lives in `core::persist`.
        let out = crate::core::persist::apply_event(db, &ev, &mut self.current_session_id);
        for line in out.logs {
            self.push_log(line);
        }
        if let Some(tab) = out.focus_tab {
            self.active = tab;
        }
        if let Some(status) = out.status {
            self.agent_status = status;
        }
        if let Some((kind, desc)) = out.notify {
            platform::notify::pending(&kind, &desc);
        }
        if out.refresh {
            let _ = self.refresh(db);
        }
    }

    /// Periodic update (stats are cheap in local SQLite).
    fn on_tick(&mut self, db: &Db) {
        if let Ok(s) = db.stats() {
            self.stats = s;
        }
    }
}

/// Main render: tabs at top, content in middle, status bar at bottom.
pub fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    // Tabs (shortcuts 1..7 map to indices).
    let titles: Vec<Line> = TABS
        .iter()
        .enumerate()
        .map(|(i, t)| Line::from(format!(" {} {} ", i + 1, t)))
        .collect();
    let tabs = Tabs::new(titles)
        .select(app.active)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" jobRabbit 🐇 "),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, chunks[0]);

    // Active tab content (each module draws its own edit overlays).
    match app.active {
        0 => dashboard::render(f, chunks[1], app),
        1 => profile::render(f, chunks[1], app),
        2 => pending::render(f, chunks[1], app),
        3 => feedback::render(f, chunks[1], app),
        4 => sessionlog::render(f, chunks[1], app),
        5 => configtab::render(f, chunks[1], app),
        6 => atstab::render(f, chunks[1], app),
        _ => {}
    }

    // Status bar: agent status + global shortcuts.
    let hint = "[Tab] tab  [1-7] go  [r] run  [p] mode  [y] dry-run  [i] auto-idle  [q] quit";
    let status = Line::from(vec![
        Span::styled(
            format!(" {} ", app.agent_status.label()),
            Style::default().fg(app.agent_status.color()),
        ),
        Span::raw("  "),
        Span::styled(hint, Style::default().fg(Color::DarkGray)),
    ]);
    f.render_widget(Paragraph::new(status), chunks[2]);
}

/// Renders current state as plain text (TTY-less preview / tests).
pub fn snapshot(app: &App, width: u16, height: u16) -> String {
    let backend = ratatui::backend::TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("test backend");
    terminal.draw(|f| ui(f, app)).expect("draw snapshot");
    // `TestBackend` implements `Display` (renders buffer with line breaks).
    format!("{}", terminal.backend())
}

/// Main TUI loop: sets up terminal, reads input in dedicated thread and
/// multiplexes input + agent events + render tick via `tokio::select!`.
pub async fn run(
    db: Db,
    tx: UnboundedSender<AppEvent>,
    mut rx: UnboundedReceiver<AppEvent>,
) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(&db)?;

    // Input in blocking thread: reads keys from crossterm and sends via channel.
    let (input_tx, mut input_rx) = unbounded_channel::<KeyEvent>();
    std::thread::spawn(move || loop {
        match event::poll(Duration::from_millis(100)) {
            Ok(true) => {
                if let Ok(CtEvent::Key(k)) = event::read() {
                    if k.kind == KeyEventKind::Press && input_tx.send(k).is_err() {
                        break;
                    }
                }
            }
            Ok(false) => {}
            Err(_) => break,
        }
    });

    // Watches for idleness (emits `IdleReached` on same event channel).
    let threshold = Duration::from_secs(app.settings.idle_threshold_secs);
    tokio::spawn(platform::idle::watch(threshold, tx.clone()));

    let mut tick = tokio::time::interval(Duration::from_millis(250));
    terminal.draw(|f| ui(f, &app))?;

    while !app.should_quit {
        tokio::select! {
            Some(k) = input_rx.recv() => {
                app.on_key(k, &tx, &db);
            }
            Some(ev) = rx.recv() => {
                match ev {
                    AppEvent::IdleReached => {
                        if app.settings.auto_run_on_idle
                            && app.agent_status != AgentStatus::Running
                        {
                            app.push_log("⏾ idle — starting auto-run".into());
                            app.trigger_run(&tx, &db);
                        }
                    }
                    other => app.on_app_event(other, &db),
                }
            }
            _ = tick.tick() => {
                app.on_tick(&db);
            }
        }
        terminal.draw(|f| ui(f, &app))?;
    }

    // Restore terminal.
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::NewJob;

    fn key(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
    }
    fn key_code(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn navigation_by_tabs() {
        let db = Db::open_in_memory().unwrap();
        let mut app = App::new(&db).unwrap();
        let (tx, _rx) = unbounded_channel::<AppEvent>();
        assert_eq!(app.active, 0);
        app.on_key(key_code(KeyCode::Tab), &tx, &db);
        assert_eq!(app.active, 1);
        app.on_key(key_code(KeyCode::BackTab), &tx, &db);
        assert_eq!(app.active, 0);
        app.on_key(key('5'), &tx, &db);
        assert_eq!(app.active, 4);
    }

    #[test]
    fn editing_does_not_switch_tab() {
        let db = Db::open_in_memory().unwrap();
        let mut app = App::new(&db).unwrap();
        let (tx, _rx) = unbounded_channel::<AppEvent>();
        app.active = 1;
        app.editing = Some(EditField::Background);
        app.editor = TextEditor::new("");
        // Tab during editing should NOT switch tabs (goes to editor).
        app.on_key(key_code(KeyCode::Tab), &tx, &db);
        assert_eq!(app.active, 1);
    }

    #[test]
    fn config_edits_text_setting() {
        let db = Db::open_in_memory().unwrap();
        let mut app = App::new(&db).unwrap();
        let (tx, _rx) = unbounded_channel::<AppEvent>();
        app.active = 5;
        app.config_sel = 0; // LinkedinUrl
        app.on_key(key_code(KeyCode::Enter), &tx, &db);
        assert_eq!(app.editing, Some(EditField::ConfigInput));
        for c in "http://li/x".chars() {
            app.on_key(key(c), &tx, &db);
        }
        app.on_key(key_code(KeyCode::Enter), &tx, &db);
        assert_eq!(app.editing, None);
        assert_eq!(app.settings.linkedin_url, "http://li/x");
    }

    #[test]
    fn config_toggles_bool_with_space() {
        let db = Db::open_in_memory().unwrap();
        let mut app = App::new(&db).unwrap();
        let (tx, _rx) = unbounded_channel::<AppEvent>();
        app.active = 5;
        // DryRun is index 8 in SETTING_KINDS (after ApplyMode=3, RequireHumanReview=4,
        // LanguageFilter=5, Locale=6, HybridThreshold=7).
        app.config_sel = 8;
        let before = app.settings.dry_run;
        app.on_key(key(' '), &tx, &db);
        assert_eq!(app.settings.dry_run, !before);
    }

    #[test]
    fn config_edits_answer_bank() {
        let db = Db::open_in_memory().unwrap();
        let mut app = App::new(&db).unwrap();
        let (tx, _rx) = unbounded_channel::<AppEvent>();
        app.active = 5;
        app.config_sel = SETTING_KINDS.len(); // first answer after settings
        let key0 = app.answers[0].key.clone();
        app.on_key(key_code(KeyCode::Enter), &tx, &db);
        assert_eq!(app.editing, Some(EditField::ConfigInput));
        for c in "valor-x".chars() {
            app.on_key(key(c), &tx, &db);
        }
        app.on_key(key_code(KeyCode::Enter), &tx, &db);
        assert_eq!(
            db.answers_map().unwrap().get(&key0).map(|s| s.as_str()),
            Some("valor-x")
        );
    }

    #[test]
    fn toggle_and_remove_variant() {
        let db = Db::open_in_memory().unwrap();
        db.add_variant("Senior Rust", "senior rust remoto").unwrap();
        let mut app = App::new(&db).unwrap();
        let (tx, _rx) = unbounded_channel::<AppEvent>();
        app.active = 1;
        app.variant_sel = 0;
        assert!(db.list_variants().unwrap()[0].enabled);
        app.on_key(key(' '), &tx, &db); // toggle off
        assert!(!db.list_variants().unwrap()[0].enabled);
        app.on_key(key('x'), &tx, &db); // delete
        assert!(db.list_variants().unwrap().is_empty());
    }

    #[test]
    fn resolve_pending_in_tab() {
        let db = Db::open_in_memory().unwrap();
        db.add_pending(None, "captcha", "x", None).unwrap();
        let mut app = App::new(&db).unwrap();
        let (tx, _rx) = unbounded_channel::<AppEvent>();
        app.active = 2;
        app.pending_sel = 0;
        assert_eq!(app.pending.len(), 1);
        app.on_key(key(' '), &tx, &db); // resolve
        assert!(db.list_pending(false).unwrap().is_empty());
    }

    #[test]
    fn agent_pipeline_persists_to_db() {
        let db = Db::open_in_memory().unwrap();
        let mut app = App::new(&db).unwrap();

        app.on_app_event(AppEvent::AgentStarted, &db);
        assert_eq!(app.agent_status, AgentStatus::Running);
        assert!(app.current_session_id.is_some());

        app.on_app_event(AppEvent::AgentSessionId("claude-xyz".into()), &db);
        app.on_app_event(
            AppEvent::AgentJobFound(NewJob {
                title: "Senior Rust".into(),
                url: "https://acme/1".into(),
                fit_score: Some(0.9),
                ..Default::default()
            }),
            &db,
        );
        app.on_app_event(
            AppEvent::AgentApplication {
                url: "https://acme/1".into(),
                status: "applied".into(),
                cv: Some("cv".into()),
                cover: Some("cover".into()),
                screenshot: None,
            },
            &db,
        );
        app.on_app_event(
            AppEvent::AgentPending {
                url: Some("https://acme/1".into()),
                kind: "captcha".into(),
                description: "resolver captcha".into(),
                field_key: None,
            },
            &db,
        );
        app.on_app_event(
            AppEvent::AgentFinished {
                result: Some("1 variante".into()),
                num_turns: Some(2),
                cost_usd: Some(0.07),
                is_error: false,
            },
            &db,
        );

        let s = db.stats().unwrap();
        assert_eq!(s.total_jobs, 1);
        assert_eq!(s.total_applications, 1);
        assert_eq!(s.applied, 1);
        assert_eq!(s.pending_actions, 1);

        let sessions = db.list_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].claude_session_id.as_deref(), Some("claude-xyz"));
        assert!(sessions[0].ended_at.is_some());
        assert_eq!(app.agent_status, AgentStatus::Idle);
        assert!(app.current_session_id.is_none());
    }

    #[test]
    fn dedup_pending_in_pipeline() {
        let db = Db::open_in_memory().unwrap();
        let mut app = App::new(&db).unwrap();
        let pend = || AppEvent::AgentPending {
            url: Some("https://acme/9".into()),
            kind: "captcha".into(),
            description: "captcha".into(),
            field_key: None,
        };
        // Same pending re-emitted 3x → one open line only.
        app.on_app_event(pend(), &db);
        app.on_app_event(pend(), &db);
        app.on_app_event(pend(), &db);
        assert_eq!(db.list_pending(false).unwrap().len(), 1);
    }

    #[test]
    fn feedback_persists_and_appears() {
        let db = Db::open_in_memory().unwrap();
        let mut app = App::new(&db).unwrap();
        app.on_app_event(
            AppEvent::AgentFeedback {
                summary: "bom progresso".into(),
                suggestions: "- ajuste a stack".into(),
            },
            &db,
        );
        assert_eq!(app.feedback.len(), 1);
        assert_eq!(app.feedback[0].summary, "bom progresso");
    }

    #[test]
    fn cv_review_persists_and_focuses_ats_tab() {
        let db = Db::open_in_memory().unwrap();
        let mut app = App::new(&db).unwrap();
        app.on_app_event(
            AppEvent::AgentCvReview {
                score: 82,
                target: "general".into(),
                report: "## Score\n82".into(),
            },
            &db,
        );
        assert_eq!(app.active, 6); // focuses the ATS tab
        let r = app.cv_review.as_ref().unwrap();
        assert_eq!(r.score, 82);
    }

    #[test]
    fn agent_finished_returns_to_idle_and_logs() {
        let db = Db::open_in_memory().unwrap();
        let mut app = App::new(&db).unwrap();
        app.on_app_event(AppEvent::AgentStarted, &db);
        assert_eq!(app.agent_status, AgentStatus::Running);
        app.on_app_event(
            AppEvent::AgentFinished {
                result: Some("ok".into()),
                num_turns: Some(1),
                cost_usd: Some(0.0),
                is_error: false,
            },
            &db,
        );
        assert_eq!(app.agent_status, AgentStatus::Idle);
    }

    #[test]
    fn session_log_receives_agent_text() {
        let db = Db::open_in_memory().unwrap();
        let mut app = App::new(&db).unwrap();
        app.on_app_event(AppEvent::AgentText("evaluating job X".into()), &db);
        assert!(app.log.iter().any(|l| l.contains("evaluating job X")));
    }

    #[test]
    fn renders_dashboard_with_title() {
        let db = Db::open_in_memory().unwrap();
        let app = App::new(&db).unwrap();
        let out = snapshot(&app, 100, 26);
        assert!(out.contains("jobRabbit"));
        assert!(out.contains("Dashboard"));
    }
}
