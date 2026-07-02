//! Assembly of prompts and orchestration of agent execution.
//!
//! Pure functions (build prompts from `Db`+`Settings`) and a reusable helper
//! for spawning. No UI dependency: TUI and web backend compose these
//! pieces with their own guards (preflight of `claude`, logging, status).

use tokio::sync::mpsc::UnboundedSender;

use crate::agent::{self, prompts, AgentConfig};
use crate::config::{self, Settings};
use crate::db::models::PendingAction;
use crate::db::Db;
use crate::event::AppEvent;
use crate::locale::Locale;

/// Assembles the [`AgentConfig`] from settings.
pub fn agent_config(settings: &Settings) -> AgentConfig {
    AgentConfig {
        claude_bin: settings.claude_bin.clone(),
        cwd: config::data_dir().ok(),
        chrome: settings.use_chrome,
        bypass_permissions: settings.bypass_permissions,
    }
}

/// Search prompts for all ACTIVE variants. `Err` if there are none.
pub fn search_prompts(db: &Db, settings: &Settings) -> Result<Vec<String>, String> {
    let profile = db.get_profile().unwrap_or_default();
    let variants: Vec<_> = db
        .list_variants()
        .unwrap_or_default()
        .into_iter()
        .filter(|v| v.enabled)
        .collect();
    if variants.is_empty() {
        return Err("no active variant — add/enable in Profile".into());
    }
    // Selected job sources are global (same for every variant).
    let sources = db.list_sources().unwrap_or_default();
    let mode = settings.apply_mode.clone();
    Ok(variants
        .iter()
        .map(|v| {
            prompts::search_and_evaluate(
                &profile,
                v,
                &mode,
                settings.dry_run,
                settings.require_human_review,
                settings.hybrid_threshold,
                settings.locale,
                settings.language_filter,
                &settings.work_model,
                &sources,
            )
        })
        .collect())
}

/// Résumé file for UPLOAD — always a format the Chrome upload tool can handle.
///
/// The upload tool cannot select `.docx`, and the system can render the CV in any
/// format, so uploads must never fail for format reasons. If the configured
/// `cv_file_path` is already a PDF, use it; otherwise render a PDF from the CV
/// markdown (`cv_content`) to a stable path and upload that. Falls back to the
/// configured path when there is no content to render.
fn upload_cv_path(settings: &Settings, cv_content: &str) -> String {
    if settings.cv_file_path.to_lowercase().ends_with(".pdf") {
        return settings.cv_file_path.clone();
    }
    let content = cv_content.trim();
    if !content.is_empty() {
        if let Ok(dir) = config::data_dir() {
            let dest = dir.join("resume-current.pdf");
            if crate::export::write_pdf_file(content, &dest).is_ok() {
                return dest.to_string_lossy().to_string();
            }
        }
    }
    settings.cv_file_path.clone()
}

/// Prompt to ACTUALLY SUBMIT an approved application (item `approval`).
///
/// `Err` if the item is not an approval or does not have associated job/application.
pub fn approval_prompt(
    db: &Db,
    settings: &Settings,
    pending: &PendingAction,
) -> Result<String, String> {
    if pending.kind != "approval" {
        return Err("this pending item is not an approval".into());
    }
    let job_id = pending.job_id.ok_or("approval without associated job")?;
    let job = db
        .get_job(job_id)
        .ok()
        .flatten()
        .ok_or("approval job not found")?;
    let app = db.application_for_job(job_id).ok().flatten();
    let cv = app
        .as_ref()
        .and_then(|a| a.cv_generated.clone())
        .unwrap_or_default();
    let cover = app
        .as_ref()
        .and_then(|a| a.cover_letter.clone())
        .unwrap_or_default();
    let ats = crate::ats::detect(&job.url);
    let playbook = crate::ats::playbook(ats, settings.locale);
    let answers = prompts::answers_block(&db.answers_map().unwrap_or_default(), settings.locale);
    let shots = crate::config::data_dir().map_err(|e| e.to_string())?.join("screenshots");
    let _ = std::fs::create_dir_all(&shots);
    let shots_str = shots.to_string_lossy().to_string();
    // Upload a PDF the Chrome tool can handle (rendered from the approved CV,
    // falling back to the base CV) instead of the raw configured file.
    let cv_source = if !cv.trim().is_empty() {
        cv.clone()
    } else {
        db.get_profile().unwrap_or_default().cv_base
    };
    let cv_upload = upload_cv_path(settings, &cv_source);
    Ok(prompts::apply_for_job(
        &job.title,
        &job.company,
        &job.url,
        &cv,
        &cover,
        &cv_upload,
        ats.name(),
        &playbook,
        &answers,
        &shots_str,
        settings.locale,
    ))
}

/// Prompt for apply-by-URL flow: extract job details from URL, detect language,
/// generate CV/cover in that language, evaluate fit, and apply (or simulate).
pub fn apply_by_url_prompt(
    url: &str,
    db: &Db,
    settings: &Settings,
) -> Result<String, String> {
    let answers = prompts::answers_block(&db.answers_map().unwrap_or_default(), settings.locale);
    let shots = crate::config::data_dir().map_err(|e| e.to_string())?.join("screenshots");
    let _ = std::fs::create_dir_all(&shots);
    let shots_str = shots.to_string_lossy().to_string();
    // Upload a PDF (rendered from the base CV) that the Chrome tool can handle.
    let profile = db.get_profile().unwrap_or_default();
    let cv_upload = upload_cv_path(settings, &profile.cv_base);
    Ok(prompts::apply_by_url(
        url,
        &cv_upload,
        &answers,
        settings.dry_run,
        settings.require_human_review,
        &shots_str,
        settings.locale,
    ))
}

