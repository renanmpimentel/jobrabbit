-- Schema do jobRabbit (SQLite). Idempotente: roda a cada abertura.

-- Perfil do candidato (linha única, id=1).
CREATE TABLE IF NOT EXISTS profile (
    id         INTEGER PRIMARY KEY CHECK (id = 1),
    background TEXT NOT NULL DEFAULT '',
    cv_base    TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL
);

-- Variantes de busca (ex.: "senior remote dev", "tech lead híbrido").
CREATE TABLE IF NOT EXISTS search_variants (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    label      TEXT NOT NULL,
    query      TEXT NOT NULL,
    enabled    INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL
);

-- Vagas encontradas.
CREATE TABLE IF NOT EXISTS jobs (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    title       TEXT NOT NULL,
    company     TEXT NOT NULL DEFAULT '',
    url         TEXT NOT NULL UNIQUE,
    source      TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    fit_score   REAL,
    found_at    TEXT NOT NULL
);

-- Candidaturas (uma vaga pode gerar uma candidatura).
CREATE TABLE IF NOT EXISTS applications (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id          INTEGER NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    status          TEXT NOT NULL DEFAULT 'pending', -- pending|applied|skipped|failed
    cv_generated    TEXT,
    cover_letter    TEXT,
    screenshot_path TEXT,
    stage           TEXT NOT NULL DEFAULT 'applied', -- applied|screening|interview|offer|rejected
    notes           TEXT,
    created_at      TEXT NOT NULL
);

-- Sessões do agente (mapeia para sessões do `claude`).
CREATE TABLE IF NOT EXISTS sessions (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    claude_session_id TEXT,
    started_at        TEXT NOT NULL,
    ended_at          TEXT,
    summary           TEXT,
    num_turns         INTEGER,
    cost_usd          REAL,
    output_tokens     INTEGER
);

-- Ações pendentes que exigem intervenção humana.
CREATE TABLE IF NOT EXISTS pending_actions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id      INTEGER REFERENCES jobs(id) ON DELETE SET NULL,
    kind        TEXT NOT NULL,                 -- captcha|required_field|login|approval|answer_needed
    description TEXT NOT NULL DEFAULT '',
    url         TEXT,
    field_key   TEXT,                          -- p/ answer_needed: chave da resposta
    resolved    INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL
);

-- Migração: deduplica pendências ABERTAS pré-existentes (do bug anterior ao
-- índice abaixo), mantendo a mais antiga de cada grupo. Idempotente — em bancos
-- já limpos não remove nada. PRECISA rodar antes do CREATE UNIQUE INDEX, senão
-- a criação do índice falha em bancos com duplicatas.
DELETE FROM pending_actions
WHERE resolved = 0
  AND id NOT IN (
      SELECT MIN(id)
      FROM pending_actions
      WHERE resolved = 0
      GROUP BY kind, COALESCE(url, ''), COALESCE(field_key, '')
  );

-- Evita pendências abertas duplicadas para a mesma (kind, url, field_key).
-- Parcial em `resolved = 0`: itens já resolvidos não bloqueiam novas pendências.
CREATE UNIQUE INDEX IF NOT EXISTS ux_pending_open
    ON pending_actions (kind, COALESCE(url, ''), COALESCE(field_key, ''))
    WHERE resolved = 0;

-- Banco de respostas de triagem (pretensão, PCD, LGPD, inglês, etc.).
CREATE TABLE IF NOT EXISTS answers (
    key        TEXT PRIMARY KEY,
    label      TEXT NOT NULL,
    value      TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL
);

-- Avaliações ATS do currículo (resume checker).
CREATE TABLE IF NOT EXISTS cv_reviews (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    score      INTEGER NOT NULL,
    target     TEXT NOT NULL DEFAULT '',
    report     TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL
);

-- Versões melhoradas do currículo (reescrita otimizada p/ ATS, p/ preview/download).
CREATE TABLE IF NOT EXISTS cv_versions (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    target     TEXT NOT NULL DEFAULT '',
    content    TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL
);

-- Análises/feedback periódico do agente.
CREATE TABLE IF NOT EXISTS feedback (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    summary     TEXT NOT NULL,
    suggestions TEXT NOT NULL DEFAULT '',
    created_at  TEXT NOT NULL
);
