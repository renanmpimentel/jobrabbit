//! Local web backend (axum): exposes the core via HTTP + SSE to the browser UI.
//!
//! Runs on `localhost` on the SAME machine as the user — continues using the `claude`
//! CLI and logged-in Chrome. The UI (web-ui/) consumes:
//! - REST (`/api/*`) to read state and trigger commands;
//! - SSE (`/api/events`) for the real-time agent event stream.

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode, Uri},
    response::sse::{Event, KeepAlive, Sse},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};
use tower_http::cors::CorsLayer;

/// Built frontend (web-ui/dist), embedded in release binary.
#[derive(RustEmbed)]
#[folder = "web-ui/dist"]
struct Assets;

use crate::config::Settings;
use crate::core::{actions, persist, AgentStatus};
use crate::db::models::{
    Answer, Application, CvReview, CvVersion, Feedback, Job, PendingAction, Profile, SearchVariant,
    Stats,
};
use crate::db::Db;
use crate::event::AppEvent;

/// Envelope sent by SSE: the persistence result + event name.
/// The UI uses `logs` for the Session panel, `status` for the agent indicator, and
/// `refresh` to re-query the lists (jobs, pending, etc.).
#[derive(Clone, Serialize)]
struct WebEvent {
    event: String,
    logs: Vec<String>,
    status: Option<AgentStatus>,
    focus_tab: Option<usize>,
    refresh: bool,
}

/// State shared among handlers. `Clone` is cheap (everything is `Arc`).
#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<Db>>,
    settings: Arc<Mutex<Settings>>,
    events: broadcast::Sender<WebEvent>,
    agent_status: Arc<Mutex<AgentStatus>>,
    session_id: Arc<Mutex<Option<i64>>>,
    /// Buffer of recent logs (for the UI to show history when loading).
    log: Arc<Mutex<Vec<String>>>,
    /// Channel through which the agent emits events (consumed by the central task).
    agent_tx: UnboundedSender<AppEvent>,
}

/// Starts the web server on `127.0.0.1:port` and blocks until shutdown.
pub async fn serve(db: Db, port: u16) -> Result<()> {
    let (agent_tx, mut agent_rx) = unbounded_channel::<AppEvent>();
    let (events, _) = broadcast::channel::<WebEvent>(512);
    let state = AppState {
        db: Arc::new(Mutex::new(db)),
        settings: Arc::new(Mutex::new(Settings::load())),
        events,
        agent_status: Arc::new(Mutex::new(AgentStatus::Idle)),
        session_id: Arc::new(Mutex::new(None)),
        log: Arc::new(Mutex::new(Vec::new())),
        agent_tx: agent_tx.clone(),
    };

    // Central task: persists each agent event and broadcasts to SSE.
    {
        let state = state.clone();
        tokio::spawn(async move {
            while let Some(ev) = agent_rx.recv().await {
                // Idle: trigger automatic search (if configured).
                if matches!(ev, AppEvent::IdleReached) {
                    let auto = state.settings.lock().unwrap().auto_run_on_idle;
                    let running =
                        matches!(*state.agent_status.lock().unwrap(), AgentStatus::Running);
                    if auto && !running {
                        let _ = start_search(&state);
                    }
                    continue;
                }
                let name = event_name(&ev);
                let out = {
                    let db = state.db.lock().unwrap();
                    let mut sid = state.session_id.lock().unwrap();
                    persist::apply_event(&db, &ev, &mut sid)
                };
                if let Some(s) = out.status.clone() {
                    *state.agent_status.lock().unwrap() = s;
                }
                if !out.logs.is_empty() {
                    let mut log = state.log.lock().unwrap();
                    log.extend(out.logs.iter().cloned());
                    let len = log.len();
                    if len > 2000 {
                        log.drain(0..len - 2000);
                    }
                }
                if let Some((kind, desc)) = &out.notify {
                    crate::platform::notify::pending(kind, desc);
                }
                let _ = state.events.send(WebEvent {
                    event: name.to_string(),
                    logs: out.logs,
                    status: out.status,
                    focus_tab: out.focus_tab,
                    refresh: out.refresh,
                });
            }
        });
    }

    // Watch for idleness (emits `IdleReached` on the same event channel).
    let threshold = Duration::from_secs(state.settings.lock().unwrap().idle_threshold_secs);
    tokio::spawn(crate::platform::idle::watch(threshold, agent_tx));

    let app = router(state);
    // By default listens only on loopback (local app). `JOBRABBIT_HOST=0.0.0.0`
    // exposes on the network (e.g., access from another device) — use with caution.
    let host: std::net::IpAddr = std::env::var("JOBRABBIT_HOST")
        .ok()
        .and_then(|h| h.parse().ok())
        .unwrap_or_else(|| std::net::IpAddr::from([127, 0, 0, 1]));
    let addr = SocketAddr::new(host, port);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("jobRabbit web at http://{addr}");
    println!("🐇 jobRabbit — open http://{addr} in your browser (Ctrl+C to exit)");
    // Automatically opens the browser when there is a graphical session.
    if std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some() {
        crate::platform::open_url(&format!("http://{addr}"));
    }
    axum::serve(listener, app).await?;
    Ok(())
}

