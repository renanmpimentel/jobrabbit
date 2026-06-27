//! Domain models persisted in SQLite.
//!
//! Timestamps are stored as RFC3339 strings (UTC) to map directly to
//! `TEXT` without extra rusqlite features.

use rusqlite::Row;
use serde::{Deserialize, Serialize};

/// Helper: current timestamp in RFC3339 (UTC).
pub fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub background: String,
    pub cv_base: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchVariant {
    pub id: i64,
    pub label: String,
    pub query: String,
    pub enabled: bool,
    pub created_at: String,
}

impl SearchVariant {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            label: row.get("label")?,
            query: row.get("query")?,
            enabled: row.get::<_, i64>("enabled")? != 0,
            created_at: row.get("created_at")?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Job {
    pub id: i64,
    pub title: String,
    pub company: String,
    pub url: String,
    pub source: String,
    pub description: String,
    pub fit_score: Option<f64>,
    pub found_at: String,
}

impl Job {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            title: row.get("title")?,
            company: row.get("company")?,
            url: row.get("url")?,
            source: row.get("source")?,
            description: row.get("description")?,
            fit_score: row.get("fit_score")?,
            found_at: row.get("found_at")?,
        })
    }
}

/// Data for a new job (no id; used in insert/upsert).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NewJob {
    pub title: String,
    pub company: String,
    pub url: String,
    pub source: String,
    pub description: String,
    pub fit_score: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Application {
    pub id: i64,
    pub job_id: i64,
    pub status: String,
    pub cv_generated: Option<String>,
    pub cover_letter: Option<String>,
    pub created_at: String,
}

impl Application {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            job_id: row.get("job_id")?,
            status: row.get("status")?,
            cv_generated: row.get("cv_generated")?,
            cover_letter: row.get("cover_letter")?,
            created_at: row.get("created_at")?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub id: i64,
    pub claude_session_id: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub summary: Option<String>,
    pub num_turns: Option<i64>,
    pub cost_usd: Option<f64>,
    pub output_tokens: Option<i64>,
}

impl Session {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            claude_session_id: row.get("claude_session_id")?,
            started_at: row.get("started_at")?,
            ended_at: row.get("ended_at")?,
            summary: row.get("summary")?,
            num_turns: row.get("num_turns")?,
            cost_usd: row.get("cost_usd")?,
            output_tokens: row.get("output_tokens")?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PendingAction {
    pub id: i64,
    pub job_id: Option<i64>,
    pub kind: String,
    pub description: String,
    pub url: Option<String>,
    pub field_key: Option<String>,
    pub resolved: bool,
    pub created_at: String,
}

impl PendingAction {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            job_id: row.get("job_id")?,
            kind: row.get("kind")?,
            description: row.get("description")?,
            url: row.get("url")?,
            field_key: row.get("field_key")?,
            resolved: row.get::<_, i64>("resolved")? != 0,
            created_at: row.get("created_at")?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Feedback {
    pub id: i64,
    pub summary: String,
    pub suggestions: String,
    pub created_at: String,
}

impl Feedback {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            summary: row.get("summary")?,
            suggestions: row.get("suggestions")?,
            created_at: row.get("created_at")?,
        })
    }
}

/// A screening answer from the answer bank.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Answer {
    pub key: String,
    pub label: String,
    pub value: String,
    pub updated_at: String,
}

impl Answer {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            key: row.get("key")?,
            label: row.get("label")?,
            value: row.get("value")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

/// ATS evaluation of the resume.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CvReview {
    pub id: i64,
    pub score: i64,
    pub target: String,
    pub report: String,
    pub created_at: String,
}

impl CvReview {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            score: row.get("score")?,
            target: row.get("target")?,
            report: row.get("report")?,
            created_at: row.get("created_at")?,
        })
    }
}

/// Improved resume version (optimized for ATS).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CvVersion {
    pub id: i64,
    pub target: String,
    pub content: String,
    pub created_at: String,
}

impl CvVersion {
    pub fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            target: row.get("target")?,
            content: row.get("content")?,
            created_at: row.get("created_at")?,
        })
    }
}

/// Canonical answer bank fields: `(key, label)`. Idempotent seed.
pub const ANSWER_FIELDS: &[(&str, &str)] = &[
    ("salary_expectation", "Salary expectation"),
    ("availability_start", "Start availability"),
    ("notice_period", "Notice period"),
    ("work_model", "Work model (remote/hybrid/on-site)"),
    ("relocation", "Available to relocate?"),
    ("travel", "Willing to travel?"),
    ("employment_type", "Employment type (CLT/PJ/both)"),
    ("disability", "Disability (PCD)?"),
    ("lgpd_consent", "LGPD consent"),
    ("english_level", "English level"),
    ("authorized_work_br", "Authorized to work in Brazil?"),
    ("needs_visa", "Needs visa/sponsorship?"),
    ("education_level", "Education level"),
    ("years_experience", "Years of experience"),
    ("drivers_license", "Has a driver's license?"),
    ("linkedin_url", "LinkedIn URL"),
    ("github_url", "GitHub URL"),
    ("preferred_city", "Preferred city (if relocating)"),
];

/// Maps legacy Portuguese answer keys to the current English keys, for migrating
/// existing databases. `linkedin_url`/`github_url` keep their keys (unchanged).
pub const ANSWER_KEY_MIGRATIONS: &[(&str, &str)] = &[
    ("pretensao_salarial", "salary_expectation"),
    ("disponibilidade_inicio", "availability_start"),
    ("aviso_previo", "notice_period"),
    ("modelo_trabalho", "work_model"),
    ("mudanca_cidade", "relocation"),
    ("disposto_viajar", "travel"),
    ("vinculo", "employment_type"),
    ("pcd", "disability"),
    ("lgpd_consentimento", "lgpd_consent"),
    ("ingles_nivel", "english_level"),
    ("autorizacao_trabalho_br", "authorized_work_br"),
    ("precisa_visto", "needs_visa"),
    ("escolaridade", "education_level"),
    ("anos_experiencia", "years_experience"),
    ("cnh", "drivers_license"),
    ("cidade_preferida", "preferred_city"),
];

/// Aggregated metrics for the dashboard.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    pub total_jobs: i64,
    pub total_applications: i64,
    pub applied: i64,
    pub pending_actions: i64,
}
