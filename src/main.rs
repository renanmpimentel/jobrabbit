//! jobRabbit 🐇 — auto-application to jobs, 100% terminal (Linux).
//!
//! Orchestrates Claude Code CLI (`claude`) via PTY, reading stream-json to
//! feed a TUI (ratatui). See the plan in docs/ for the architecture.

// Several items are deliberate API/data surface: structs mirroring the
// stream-json JSON (fields read via serde but not all consumed) and helpers
// kept for future evolution (standalone CV/cover-letter generation, secrets, etc.).
#![allow(dead_code)]

mod agent;
mod ats;
mod config;
mod core;
mod db;
mod event;
mod export;
mod import;
mod locale;
mod platform;
mod tui;
mod web;

use anyhow::Result;
use tokio::sync::mpsc::unbounded_channel;

use db::Db;
use event::AppEvent;

fn main() -> Result<()> {
    init_tracing();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(async_main())
}

async fn async_main() -> Result<()> {
    // Snapshot mode: renders screens as text and exits (no TTY). Useful for
    // visual preview/debug: `jobrabbit --snapshot`.
    if std::env::args().any(|a| a == "--snapshot") {
        return snapshot_mode();
    }

    // Doctor: checks dependencies and configuration and exits (does not start the UI).
    if std::env::args().any(|a| a == "--doctor") {
        return doctor_cli();
    }

    // E2E self-test: runs ONE real `claude` session (safe prompt, no browsing)
    // through the entire pipeline (PTY → stream-json → protocol → persistence) and validates.
    if std::env::args().any(|a| a == "--selftest-agent") {
        return selftest_agent().await;
    }

    // Limpa os dados de execução (vagas não aplicadas, candidaturas não aplicadas,
    // pendências, sessões e feedback) e sai. Preserva perfil, variantes de busca,
    // respostas e TODAS as vagas/candidaturas com status='applied'.
    if std::env::args().any(|a| a == "--reset-runs") {
        let db = Db::open(config::db_path()?)?;
        db.clear_runs()?;
        println!("execution data cleared (applied jobs and their proofs are kept)");
        return Ok(());
    }

    // Headless profile import (CV, LinkedIn, or both combined).
    // Flags without value fall back to configured defaults (settings.json).
    let want_cv = std::env::args().any(|a| a == "--import-cv");
    let want_li = std::env::args().any(|a| a == "--import-linkedin");
    if want_cv || want_li {
        let cfg = config::Settings::load();
        let cv = flag_value("--import-cv").or_else(|| {
            (want_cv && !cfg.cv_file_path.is_empty()).then(|| cfg.cv_file_path.clone())
        });
        let url = flag_value("--import-linkedin").or_else(|| {
            (want_li && !cfg.linkedin_url.is_empty()).then(|| cfg.linkedin_url.clone())
        });
        match (cv, url) {
            (Some(p), Some(u)) => return import_profile_cli(ImportSource::Both(p, u)).await,
            (Some(p), None) => return import_profile_cli(ImportSource::Cv(p)).await,
            (None, Some(u)) => return import_profile_cli(ImportSource::Linkedin(u)).await,
            (None, None) => anyhow::bail!(
                "nothing to import — pass a value or configure linkedin_url/cv_file_path in the Config tab"
            ),
        }
    }

    // Cutover: the DEFAULT mode is local web UI (browser). Use `--tui` for the
    // classic TUI. Everything remains local (claude + user's Chrome).
    // `--web` continues working as an explicit alias.
    if !std::env::args().any(|a| a == "--tui") {
        let db = Db::open(config::db_path()?)?;
        let port = flag_value("--port")
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8787);
        return web::serve(db, port).await;
    }

    let db = Db::open(config::db_path()?)?;
    tracing::info!("jobRabbit starting (TUI)");

    // Event channel from core → TUI. The `tx` is cloned for those who produce
    // events (agent, idle, demos).
    let (tx, rx) = unbounded_channel::<AppEvent>();
    tui::run(db, tx, rx).await
}