fn router(state: AppState) -> Router {
    Router::new()
        // Reads
        .route("/api/stats", get(get_stats))
        .route("/api/jobs", get(get_jobs))
        .route("/api/pending", get(get_pending))
        .route("/api/applications", get(get_applications))
        .route("/api/feedback", get(get_feedback))
        .route("/api/variants", get(get_variants).post(post_variant))
        .route("/api/profile", get(get_profile).post(post_profile))
        .route("/api/settings", get(get_settings).post(post_settings))
        .route("/api/answers", get(get_answers))
        .route("/api/cv-review", get(get_cv_review))
        .route("/api/cv-version", get(get_cv_version))
        .route("/api/cv-version/download", get(download_cv_version))
        .route("/api/status", get(get_status))
        .route("/api/log", get(get_log))
        .route("/api/doctor", get(get_doctor))
        // Commands
        .route("/api/run", post(post_run))
        .route("/api/feedback/run", post(post_feedback_run))
        .route("/api/cv-review/run", post(post_cv_review))
        .route("/api/cv-improve/run", post(post_cv_improve))
        .route("/api/import", post(post_import))
        .route("/api/reset-runs", post(reset_runs))
        .route("/api/variants/:id", delete(delete_variant))
        .route("/api/variants/:id/toggle", post(toggle_variant))
        .route("/api/pending/:id/resolve", post(resolve_pending))
        .route("/api/pending/:id/approve", post(approve_pending))
        .route("/api/pending/:id/answer", post(answer_pending))
        // Real-time
        .route("/api/events", get(sse_events))
        // Embedded frontend (SPA): any non-API route falls through to static_handler.
        .fallback(static_handler)
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Serves the embedded static files (web-ui/dist). Falls back to
/// `index.html` (SPA) when the path doesn't exist.
async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };
    match Assets::get(path) {
        Some(content) => {
            let mime = mime_for(path);
            ([(header::CONTENT_TYPE, mime)], content.data).into_response()
        }
        None => match Assets::get("index.html") {
            Some(content) => (
                [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                content.data,
            )
                .into_response(),
            None => (
                StatusCode::NOT_FOUND,
                "frontend not embedded — run `npm run build` in web-ui/ and recompile",
            )
                .into_response(),
        },
    }
}

fn mime_for(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("ico") => "image/x-icon",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        _ => "application/octet-stream",
    }
}

// ---- Helpers ---------------------------------------------------------------

type ApiError = (StatusCode, String);

fn internal<E: std::fmt::Display>(e: E) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

fn ok() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true }))
}

fn event_name(ev: &AppEvent) -> &'static str {
    match ev {
        AppEvent::Tick => "Tick",
        AppEvent::AgentStarted => "AgentStarted",
        AppEvent::AgentSessionId(_) => "AgentSessionId",
        AppEvent::AgentText(_) => "AgentText",
        AppEvent::AgentToolUse { .. } => "AgentToolUse",
        AppEvent::AgentJobFound(_) => "AgentJobFound",
        AppEvent::AgentApplication { .. } => "AgentApplication",
        AppEvent::AgentPending { .. } => "AgentPending",
        AppEvent::AgentAnswer { .. } => "AgentAnswer",
        AppEvent::AgentFeedback { .. } => "AgentFeedback",
        AppEvent::AgentCvReview { .. } => "AgentCvReview",
        AppEvent::AgentCvImproved { .. } => "AgentCvImproved",
        AppEvent::AgentProfile { .. } => "AgentProfile",
        AppEvent::AgentRaw(_) => "AgentRaw",
        AppEvent::AgentFinished { .. } => "AgentFinished",
        AppEvent::AgentError(_) => "AgentError",
        AppEvent::IdleReached => "IdleReached",
        AppEvent::Quit => "Quit",
    }
}

