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

    /// Ensures that the canonical answer fields exist (with empty value).
    fn seed_answers(&self) -> Result<()> {
        for (key, label) in ANSWER_FIELDS {
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
            "SELECT id, job_id, status, cv_generated, cover_letter, created_at
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
            "SELECT id, job_id, status, cv_generated, cover_letter, created_at
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
        // seed creates all canonical fields, all empty
        let all = db.get_answers().unwrap();
        assert_eq!(all.len(), ANSWER_FIELDS.len());
        assert_eq!(db.missing_answers().unwrap().len(), ANSWER_FIELDS.len());
        assert!(db.answers_map().unwrap().is_empty());

        db.set_answer("salary_expectation", "", "R$ 25,000")
            .unwrap();
        assert_eq!(
            db.answers_map().unwrap().get("salary_expectation").unwrap(),
            "R$ 25,000"
        );
        assert_eq!(db.missing_answers().unwrap().len(), ANSWER_FIELDS.len() - 1);

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
}