/// Renders each tab with demo data and prints to stdout.
fn snapshot_mode() -> Result<()> {
    use db::models::NewJob;

    let db = Db::open_in_memory()?;
    // Seeds demo data so the preview has content.
    db.save_profile(
        "Backend dev, 8 years, Rust/Go, remote.",
        "—— Base CV ——\nExperience: ...",
    )?;
    db.add_variant("Senior Remote", "senior rust backend remote")?;
    db.add_variant("Tech Lead", "tech lead plataforma")?;
    let job = db.upsert_job(&NewJob {
        title: "Senior Rust Engineer".into(),
        company: "Acme".into(),
        url: "https://acme.jobs/123".into(),
        fit_score: Some(0.87),
        ..Default::default()
    })?;
    db.add_application(job, "applied", Some("cv"), Some("cover"))?;
    db.add_pending(
        Some(job),
        "captcha",
        "Solve captcha in registration",
        Some("https://acme.jobs/123"),
    )?;
    db.add_feedback(
        "Good average fit (0.85); few remote jobs found",
        "- Add 'remote staff engineer' variant\n- Highlight leadership in CV",
    )?;
    db.set_answer("salary_expectation", "Salary expectation", "R$ 25,000")?;
    db.set_answer("english_level", "English level", "advanced")?;
    db.add_cv_review(
        82,
        "Engineering Manager (Payments)",
        "## Score: 82/100\n## Strengths\n- Quantified achievements\n## Issues\n- Missing skills section\n## Suggestions\n- Add keywords: Kafka, SRE",
    )?;

    let mut app = tui::App::new(&db)?;
    // Sample values for the Config tab preview.
    app.settings.linkedin_url = "https://www.linkedin.com/in/exemplo".into();
    app.settings.cv_file_path = "/home/user/cv.docx".into();
    app.on_app_event(
        event::AppEvent::AgentText("Analyzing job: Senior Rust Engineer @ Acme (fit 0.87)".into()),
        &db,
    );
    app.on_app_event(
        event::AppEvent::AgentToolUse {
            name: "Bash".into(),
            input: serde_json::json!({"command":"open job in Chrome"}),
        },
        &db,
    );
    app.on_app_event(
        event::AppEvent::AgentFinished {
            result: Some("Application sent.".into()),
            num_turns: Some(3),
            cost_usd: Some(0.12),
            is_error: false,
        },
        &db,
    );

    for (i, name) in tui::TABS.iter().enumerate() {
        app.active = i;
        println!("\n===== {name} =====");
        print!("{}", tui::snapshot(&app, 100, 26));
    }
    Ok(())
}

/// Environment diagnostic in terminal (`--doctor`). Exits with code 1 if there are errors.
fn doctor_cli() -> Result<()> {
    let db = Db::open(config::db_path()?)?;
    let settings = config::Settings::load();
    let checks = core::doctor::run(&db, &settings);

    println!("🐇 jobRabbit — environment diagnostic\n");
    for c in &checks {
        let icon = match c.status {
            core::doctor::Status::Ok => "✔",
            core::doctor::Status::Warn => "⚠",
            core::doctor::Status::Fail => "✖",
        };
        println!("{icon}  {} — {}", c.name, c.detail);
        if let Some(h) = &c.hint {
            println!("       ↳ {h}");
        }
    }
    let (ok, warn, fail) = core::doctor::summary(&checks);
    println!("\nSummary: {ok} ok · {warn} warning(s) · {fail} error(s).");
    if fail > 0 {
        std::process::exit(1);
    }
    Ok(())
}