/// Assembles search prompts and triggers execution. Reused by `/api/run`
/// and by auto-execution when idle.
fn start_search(state: &AppState) -> std::result::Result<(), String> {
    if matches!(*state.agent_status.lock().unwrap(), AgentStatus::Running) {
        return Err("agent is already running".into());
    }
    let settings = state.settings.lock().unwrap().clone();
    let prompts = {
        let db = state.db.lock().unwrap();
        actions::search_prompts(&db, &settings)?
    };
    preflight_claude(&settings)?;
    let n = prompts.len();
    actions::spawn_run(
        actions::agent_config(&settings),
        prompts,
        format!("{n} variante(s) processada(s)"),
        state.agent_tx.clone(),
    );
    Ok(())
}

/// Triggers ONE session (single prompt): application, ATS, feedback, import.
fn spawn_one(state: &AppState, prompt: String, msg: &str) -> std::result::Result<(), String> {
    if matches!(*state.agent_status.lock().unwrap(), AgentStatus::Running) {
        return Err("agent is already running".into());
    }
    let settings = state.settings.lock().unwrap().clone();
    preflight_claude(&settings)?;
    actions::spawn_run(
        actions::agent_config(&settings),
        vec![prompt],
        msg.to_string(),
        state.agent_tx.clone(),
    );
    Ok(())
}

fn preflight_claude(settings: &Settings) -> std::result::Result<(), String> {
    if crate::platform::claude_available(&settings.claude_bin) {
        Ok(())
    } else {
        Err(format!(
            "`{}` not found in PATH — install/authenticate Claude Code",
            settings.claude_bin
        ))
    }
}

// ---- Handlers: leitura -----------------------------------------------------

async fn get_stats(State(s): State<AppState>) -> Result<Json<Stats>, ApiError> {
    let db = s.db.lock().unwrap();
    db.stats().map(Json).map_err(internal)
}
async fn get_jobs(State(s): State<AppState>) -> Result<Json<Vec<Job>>, ApiError> {
    let db = s.db.lock().unwrap();
    db.list_jobs().map(Json).map_err(internal)
}
async fn get_pending(State(s): State<AppState>) -> Result<Json<Vec<PendingAction>>, ApiError> {
    let db = s.db.lock().unwrap();
    db.list_pending(false).map(Json).map_err(internal)
}
async fn get_applications(State(s): State<AppState>) -> Result<Json<Vec<Application>>, ApiError> {
    let db = s.db.lock().unwrap();
    db.list_applications().map(Json).map_err(internal)
}
async fn get_feedback(State(s): State<AppState>) -> Result<Json<Vec<Feedback>>, ApiError> {
    let db = s.db.lock().unwrap();
    db.list_feedback().map(Json).map_err(internal)
}
async fn get_variants(State(s): State<AppState>) -> Result<Json<Vec<SearchVariant>>, ApiError> {
    let db = s.db.lock().unwrap();
    db.list_variants().map(Json).map_err(internal)
}
async fn get_profile(State(s): State<AppState>) -> Result<Json<Profile>, ApiError> {
    let db = s.db.lock().unwrap();
    db.get_profile().map(Json).map_err(internal)
}
async fn get_answers(State(s): State<AppState>) -> Result<Json<Vec<Answer>>, ApiError> {
    let db = s.db.lock().unwrap();
    db.get_answers().map(Json).map_err(internal)
}
async fn get_cv_review(State(s): State<AppState>) -> Result<Json<Option<CvReview>>, ApiError> {
    let db = s.db.lock().unwrap();
    db.latest_cv_review().map(Json).map_err(internal)
}
async fn get_cv_version(State(s): State<AppState>) -> Result<Json<Option<CvVersion>>, ApiError> {
    let db = s.db.lock().unwrap();
    db.latest_cv_version().map(Json).map_err(internal)
}
#[derive(Deserialize)]
struct DownloadParams {
    /// "md" (default) | "pdf" | "docx".
    format: Option<String>,
}

