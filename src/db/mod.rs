//! Persistence (SQLite via rusqlite).
//!
//! [`Db`] wraps a `Connection`, applies the schema on open, and provides the
//! CRUD methods used by screens and the agent.

pub mod models;

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;

use models::*;

const SCHEMA: &str = include_str!("schema.sql");

pub struct Db {
    conn: Connection,
}

impl Db {
    /// Opens (or creates) the database at the given path and applies the schema.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path).with_context(|| format!("open db at {path:?}"))?;
        Self::from_conn(conn)
    }

    /// In-memory database (tests).
    pub fn open_in_memory() -> Result<Self> {
        Self::from_conn(Connection::open_in_memory()?)
    }

    fn from_conn(conn: Connection) -> Result<Self> {
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        conn.execute_batch(SCHEMA).context("apply schema")?;
        // Light migration for old DBs (ignores if column already exists).
        let _ = conn.execute("ALTER TABLE pending_actions ADD COLUMN field_key TEXT", []);
        let _ = conn.execute("ALTER TABLE applications ADD COLUMN screenshot_path TEXT", []);
        let db = Self { conn };
        db.migrate_answer_keys()?;
        db.seed_answers()?;
        Ok(db)
    }

    /// Migrates legacy Portuguese answer keys to the current English keys,
    /// carrying over each saved value/timestamp. Idempotent and safe to run on
    /// every open. Must run BEFORE [`Self::seed_answers`].
    fn migrate_answer_keys(&self) -> Result<()> {
        for (old_key, new_key) in ANSWER_KEY_MIGRATIONS {
            // Canonical English label for the new key.
            let label = ANSWER_FIELDS
                .iter()
                .find(|(k, _)| k == new_key)
                .map(|(_, l)| *l)
                .unwrap_or("");
            // Carry the legacy row's value/timestamp to the new key. If the new key
            // doesn't exist yet, insert it; if it already exists but is still empty
            // (e.g. freshly seeded), fill it from the legacy value. A non-empty new
            // value is never overwritten. No-op if the legacy key is absent.
            self.conn.execute(
                "INSERT INTO answers (key, label, value, updated_at)
                 SELECT ?2, ?3, value, updated_at FROM answers WHERE key = ?1
                 ON CONFLICT(key) DO UPDATE SET
                     value = excluded.value,
                     updated_at = excluded.updated_at
                 WHERE answers.value = '' AND excluded.value != ''",
                params![old_key, new_key, label],
            )?;
            // Drop the legacy key.
            self.conn
                .execute("DELETE FROM answers WHERE key = ?1", params![old_key])?;
        }
        Ok(())
    }

    /// Ensures that the canonical answer fields (triage + identity) exist (with
    /// empty value).
    fn seed_answers(&self) -> Result<()> {
        for (key, label) in ANSWER_FIELDS.iter().chain(IDENTITY_FIELDS.iter()) {
            self.conn.execute(
                "INSERT OR IGNORE INTO answers (key, label, value, updated_at)
                 VALUES (?1, ?2, '', ?3)",
                params![key, label, now_rfc3339()],
            )?;
        }
        Ok(())
    }

    // ---- Answers (answer bank) ---------------------------------------------

    pub fn get_answers(&self) -> Result<Vec<Answer>> {
        let mut stmt = self
            .conn
            .prepare("SELECT key, label, value, updated_at FROM answers ORDER BY rowid")?;
        let rows = stmt
            .query_map([], Answer::from_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// Map `key -> value` with only filled answers (non-empty value).
    pub fn answers_map(&self) -> Result<std::collections::HashMap<String, String>> {
        let mut map = std::collections::HashMap::new();
        for a in self.get_answers()? {
            if !a.value.trim().is_empty() {
                map.insert(a.key, a.value);
            }
        }
        Ok(map)
    }

    /// Sets/updates an answer (creates record if key is new).
    pub fn set_answer(&self, key: &str, label: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO answers (key, label, value, updated_at) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(key) DO UPDATE SET value = ?3, updated_at = ?4,
                label = CASE WHEN ?2 != '' THEN ?2 ELSE label END",
            params![key, label, value, now_rfc3339()],
        )?;
        Ok(())
    }

    /// Answers not yet filled.
    pub fn missing_answers(&self) -> Result<Vec<Answer>> {
        Ok(self
            .get_answers()?
            .into_iter()
            .filter(|a| a.value.trim().is_empty())
            .collect())
    }

    // ---- Profile ----------------------------------------------------------

    pub fn get_profile(&self) -> Result<Profile> {
        let mut stmt = self
            .conn
            .prepare("SELECT background, cv_base, updated_at FROM profile WHERE id = 1")?;
        let p = stmt
            .query_row([], |r| {
                Ok(Profile {
                    background: r.get(0)?,
                    cv_base: r.get(1)?,
                    updated_at: r.get(2)?,
                })
            })
            .ok();
        Ok(p.unwrap_or_default())
    }

    pub fn save_profile(&self, background: &str, cv_base: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO profile (id, background, cv_base, updated_at) VALUES (1, ?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET background = ?1, cv_base = ?2, updated_at = ?3",
            params![background, cv_base, now_rfc3339()],
        )?;
        Ok(())
    }

    // ---- Search variants --------------------------------------------------

    pub fn add_variant(&self, label: &str, query: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO search_variants (label, query, enabled, created_at) VALUES (?1, ?2, 1, ?3)",
            params![label, query, now_rfc3339()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_variants(&self) -> Result<Vec<SearchVariant>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, label, query, enabled, created_at FROM search_variants ORDER BY id",
        )?;
        let rows = stmt
            .query_map([], SearchVariant::from_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn set_variant_enabled(&self, id: i64, enabled: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE search_variants SET enabled = ?1 WHERE id = ?2",
            params![enabled as i64, id],
        )?;
        Ok(())
    }

    pub fn delete_variant(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM search_variants WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Apaga os dados de execução (vagas não aplicadas, candidaturas não aplicadas, pendências,
    /// sessões e feedback), numa transação atômica. Preserva perfil, variantes de busca,
    /// respostas de triagem, dados de currículo (cv_reviews/cv_versions), e TODAS as vagas
    /// e candidaturas com status='applied' (e seus screenshots). Permite recomeçar uma busca
    /// do zero mantendo o histórico de aplicações bem-sucedidas. Idempotente.
    pub fn clear_runs(&self) -> Result<()> {
        self.conn.execute_batch(
            "BEGIN;
             DELETE FROM applications WHERE status != 'applied';
             DELETE FROM pending_actions;
             DELETE FROM sessions;
             DELETE FROM feedback;
             DELETE FROM jobs WHERE NOT EXISTS (SELECT 1 FROM applications a WHERE a.job_id = jobs.id AND a.status = 'applied');
             COMMIT;",
        )?;
        Ok(())
    }

    // ---- Jobs -------------------------------------------------------------

    /// Inserts a job or updates by unique URL. Returns the id.
    pub fn upsert_job(&self, job: &NewJob) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO jobs (title, company, url, source, description, fit_score, found_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(url) DO UPDATE SET
                title = ?1, company = ?2, source = ?4, description = ?5, fit_score = ?6",
            params![
                job.title,
                job.company,
                job.url,
                job.source,
                job.description,
                job.fit_score,
                now_rfc3339()
            ],
        )?;
        let id: i64 = self.conn.query_row(
            "SELECT id FROM jobs WHERE url = ?1",
            params![job.url],
            |r| r.get(0),
        )?;
        Ok(id)
    }

    /// Job id with the given URL, if it exists.
    pub fn job_id_by_url(&self, url: &str) -> Result<Option<i64>> {
        let id = self
            .conn
            .query_row("SELECT id FROM jobs WHERE url = ?1", params![url], |r| {
                r.get::<_, i64>(0)
            })
            .ok();
        Ok(id)
    }

    pub fn list_jobs(&self) -> Result<Vec<Job>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, company, url, source, description, fit_score, found_at
             FROM jobs ORDER BY found_at DESC",
        )?;
        let rows = stmt
            .query_map([], Job::from_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// Jobs that don't yet have an `applied` application — used by the web jobs
    /// list so already-applied vacancies disappear from view.
    pub fn list_jobs_unapplied(&self) -> Result<Vec<Job>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, company, url, source, description, fit_score, found_at
             FROM jobs j
             WHERE NOT EXISTS (
                 SELECT 1 FROM applications a
                 WHERE a.job_id = j.id AND a.status = 'applied'
             )
             ORDER BY found_at DESC",
        )?;
        let rows = stmt
            .query_map([], Job::from_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// Jobs that have an `applied` application.
    pub fn list_jobs_applied(&self) -> Result<Vec<Job>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, company, url, source, description, fit_score, found_at
             FROM jobs j
             WHERE EXISTS (
                 SELECT 1 FROM applications a
                 WHERE a.job_id = j.id AND a.status = 'applied'
             )
             ORDER BY found_at DESC",
        )?;
        let rows = stmt
            .query_map([], Job::from_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    // ---- Applications -----------------------------------------------------

    pub fn add_application(
        &self,
        job_id: i64,
        status: &str,
        cv: Option<&str>,
        cover: Option<&str>,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO applications (job_id, status, cv_generated, cover_letter, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![job_id, status, cv, cover, now_rfc3339()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Current application for a job (if any).
    pub fn application_for_job(&self, job_id: i64) -> Result<Option<Application>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, job_id, status, cv_generated, cover_letter, screenshot_path, created_at
             FROM applications WHERE job_id = ?1 ORDER BY id DESC LIMIT 1",
        )?;
        let app = stmt.query_row(params![job_id], Application::from_row).ok();
        Ok(app)
    }

    /// Updates only the status of the most recent application for a job (preserves CV/cover).
    pub fn set_application_status(&self, job_id: i64, status: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE applications SET status = ?1
             WHERE id = (SELECT id FROM applications WHERE job_id = ?2 ORDER BY id DESC LIMIT 1)",
            params![status, job_id],
        )?;
        Ok(())
    }

    /// Updates the screenshot_path of the most recent application for a job.
    pub fn set_application_screenshot(&self, job_id: i64, path: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE applications SET screenshot_path = ?1
             WHERE id = (SELECT id FROM applications WHERE job_id = ?2 ORDER BY id DESC LIMIT 1)",
            params![path, job_id],
        )?;
        Ok(())
    }

    /// Application by id.
    pub fn get_application(&self, id: i64) -> Result<Option<Application>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, job_id, status, cv_generated, cover_letter, screenshot_path, created_at
             FROM applications WHERE id = ?1",
        )?;
        Ok(stmt.query_row(params![id], Application::from_row).ok())
    }

    /// Job by id.
    pub fn get_job(&self, id: i64) -> Result<Option<Job>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, company, url, source, description, fit_score, found_at
             FROM jobs WHERE id = ?1",
        )?;
        Ok(stmt.query_row(params![id], Job::from_row).ok())
    }

    pub fn list_applications(&self) -> Result<Vec<Application>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, job_id, status, cv_generated, cover_letter, screenshot_path, created_at
             FROM applications ORDER BY created_at DESC",
        )?;
        let rows = stmt
            .query_map([], Application::from_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    // ---- Sessions ---------------------------------------------------------

    /// Creates a session and returns the local id.
    pub fn start_session(&self, claude_session_id: Option<&str>) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO sessions (claude_session_id, started_at) VALUES (?1, ?2)",
            params![claude_session_id, now_rfc3339()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Records the `claude`'s `session_id` (captured in the `init` event).
    pub fn set_session_claude_id(&self, id: i64, claude_session_id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE sessions SET claude_session_id = ?1 WHERE id = ?2",
            params![claude_session_id, id],
        )?;
        Ok(())
    }

    /// Finishes the session with metrics from the `result` event.
    pub fn finish_session(
        &self,
        id: i64,
        summary: Option<&str>,
        num_turns: Option<i64>,
        cost_usd: Option<f64>,
        output_tokens: Option<i64>,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE sessions SET ended_at = ?1, summary = ?2, num_turns = ?3, cost_usd = ?4, output_tokens = ?5
             WHERE id = ?6",
            params![now_rfc3339(), summary, num_turns, cost_usd, output_tokens, id],
        )?;
        Ok(())
    }

    pub fn list_sessions(&self) -> Result<Vec<Session>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, claude_session_id, started_at, ended_at, summary, num_turns, cost_usd, output_tokens
             FROM sessions ORDER BY started_at DESC",
        )?;
        let rows = stmt
            .query_map([], Session::from_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// Gets the most recent Claude session ID (for resuming a continuation).
    pub fn latest_claude_session_id(&self) -> Result<Option<String>> {
        let sid = self
            .conn
            .query_row(
                "SELECT claude_session_id FROM sessions WHERE claude_session_id IS NOT NULL ORDER BY started_at DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .ok();
        Ok(sid)
    }

    // ---- Pending actions --------------------------------------------------

    pub fn add_pending(
        &self,
        job_id: Option<i64>,
        kind: &str,
        description: &str,
        url: Option<&str>,
    ) -> Result<i64> {
        self.add_pending_full(job_id, kind, description, url, None)
    }

    /// Same as [`add_pending`], with `field_key` (used in `answer_needed`).
    pub fn add_pending_full(
        &self,
        job_id: Option<i64>,
        kind: &str,
        description: &str,
        url: Option<&str>,
        field_key: Option<&str>,
    ) -> Result<i64> {
        // `OR IGNORE`: if an equivalent OPEN pending already exists
        // (same kind/url/field_key), don't duplicate — respects `ux_pending_open`.
        self.conn.execute(
            "INSERT OR IGNORE INTO pending_actions (job_id, kind, description, url, field_key, resolved, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6)",
            params![job_id, kind, description, url, field_key, now_rfc3339()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_pending(&self, include_resolved: bool) -> Result<Vec<PendingAction>> {
        let sql = if include_resolved {
            "SELECT id, job_id, kind, description, url, field_key, resolved, created_at
             FROM pending_actions ORDER BY created_at DESC"
        } else {
            "SELECT id, job_id, kind, description, url, field_key, resolved, created_at
             FROM pending_actions WHERE resolved = 0 ORDER BY created_at DESC"
        };
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt
            .query_map([], PendingAction::from_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn resolve_pending(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE pending_actions SET resolved = 1 WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    // ---- Feedback ---------------------------------------------------------

    pub fn add_feedback(&self, summary: &str, suggestions: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO feedback (summary, suggestions, created_at) VALUES (?1, ?2, ?3)",
            params![summary, suggestions, now_rfc3339()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_feedback(&self) -> Result<Vec<Feedback>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, summary, suggestions, created_at FROM feedback ORDER BY created_at DESC",
        )?;
        let rows = stmt
            .query_map([], Feedback::from_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    // ---- CV reviews (ATS tab) --------------------------------------------

    pub fn add_cv_review(&self, score: i64, target: &str, report: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO cv_reviews (score, target, report, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![score, target, report, now_rfc3339()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn latest_cv_review(&self) -> Result<Option<CvReview>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, score, target, report, created_at FROM cv_reviews ORDER BY id DESC LIMIT 1",
        )?;
        Ok(stmt.query_row([], CvReview::from_row).ok())
    }

    pub fn add_cv_version(&self, target: &str, content: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO cv_versions (target, content, created_at) VALUES (?1, ?2, ?3)",
            params![target, content, now_rfc3339()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn latest_cv_version(&self) -> Result<Option<CvVersion>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, target, content, created_at FROM cv_versions ORDER BY id DESC LIMIT 1",
        )?;
        Ok(stmt.query_row([], CvVersion::from_row).ok())
    }

    /// Count of applications per day (key `YYYY-MM-DD` → total).
    pub fn applications_per_day(&self) -> Result<std::collections::HashMap<String, i64>> {
        let mut stmt = self.conn.prepare(
            "SELECT substr(created_at, 1, 10) AS dia, COUNT(*) AS n
             FROM applications GROUP BY dia",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((r.get::<_, String>("dia")?, r.get::<_, i64>("n")?))
        })?;
        let mut map = std::collections::HashMap::new();
        for row in rows {
            let (d, n) = row?;
            map.insert(d, n);
        }
        Ok(map)
    }

    // ---- Stats (dashboard) ------------------------------------------------

    pub fn stats(&self) -> Result<Stats> {
        let total_jobs: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM jobs", [], |r| r.get(0))?;
        let total_applications: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM applications", [], |r| r.get(0))?;
        let applied: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM applications WHERE status = 'applied'",
            [],
            |r| r.get(0),
        )?;
        let pending_actions: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM pending_actions WHERE resolved = 0",
            [],
            |r| r.get(0),
        )?;
        Ok(Stats {
            total_jobs,
            total_applications,
            applied,
            pending_actions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_and_applies_schema() {
        let db = Db::open_in_memory().unwrap();
        // empty profile by default
        assert_eq!(db.get_profile().unwrap(), Profile::default());
    }

    #[test]
    fn profile_upsert() {
        let db = Db::open_in_memory().unwrap();
        db.save_profile("bg", "cv").unwrap();
        let p = db.get_profile().unwrap();
        assert_eq!(p.background, "bg");
        assert_eq!(p.cv_base, "cv");
        // upsert updates, doesn't duplicate
        db.save_profile("bg2", "cv2").unwrap();
        assert_eq!(db.get_profile().unwrap().background, "bg2");
    }

    #[test]
    fn variants_crud() {
        let db = Db::open_in_memory().unwrap();
        let id = db
            .add_variant("Senior Remote", "senior rust remote")
            .unwrap();
        assert_eq!(db.list_variants().unwrap().len(), 1);
        db.set_variant_enabled(id, false).unwrap();
        assert!(!db.list_variants().unwrap()[0].enabled);
        db.delete_variant(id).unwrap();
        assert!(db.list_variants().unwrap().is_empty());
    }

    #[test]
    fn job_upsert_by_url_no_duplicate() {
        let db = Db::open_in_memory().unwrap();
        let mut j = NewJob {
            title: "Dev".into(),
            url: "https://x/1".into(),
            ..Default::default()
        };
        let id1 = db.upsert_job(&j).unwrap();
        j.title = "Senior Dev".into();
        let id2 = db.upsert_job(&j).unwrap();
        assert_eq!(id1, id2, "same URL should update, not create");
        let jobs = db.list_jobs().unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].title, "Senior Dev");
    }

    #[test]
    fn application_flow_and_stats() {
        let db = Db::open_in_memory().unwrap();
        let job_id = db
            .upsert_job(&NewJob {
                title: "Dev".into(),
                url: "https://x/2".into(),
                ..Default::default()
            })
            .unwrap();
        db.add_application(job_id, "applied", Some("cv"), Some("cover"))
            .unwrap();
        db.add_pending(
            Some(job_id),
            "captcha",
            "resolve captcha",
            Some("https://x/2"),
        )
        .unwrap();

        let s = db.stats().unwrap();
        assert_eq!(s.total_jobs, 1);
        assert_eq!(s.total_applications, 1);
        assert_eq!(s.applied, 1);
        assert_eq!(s.pending_actions, 1);
    }

    #[test]
    fn pending_resolve() {
        let db = Db::open_in_memory().unwrap();
        let id = db
            .add_pending(None, "login", "log in to linkedin", None)
            .unwrap();
        assert_eq!(db.list_pending(false).unwrap().len(), 1);
        db.resolve_pending(id).unwrap();
        assert!(db.list_pending(false).unwrap().is_empty());
        assert_eq!(db.list_pending(true).unwrap().len(), 1);
    }

    #[test]
    fn pending_no_duplicate_open() {
        let db = Db::open_in_memory().unwrap();
        // Same (kind, url, field_key) re-emitted multiple times → 1 open pending.
        for _ in 0..3 {
            db.add_pending_full(None, "captcha", "captcha on job", Some("https://x/1"), None)
                .unwrap();
        }
        assert_eq!(db.list_pending(false).unwrap().len(), 1);

        // Different kind for same url is NOT considered duplicate.
        db.add_pending_full(None, "login", "log in", Some("https://x/1"), None)
            .unwrap();
        assert_eq!(db.list_pending(false).unwrap().len(), 2);

        // After resolving, an equivalent new pending can reappear.
        let open = db.list_pending(false).unwrap();
        let captcha = open.iter().find(|p| p.kind == "captcha").unwrap();
        db.resolve_pending(captcha.id).unwrap();
        db.add_pending_full(None, "captcha", "captcha again", Some("https://x/1"), None)
            .unwrap();
        assert_eq!(
            db.list_pending(false)
                .unwrap()
                .iter()
                .filter(|p| p.kind == "captcha")
                .count(),
            1
        );
    }

    #[test]
    fn answers_seed_set_and_missing() {
        let db = Db::open_in_memory().unwrap();
        // seed creates all canonical fields (triage + identity), all empty
        let all = db.get_answers().unwrap();
        let total_seeded = ANSWER_FIELDS.len() + IDENTITY_FIELDS.len();
        assert_eq!(all.len(), total_seeded);
        assert_eq!(db.missing_answers().unwrap().len(), total_seeded);
        assert!(db.answers_map().unwrap().is_empty());

        db.set_answer("salary_expectation", "", "R$ 25,000")
            .unwrap();
        assert_eq!(
            db.answers_map().unwrap().get("salary_expectation").unwrap(),
            "R$ 25,000"
        );
        assert_eq!(db.missing_answers().unwrap().len(), total_seeded - 1);

        // new key (non-canonical) also works
        db.set_answer("custom_question", "Question X?", "yes")
            .unwrap();
        assert_eq!(
            db.answers_map().unwrap().get("custom_question").unwrap(),
            "yes"
        );
    }

    #[test]
    fn migrates_legacy_answer_keys_to_english() {
        let db = Db::open_in_memory().unwrap();
        // Simulate an old DB row with a legacy Portuguese key + value.
        db.conn
            .execute(
                "INSERT OR REPLACE INTO answers (key, label, value, updated_at)
                 VALUES ('pretensao_salarial', 'Pretensão salarial', 'R$ 30k', ?1)",
                params![now_rfc3339()],
            )
            .unwrap();

        // Re-run the migration (idempotent; also runs on every open).
        db.migrate_answer_keys().unwrap();

        let answers = db.get_answers().unwrap();
        // Legacy key is gone; new key carries the value.
        assert!(!answers.iter().any(|a| a.key == "pretensao_salarial"));
        let migrated = answers
            .iter()
            .find(|a| a.key == "salary_expectation")
            .expect("salary_expectation should exist after migration");
        assert_eq!(migrated.value, "R$ 30k");
        assert_eq!(migrated.label, "Salary expectation");
    }

    #[test]
    fn cv_review_add_latest() {
        let db = Db::open_in_memory().unwrap();
        assert!(db.latest_cv_review().unwrap().is_none());
        db.add_cv_review(70, "general", "rep1").unwrap();
        db.add_cv_review(85, "Eng Manager", "rep2").unwrap();
        let latest = db.latest_cv_review().unwrap().unwrap();
        assert_eq!(latest.score, 85);
        assert_eq!(latest.target, "Eng Manager");
    }

    #[test]
    fn applications_per_day_groups() {
        let db = Db::open_in_memory().unwrap();
        let job = db
            .upsert_job(&NewJob {
                title: "Dev".into(),
                url: "https://x/9".into(),
                ..Default::default()
            })
            .unwrap();
        db.add_application(job, "applied", None, None).unwrap();
        db.add_application(job, "applied", None, None).unwrap();
        let map = db.applications_per_day().unwrap();
        let total: i64 = map.values().sum();
        assert_eq!(total, 2);
        // key is YYYY-MM-DD (10 chars)
        assert!(map.keys().all(|k| k.len() == 10));
    }

    #[test]
    fn session_start_finish() {
        let db = Db::open_in_memory().unwrap();
        let id = db.start_session(Some("claude-abc")).unwrap();
        db.finish_session(id, Some("summary"), Some(3), Some(0.12), Some(420))
            .unwrap();
        let s = &db.list_sessions().unwrap()[0];
        assert_eq!(s.claude_session_id.as_deref(), Some("claude-abc"));
        assert_eq!(s.num_turns, Some(3));
        assert!(s.ended_at.is_some());
    }

    #[test]
    fn latest_claude_session_id() {
        let db = Db::open_in_memory().unwrap();

        // No sessions yet
        assert_eq!(db.latest_claude_session_id().unwrap(), None);

        // Create a session without a claude_session_id (None)
        db.start_session(None).unwrap();

        // Still None (session exists but has no claude_session_id)
        assert_eq!(db.latest_claude_session_id().unwrap(), None);

        // Create a session with a claude_session_id
        db.start_session(Some("sess-1")).unwrap();

        // Now returns that session ID
        assert_eq!(
            db.latest_claude_session_id().unwrap(),
            Some("sess-1".to_string())
        );

        // Create another one with a different ID
        db.start_session(Some("sess-2")).unwrap();

        // Returns the most recent one
        assert_eq!(
            db.latest_claude_session_id().unwrap(),
            Some("sess-2".to_string())
        );
    }

    #[test]
    fn identity_fields_are_seeded() {
        let db = Db::open_in_memory().unwrap();
        let answers = db.get_answers().unwrap();
        for key in ["cpf", "phone", "full_name", "birth_date", "city_state"] {
            let a = answers
                .iter()
                .find(|a| a.key == key)
                .unwrap_or_else(|| panic!("identity field {key} not seeded"));
            assert_eq!(a.value, "", "{key} should start empty");
        }
    }

    #[test]
    fn clear_runs_wipes_execution_data_only() {
        let db = Db::open_in_memory().unwrap();

        // Dados que DEVEM ser preservados.
        db.save_profile("bg", "cv").unwrap();
        db.add_variant("Senior Remote", "senior rust remote").unwrap();
        db.set_answer("english_level", "English level", "advanced").unwrap();
        db.add_cv_review(82, "EM", "report").unwrap();

        // Applied job with screenshot — MUST be preserved.
        let applied_job = db
            .upsert_job(&NewJob {
                title: "Senior Rust".into(),
                url: "https://acme.jobs/applied-1".into(),
                ..Default::default()
            })
            .unwrap();
        let app_id = db.add_application(applied_job, "applied", Some("applied-cv"), Some("applied-cover")).unwrap();
        db.set_application_screenshot(applied_job, "/path/to/screenshot.png").unwrap();

        // Non-applied job with execution data — MUST be deleted.
        let non_applied_job = db
            .upsert_job(&NewJob {
                title: "Rust Eng".into(),
                url: "https://acme.jobs/1".into(),
                ..Default::default()
            })
            .unwrap();
        db.add_application(non_applied_job, "ready", Some("draft-cv"), None).unwrap();
        db.add_pending(Some(non_applied_job), "captcha", "solve", Some("https://acme.jobs/1")).unwrap();
        db.start_session(Some("sess-1")).unwrap();
        db.add_feedback("summary", "suggestions").unwrap();

        db.clear_runs().unwrap();

        // Non-applied execution data: all deleted.
        let non_applied_jobs = db.list_jobs().unwrap();
        assert_eq!(non_applied_jobs.len(), 1, "only applied job should remain");
        assert_eq!(non_applied_jobs[0].url, "https://acme.jobs/applied-1");

        let all_apps = db.list_applications().unwrap();
        assert_eq!(all_apps.len(), 1, "only applied application should remain");
        assert_eq!(all_apps[0].id, app_id);
        assert_eq!(all_apps[0].status, "applied");
        assert_eq!(all_apps[0].screenshot_path, Some("/path/to/screenshot.png".to_string()), "screenshot_path must be preserved");

        assert!(db.list_pending(true).unwrap().is_empty(), "all pending actions deleted");
        assert!(db.list_sessions().unwrap().is_empty(), "all sessions deleted");
        assert!(db.list_feedback().unwrap().is_empty(), "all feedback deleted");

        // Profile and answers preserved.
        assert_eq!(db.get_profile().unwrap().background, "bg");
        assert_eq!(db.list_variants().unwrap().len(), 1);
        let answers = db.get_answers().unwrap();
        assert!(answers.len() > 1);
        assert!(answers.iter().any(|a| a.key == "english_level" && a.value == "advanced"));
        assert!(db.latest_cv_review().unwrap().is_some());

        // Idempotent: running again on a clean db does not fail.
        db.clear_runs().unwrap();
        assert_eq!(db.list_jobs_applied().unwrap().len(), 1);
    }

    #[test]
    fn list_jobs_unapplied_excludes_applied_jobs() {
        let db = Db::open_in_memory().unwrap();

        // Create two jobs
        let job1 = db
            .upsert_job(&NewJob {
                title: "Senior Dev".into(),
                url: "https://jobs.example/1".into(),
                company: "Company A".into(),
                source: "LinkedIn".into(),
                description: "Great opportunity".into(),
                fit_score: Some(0.85),
            })
            .unwrap();

        let job2 = db
            .upsert_job(&NewJob {
                title: "Rust Engineer".into(),
                url: "https://jobs.example/2".into(),
                company: "Company B".into(),
                source: "GitHub".into(),
                description: "Exciting project".into(),
                fit_score: Some(0.92),
            })
            .unwrap();

        // Mark job1 as applied
        db.add_application(job1, "applied", None, None)
            .unwrap();

        // Verify list_jobs still returns both jobs
        let all_jobs = db.list_jobs().unwrap();
        assert_eq!(all_jobs.len(), 2);

        // Verify list_jobs_unapplied returns only job2 (the non-applied one)
        let unapplied = db.list_jobs_unapplied().unwrap();
        assert_eq!(unapplied.len(), 1);
        assert_eq!(unapplied[0].id, job2);
        assert_eq!(unapplied[0].title, "Rust Engineer");
    }

    #[test]
    fn list_jobs_applied_returns_only_applied() {
        let db = Db::open_in_memory().unwrap();

        // Create two jobs
        let job1 = db
            .upsert_job(&NewJob {
                title: "Senior Dev".into(),
                url: "https://jobs.example/1".into(),
                company: "Company A".into(),
                source: "LinkedIn".into(),
                description: "Great opportunity".into(),
                fit_score: Some(0.85),
            })
            .unwrap();

        let job2 = db
            .upsert_job(&NewJob {
                title: "Rust Engineer".into(),
                url: "https://jobs.example/2".into(),
                company: "Company B".into(),
                source: "GitHub".into(),
                description: "Exciting project".into(),
                fit_score: Some(0.92),
            })
            .unwrap();

        // Mark job1 as applied
        db.add_application(job1, "applied", None, None)
            .unwrap();

        // Verify list_jobs_applied returns only job1
        let applied = db.list_jobs_applied().unwrap();
        assert_eq!(applied.len(), 1);
        assert_eq!(applied[0].id, job1);
        assert_eq!(applied[0].title, "Senior Dev");

        // Verify list_jobs_unapplied returns only job2
        let unapplied = db.list_jobs_unapplied().unwrap();
        assert_eq!(unapplied.len(), 1);
        assert_eq!(unapplied[0].id, job2);
    }

    #[test]
    fn application_screenshot_path() {
        let db = Db::open_in_memory().unwrap();
        let job_id = db
            .upsert_job(&NewJob {
                title: "Dev".into(),
                url: "https://x/3".into(),
                ..Default::default()
            })
            .unwrap();
        db.add_application(job_id, "applied", None, None)
            .unwrap();

        // Set screenshot path on the most recent application
        db.set_application_screenshot(job_id, "/tmp/screenshot.png")
            .unwrap();

        // Verify application_for_job returns it
        let app = db.application_for_job(job_id).unwrap().unwrap();
        assert_eq!(app.screenshot_path.as_deref(), Some("/tmp/screenshot.png"));

        // Verify get_application also returns it
        let app2 = db.get_application(app.id).unwrap().unwrap();
        assert_eq!(app2.screenshot_path.as_deref(), Some("/tmp/screenshot.png"));
    }
}