/// Reads the value that follows a CLI flag (`--flag value`).
fn flag_value(flag: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

enum ImportSource {
    Cv(String),
    Linkedin(String),
    Both(String, String),
}

/// Headless profile import via real `claude`, persisting to user's DB.
async fn import_profile_cli(source: ImportSource) -> Result<()> {
    use agent::AgentConfig;

    let db = Db::open(config::db_path()?)?;
    let mut app = tui::App::new(&db)?;
    let (tx, mut rx) = unbounded_channel::<AppEvent>();

    let prompt = match source {
        ImportSource::Cv(path) => {
            println!("jobRabbit — importing profile from: {path}");
            let text = import::extract_text(std::path::Path::new(&path))?;
            app.settings.cv_file_path = path.clone();
            let _ = app.settings.save();
            agent::prompts::build_profile(&text, app.settings.locale)
        }
        ImportSource::Linkedin(url) => {
            println!("jobRabbit — importing profile from LinkedIn: {url}");
            agent::prompts::build_profile_from_linkedin(&url, app.settings.locale)
        }
        ImportSource::Both(path, url) => {
            println!("jobRabbit — importing profile from: {path} + LinkedIn: {url}");
            let text = import::extract_text(std::path::Path::new(&path))?;
            app.settings.cv_file_path = path.clone();
            let _ = app.settings.save();
            agent::prompts::build_profile_combined(&text, &url, app.settings.locale)
        }
    };

    let cfg = AgentConfig {
        claude_bin: app.settings.claude_bin.clone(),
        cwd: config::data_dir().ok(),
        chrome: app.settings.use_chrome,
        bypass_permissions: app.settings.bypass_permissions,
    };

    let _ = tx.send(AppEvent::AgentStarted);
    let summary = agent::run_session(&cfg, &prompt, None, &tx).await?;
    let _ = tx.send(AppEvent::AgentFinished {
        result: summary.result.clone(),
        num_turns: summary.num_turns,
        cost_usd: summary.cost_usd,
        is_error: summary.is_error,
    });
    drop(tx);

    while let Some(ev) = rx.recv().await {
        app.on_app_event(ev, &db);
    }

    let p = db.get_profile()?;
    println!("\n--- Profile imported ---");
    println!("background: {}", truncate(&p.background, 120));
    println!("cv_base:    {} chars", p.cv_base.len());
    let vars = db.list_variants()?;
    println!("variants:   {}", vars.len());
    for v in &vars {
        println!("  - {} — {}", v.label, v.query);
    }
    if p.background.is_empty() && p.cv_base.is_empty() {
        anyhow::bail!("nothing was imported — check the file/URL and agent output");
    }
    println!("\n✔ profile imported");
    Ok(())
}

fn truncate(s: &str, n: usize) -> String {
    let s = s.replace('\n', " ");
    if s.chars().count() > n {
        format!("{}…", s.chars().take(n).collect::<String>())
    } else {
        s
    }
}

/// End-to-end self-test with real `claude` (no browsing).
async fn selftest_agent() -> Result<()> {
    use agent::AgentConfig;

    println!("jobRabbit — E2E selftest (runs real `claude`, safe prompt without browsing)\n");

    let db = Db::open_in_memory()?;
    let mut app = tui::App::new(&db)?;
    let (tx, mut rx) = unbounded_channel::<AppEvent>();

    let prompt = "You are in automated test mode. DO NOT browse the web or use \
        tools. Emit EXACTLY these three lines (NDJSON), nothing else, without code fences:\n\
        {\"type\":\"job\",\"title\":\"QA Tester\",\"company\":\"TestCorp\",\"url\":\"https://test/42\",\"source\":\"linkedin\",\"description\":\"test job\",\"fit_score\":0.95}\n\
        {\"type\":\"application\",\"url\":\"https://test/42\",\"status\":\"applied\",\"cv\":\"test cv\",\"cover\":\"test letter\"}\n\
        {\"type\":\"pending\",\"url\":\"https://test/42\",\"kind\":\"captcha\",\"description\":\"solve test captcha\"}";

    // Isolated test: no Chrome/browsing, but bypassing permissions (no prompts).
    let cfg = AgentConfig {
        chrome: false,
        bypass_permissions: true,
        ..AgentConfig::default()
    };

    // Event order in the channel: started → (session events) → finished.
    let _ = tx.send(AppEvent::AgentStarted);
    let summary = agent::run_session(&cfg, prompt, None, &tx).await?;
    let _ = tx.send(AppEvent::AgentFinished {
        result: summary.result.clone(),
        num_turns: summary.num_turns,
        cost_usd: summary.cost_usd,
        is_error: summary.is_error,
    });
    drop(tx);

    while let Some(ev) = rx.recv().await {
        app.on_app_event(ev, &db);
    }

    let s = db.stats()?;
    println!("\n--- Result persisted in SQLite ---");
    println!("jobs:         {}", s.total_jobs);
    println!("applications: {}", s.total_applications);
    println!("applied:      {}", s.applied);
    println!("pending:      {}", s.pending_actions);
    let sessions = db.list_sessions()?;
    if let Some(sess) = sessions.first() {
        println!(
            "session:      claude_id={:?} turns={:?} cost=${:.4}",
            sess.claude_session_id,
            sess.num_turns,
            sess.cost_usd.unwrap_or(0.0)
        );
    }

    let ok =
        s.total_jobs == 1 && s.total_applications == 1 && s.applied == 1 && s.pending_actions == 1;
    println!(
        "\n{}",
        if ok {
            "✔ PASS — E2E pipeline validated"
        } else {
            "✖ FAIL — unexpected counts"
        }
    );
    if !ok {
        anyhow::bail!("selftest failed");
    }
    Ok(())
}

/// Logs go to a file (not to the screen, which is occupied by the TUI).
fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter =
        EnvFilter::try_from_env("JOBRABBIT_LOG").unwrap_or_else(|_| EnvFilter::new("info"));

    // Tries file at ~/.local/share/jobrabbit/jobrabbit.log; falls back to stderr.
    if let Ok(dir) = config::data_dir() {
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("jobrabbit.log");
        if let Ok(file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = fmt()
                .with_env_filter(filter)
                .with_ansi(false)
                .with_writer(move || file.try_clone().expect("clone log handle"))
                .try_init();
            return;
        }
    }
    let _ = fmt().with_writer(std::io::stderr).try_init();
}