/// Downloads the latest improved CV version as md (default), pdf or docx.
async fn download_cv_version(
    State(s): State<AppState>,
    Query(params): Query<DownloadParams>,
) -> Response {
    let content = {
        let db = s.db.lock().unwrap();
        match db.latest_cv_version().ok().flatten() {
            Some(v) => v.content,
            None => return (StatusCode::NOT_FOUND, "no version generated yet").into_response(),
        }
    };
    let fmt = params.format.as_deref().unwrap_or("md");
    let (bytes, mime, filename): (Vec<u8>, &str, &str) = match fmt {
        "pdf" => match crate::export::to_pdf(&content) {
            Ok(b) => (b, "application/pdf", "resume-improved.pdf"),
            Err(e) => return internal(e).into_response(),
        },
        "docx" => match crate::export::to_docx(&content) {
            Ok(b) => (
                b,
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                "resume-improved.docx",
            ),
            Err(e) => return internal(e).into_response(),
        },
        _ => (
            content.into_bytes(),
            "text/markdown; charset=utf-8",
            "resume-improved.md",
        ),
    };
    (
        [
            (header::CONTENT_TYPE, mime.to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        bytes,
    )
        .into_response()
}
async fn get_settings(State(s): State<AppState>) -> Json<Settings> {
    Json(s.settings.lock().unwrap().clone())
}
async fn get_status(State(s): State<AppState>) -> Json<AgentStatus> {
    Json(s.agent_status.lock().unwrap().clone())
}
async fn get_log(State(s): State<AppState>) -> Json<Vec<String>> {
    Json(s.log.lock().unwrap().clone())
}
async fn get_doctor(State(s): State<AppState>) -> Json<Vec<crate::core::doctor::Check>> {
    let db = s.db.lock().unwrap();
    let settings = s.settings.lock().unwrap().clone();
    Json(crate::core::doctor::run(&db, &settings))
}

// ---- Handlers: comandos ----------------------------------------------------

async fn post_run(State(s): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    start_search(&s).map_err(|e| (StatusCode::CONFLICT, e))?;
    Ok(ok())
}

#[derive(Deserialize)]
struct ProfileBody {
    background: String,
    cv_base: String,
}
async fn post_profile(
    State(s): State<AppState>,
    Json(body): Json<ProfileBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let db = s.db.lock().unwrap();
    db.save_profile(&body.background, &body.cv_base)
        .map_err(internal)?;
    Ok(ok())
}

async fn post_settings(
    State(s): State<AppState>,
    Json(body): Json<Settings>,
) -> Result<Json<serde_json::Value>, ApiError> {
    body.save().map_err(internal)?;
    *s.settings.lock().unwrap() = body;
    Ok(ok())
}

#[derive(Deserialize)]
struct VariantBody {
    label: String,
    query: String,
}
async fn post_variant(
    State(s): State<AppState>,
    Json(body): Json<VariantBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let db = s.db.lock().unwrap();
    db.add_variant(&body.label, &body.query).map_err(internal)?;
    Ok(ok())
}
async fn delete_variant(
    State(s): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let db = s.db.lock().unwrap();
    db.delete_variant(id).map_err(internal)?;
    Ok(ok())
}

/// Apaga os dados de execução (jobs, applications, pending, sessions, feedback),
/// permitindo recomeçar uma busca do zero. Preserva perfil, variantes e respostas.
async fn reset_runs(State(s): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let db = s.db.lock().unwrap();
    db.clear_runs().map_err(internal)?;
    Ok(ok())
}

async fn toggle_variant(
    State(s): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let db = s.db.lock().unwrap();
    let cur = db
        .list_variants()
        .map_err(internal)?
        .into_iter()
        .find(|v| v.id == id)
        .ok_or((StatusCode::NOT_FOUND, "variant not found".into()))?;
    db.set_variant_enabled(id, !cur.enabled).map_err(internal)?;
    Ok(ok())
}

async fn resolve_pending(
    State(s): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let db = s.db.lock().unwrap();
    db.resolve_pending(id).map_err(internal)?;
    Ok(ok())
}

async fn approve_pending(
    State(s): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (prompt, settings) = {
        let db = s.db.lock().unwrap();
        let settings = s.settings.lock().unwrap().clone();
        let p = db
            .list_pending(false)
            .map_err(internal)?
            .into_iter()
            .find(|p| p.id == id)
            .ok_or((StatusCode::NOT_FOUND, "pending not found".into()))?;
        let prompt = actions::approval_prompt(&db, &settings, &p)
            .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
        (prompt, settings)
    };
    let _ = settings;
    spawn_one(&s, prompt, "end of application attempt").map_err(|e| (StatusCode::CONFLICT, e))?;
    Ok(ok())
}

#[derive(Deserialize)]
struct AnswerBody {
    value: String,
}
async fn answer_pending(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<AnswerBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let db = s.db.lock().unwrap();
    let p = db
        .list_pending(false)
        .map_err(internal)?
        .into_iter()
        .find(|p| p.id == id)
        .ok_or((StatusCode::NOT_FOUND, "pending not found".into()))?;
    let key = p.field_key.clone().unwrap_or_else(|| "answer".into());
    db.set_answer(&key, &p.description, &body.value)
        .map_err(internal)?;
    db.resolve_pending(id).map_err(internal)?;
    Ok(ok())
}

#[derive(Deserialize)]
struct CvReviewBody {
    /// Target job text (or empty/absent for general evaluation).
    target: Option<String>,
}
async fn post_cv_review(
    State(s): State<AppState>,
    body: Option<Json<CvReviewBody>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let target = body.and_then(|b| b.0.target);
    let prompt = {
        let db = s.db.lock().unwrap();
        let settings = s.settings.lock().unwrap().clone();
        let alvo = target.as_deref().filter(|t| !t.trim().is_empty());
        actions::cv_review_prompt(&db, &settings, alvo).map_err(|e| (StatusCode::BAD_REQUEST, e))?
    };
    spawn_one(&s, prompt, "resume evaluation completed").map_err(|e| (StatusCode::CONFLICT, e))?;
    Ok(ok())
}

async fn post_cv_improve(
    State(s): State<AppState>,
    body: Option<Json<CvReviewBody>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let target = body.and_then(|b| b.0.target);
    let prompt = {
        let db = s.db.lock().unwrap();
        let settings = s.settings.lock().unwrap().clone();
        let alvo = target.as_deref().filter(|t| !t.trim().is_empty());
        actions::improve_cv_prompt(&db, &settings, alvo)
            .map_err(|e| (StatusCode::BAD_REQUEST, e))?
    };
    spawn_one(&s, prompt, "improved resume version generated")
        .map_err(|e| (StatusCode::CONFLICT, e))?;
    Ok(ok())
}

async fn post_feedback_run(State(s): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let prompt = {
        let locale = s.settings.lock().unwrap().locale;
        let db = s.db.lock().unwrap();
        actions::feedback_prompt(&db, locale)
    };
    spawn_one(&s, prompt, "feedback analysis generated").map_err(|e| (StatusCode::CONFLICT, e))?;
    Ok(ok())
}

#[derive(Deserialize)]
struct ImportBody {
    cv_path: Option<String>,
    linkedin_url: Option<String>,
}
async fn post_import(
    State(s): State<AppState>,
    Json(body): Json<ImportBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let locale = s.settings.lock().unwrap().locale;
    let (prompt, msg) = if let Some(path) = body.cv_path.filter(|p| !p.trim().is_empty()) {
        let (prompt, _text) =
            actions::import_cv_prompt(&path, locale).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
        // Store CV path for future upload.
        let mut settings = s.settings.lock().unwrap();
        settings.cv_file_path = path;
        let _ = settings.save();
        (prompt, "profile imported from CV")
    } else if let Some(url) = body.linkedin_url.filter(|u| !u.trim().is_empty()) {
        let prompt = actions::import_linkedin_prompt(&url, locale);
        let mut settings = s.settings.lock().unwrap();
        settings.linkedin_url = url;
        let _ = settings.save();
        (prompt, "profile imported from LinkedIn")
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            "provide cv_path or linkedin_url".into(),
        ));
    };
    spawn_one(&s, prompt, msg).map_err(|e| (StatusCode::CONFLICT, e))?;
    Ok(ok())
}

// ---- SSE -------------------------------------------------------------------

async fn sse_events(
    State(s): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = s.events.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| {
        let we = msg.ok()?;
        let data = serde_json::to_string(&we).ok()?;
        // We don't name the event (no `.event(...)`): so it arrives as the default type
        // `message` and the browser's `EventSource.onmessage` fires. The
        // event name is inside the JSON (`we.event`).
        Some(Ok(Event::default().data(data)))
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}