/// Base CV text for ATS evaluation: file (if any) otherwise `cv_base`.
pub fn cv_source_text(db: &Db, settings: &Settings) -> String {
    let path = settings.cv_file_path.trim();
    if !path.is_empty() {
        if let Ok(t) = crate::import::extract_text(std::path::Path::new(path)) {
            return t;
        }
    }
    db.get_profile().map(|p| p.cv_base).unwrap_or_default()
}

/// Prompt for ATS evaluation of resume (general or against a target). `Err` if
/// there is no resume available.
pub fn cv_review_prompt(
    db: &Db,
    settings: &Settings,
    target: Option<&str>,
) -> Result<String, String> {
    let cv = cv_source_text(db, settings);
    if cv.trim().is_empty() {
        return Err("no resume — import a CV or fill in the profile".into());
    }
    Ok(prompts::review_cv(&cv, target, settings.locale))
}

/// Prompt to GENERATE an improved version of the resume (optimized for ATS).
/// `Err` if there is no resume available.
pub fn improve_cv_prompt(
    db: &Db,
    settings: &Settings,
    target: Option<&str>,
) -> Result<String, String> {
    let cv = cv_source_text(db, settings);
    if cv.trim().is_empty() {
        return Err("no resume — import a CV or fill in the profile".into());
    }
    Ok(prompts::improve_cv(&cv, target, settings.locale))
}

/// Prompt for periodic feedback analysis (recent results).
pub fn feedback_prompt(db: &Db, locale: Locale) -> String {
    let profile = db.get_profile().unwrap_or_default();
    let summary = results_summary(db);
    prompts::analyze_feedback(&profile, &summary, locale)
}

/// Summary of recent results to feed the feedback prompt.
pub fn results_summary(db: &Db) -> String {
    let s = db.stats().unwrap_or_default();
    let mut out = format!(
        "Totals — jobs: {}, applications: {}, applied: {}, pending: {}\n",
        s.total_jobs, s.total_applications, s.applied, s.pending_actions
    );
    if let Ok(jobs) = db.list_jobs() {
        out.push_str("Recent jobs:\n");
        for j in jobs.iter().take(10) {
            out.push_str(&format!(
                "- {} @ {} (fit {:.2})\n",
                j.title,
                j.company,
                j.fit_score.unwrap_or(0.0)
            ));
        }
    }
    out
}

/// Prompt to build the profile from a CV file. Returns
/// `(prompt, extracted_text)` — the caller saves the file path.
pub fn import_cv_prompt(path: &str, locale: Locale) -> Result<(String, String), String> {
    let text = crate::import::extract_text(std::path::Path::new(path))
        .map_err(|e| format!("failed to read {path}: {e}"))?;
    let prompt = prompts::build_profile(&text, locale);
    Ok((prompt, text))
}

/// Prompt to build the profile by browsing LinkedIn.
pub fn import_linkedin_prompt(url: &str, locale: Locale) -> String {
    prompts::build_profile_from_linkedin(url, locale)
}

/// Spawns an agent execution that processes `prompts` in sequence.
///
/// Emits `AgentStarted` at the start and `AgentFinished` when done (aggregating
/// turns/cost/error). It is the shared engine for search, application,
/// feedback, ATS and imports.
pub fn spawn_run(
    cfg: AgentConfig,
    prompts: Vec<String>,
    done_msg: String,
    tx: UnboundedSender<AppEvent>,
) {
    let _ = tx.send(AppEvent::AgentStarted);
    tokio::spawn(async move {
        let mut turns: u32 = 0;
        let mut cost: f64 = 0.0;
        let mut err = false;
        for p in &prompts {
            match agent::run_session(&cfg, p, None, &tx).await {
                Ok(s) => {
                    turns += s.num_turns.unwrap_or(0);
                    cost += s.cost_usd.unwrap_or(0.0);
                    err |= s.is_error;
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::AgentError(e.to_string()));
                    err = true;
                }
            }
        }
        let _ = tx.send(AppEvent::AgentFinished {
            result: Some(done_msg),
            num_turns: Some(turns),
            cost_usd: Some(cost),
            is_error: err,
        });
    });
}

/// Spawns a resumption of a previous session: single prompt with session continuation.
///
/// Similar to `spawn_run` but for continuing after manual intervention. Emits
/// `AgentStarted` before and `AgentFinished` after, like `spawn_run`.
pub fn spawn_resume(
    cfg: AgentConfig,
    prompt: String,
    resume_sid: String,
    done_msg: String,
    tx: UnboundedSender<AppEvent>,
) {
    let _ = tx.send(AppEvent::AgentStarted);
    tokio::spawn(async move {
        match agent::run_session(&cfg, &prompt, Some(&resume_sid), &tx).await {
            Ok(s) => {
                let _ = tx.send(AppEvent::AgentFinished {
                    result: Some(done_msg),
                    num_turns: s.num_turns,
                    cost_usd: s.cost_usd,
                    is_error: s.is_error,
                });
            }
            Err(e) => {
                let _ = tx.send(AppEvent::AgentError(e.to_string()));
                let _ = tx.send(AppEvent::AgentFinished {
                    result: Some(format!("error: {e}")),
                    num_turns: None,
                    cost_usd: None,
                    is_error: true,
                });
            }
        }
    });
}
