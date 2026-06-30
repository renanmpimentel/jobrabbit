# Limpar Dados de Execução — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Adicionar a capacidade de limpar os dados de execução do jobRabbit (vagas, candidaturas, pendências, sessões e feedback) via Web UI e CLI, para que cada nova busca comece do zero.

**Architecture:** Um único método de banco `clear_runs()` (transação atômica) é a fonte de verdade. Ele é exposto por uma rota `POST /api/reset-runs` (consumida por um botão "Zona de risco" no Config) e por uma flag CLI `--reset-runs`.

**Tech Stack:** Rust (rusqlite, axum, tokio), React + TypeScript (Vite, react-i18next).

## Global Constraints

- Build/test SOMENTE via Docker: `make test` (fmt + clippy -D warnings + test + web build). Manter tudo verde.
- Clippy com `-D warnings`: zero warnings.
- COMMITS: NUNCA adicionar trailer `Co-Authored-By: Claude` neste repo.
- Preservar sempre: `profile`, `search_variants`, `answers`, `cv_reviews`, `cv_versions`.
- Apagar apenas: `applications`, `pending_actions`, `sessions`, `feedback`, `jobs`.
- i18n: toda string visível usa `t("...")` com chave em `en.json` E `pt-BR.json`.

---

## File Structure

- `src/db/mod.rs` — novo método `clear_runs()` + teste inline no `mod tests`.
- `src/web/mod.rs` — nova rota `POST /api/reset-runs` + handler `reset_runs`.
- `src/main.rs` — nova flag CLI `--reset-runs`.
- `web-ui/src/pages/Config.tsx` — card "Zona de risco" com botão de limpeza.
- `web-ui/src/locales/en.json` — chaves i18n novas.
- `web-ui/src/locales/pt-BR.json` — chaves i18n novas.

---

## Task 1: Método `clear_runs()` no banco

**Files:**
- Modify: `src/db/mod.rs` (adicionar método na `impl Db`, perto de `delete_variant`)
- Test: `src/db/mod.rs` (novo `#[test]` dentro de `mod tests`)

**Interfaces:**
- Produces: `pub fn clear_runs(&self) -> anyhow::Result<()>` — apaga as 5 tabelas de execução numa transação. Consumido pelas Tasks 2 e 3.

- [ ] **Step 1: Escrever o teste que falha**

Adicionar ao final do `mod tests` em `src/db/mod.rs`:

```rust
    #[test]
    fn clear_runs_wipes_execution_data_only() {
        let db = Db::open_in_memory().unwrap();

        // Dados que DEVEM ser preservados.
        db.save_profile("bg", "cv").unwrap();
        db.add_variant("Senior Remote", "senior rust remote").unwrap();
        db.set_answer("english_level", "English level", "advanced").unwrap();
        db.add_cv_review(82, "EM", "report").unwrap();

        // Dados de execução que DEVEM ser apagados.
        let job = db
            .upsert_job(&NewJob {
                title: "Rust Eng".into(),
                url: "https://acme.jobs/1".into(),
                ..Default::default()
            })
            .unwrap();
        db.add_application(job, "applied", None, None).unwrap();
        db.add_pending(Some(job), "captcha", "solve", Some("https://acme.jobs/1")).unwrap();
        db.start_session(Some("sess-1")).unwrap();
        db.add_feedback("summary", "suggestions").unwrap();

        db.clear_runs().unwrap();

        // Execução: tudo vazio.
        assert!(db.list_jobs().unwrap().is_empty());
        assert!(db.list_applications().unwrap().is_empty());
        assert!(db.list_pending(true).unwrap().is_empty());
        assert!(db.list_sessions().unwrap().is_empty());
        assert!(db.list_feedback().unwrap().is_empty());

        // Preservado: intacto.
        assert_eq!(db.get_profile().unwrap().background, "bg");
        assert_eq!(db.list_variants().unwrap().len(), 1);
        assert!(db.latest_cv_review().unwrap().is_some());

        // Idempotente: rodar de novo num banco já limpo não falha.
        db.clear_runs().unwrap();
    }
```

- [ ] **Step 2: Rodar o teste para confirmar que falha**

Run: `make test`
Expected: FALHA na compilação — `no method named clear_runs found for struct Db`.

- [ ] **Step 3: Implementar o método mínimo**

Adicionar na `impl Db` em `src/db/mod.rs`, logo após `delete_variant`:

```rust
    /// Apaga os dados de execução (vagas, candidaturas, pendências, sessões e
    /// feedback), numa transação atômica. Preserva perfil, variantes de busca,
    /// respostas de triagem e dados de currículo (cv_reviews/cv_versions).
    /// Permite recomeçar uma busca do zero. Idempotente.
    pub fn clear_runs(&self) -> Result<()> {
        self.conn.execute_batch(
            "BEGIN;
             DELETE FROM applications;
             DELETE FROM pending_actions;
             DELETE FROM sessions;
             DELETE FROM feedback;
             DELETE FROM jobs;
             COMMIT;",
        )?;
        Ok(())
    }
```

- [ ] **Step 4: Rodar o teste para confirmar que passa**

Run: `make test`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/db/mod.rs
git commit -m "feat(db): add clear_runs to wipe execution data only"
```

---

## Task 2: Rota web `POST /api/reset-runs`

**Files:**
- Modify: `src/web/mod.rs` (registrar rota no `router()`; adicionar handler perto de `delete_variant`)

**Interfaces:**
- Consumes: `db.clear_runs()` (Task 1); helpers existentes `internal`, `ok`, `AppState`, `ApiError`.
- Produces: rota HTTP `POST /api/reset-runs` retornando `{"ok": true}`.

- [ ] **Step 1: Registrar a rota no `router()`**

Junto às outras rotas POST (após `.route("/api/import", post(post_import))`):

```rust
        .route("/api/reset-runs", post(reset_runs))
```

- [ ] **Step 2: Adicionar o handler**

Logo após o handler `delete_variant`:

```rust
/// Apaga os dados de execução (jobs, applications, pending, sessions, feedback),
/// permitindo recomeçar uma busca do zero. Preserva perfil, variantes e respostas.
async fn reset_runs(State(s): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let db = s.db.lock().unwrap();
    db.clear_runs().map_err(internal)?;
    Ok(ok())
}
```

- [ ] **Step 3: Build/test**

Run: `make test`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add src/web/mod.rs
git commit -m "feat(web): add POST /api/reset-runs to clear execution data"
```

---

## Task 3: Flag CLI `--reset-runs`

**Files:**
- Modify: `src/main.rs` (checagem em `async_main`, antes do default web)

**Interfaces:**
- Consumes: `db.clear_runs()` (Task 1); `Db::open`, `config::db_path`.

- [ ] **Step 1: Adicionar o handler da flag**

Em `async_main`, após o bloco do `--selftest-agent` e antes do bloco de import:

```rust
    // Limpa os dados de execução (vagas, candidaturas, pendências, sessões,
    // feedback) e sai. Preserva perfil, variantes de busca e respostas.
    if std::env::args().any(|a| a == "--reset-runs") {
        let db = Db::open(config::db_path()?)?;
        db.clear_runs()?;
        println!("execution data cleared (jobs, applications, pending, sessions, feedback)");
        return Ok(());
    }
```

- [ ] **Step 2: Build/test**

Run: `make test`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat(cli): add --reset-runs flag to clear execution data"
```

---

## Task 4: Botão "Zona de risco" no Config + i18n

**Files:**
- Modify: `web-ui/src/pages/Config.tsx` (novo card antes do botão "Salvar")
- Modify: `web-ui/src/locales/en.json`, `web-ui/src/locales/pt-BR.json`

**Interfaces:**
- Consumes: rota `POST /api/reset-runs` (Task 2); helpers `post`, `useInvalidate`, `Card`, `CardHeader`, `Button`, `Row`.

- [ ] **Step 1: Chaves i18n em `en.json`** (no bloco `config`, após `linkedinUrl`, antes de `save`):

```json
    "dangerZone": "Danger Zone",
    "resetRuns": "Clear execution data",
    "resetRunsHint": "removes found jobs, applications, pending actions, sessions and feedback — keeps your profile, searches and answers",
    "resetRunsConfirm": "Clear all execution data? Found jobs, applications, pending actions, sessions and feedback will be permanently deleted. Your profile, searches and answers are kept.",
```

- [ ] **Step 2: Chaves i18n em `pt-BR.json`** (mesma posição):

```json
    "dangerZone": "Zona de risco",
    "resetRuns": "Limpar dados de execução",
    "resetRunsHint": "remove vagas encontradas, candidaturas, pendências, sessões e feedback — mantém seu perfil, buscas e respostas",
    "resetRunsConfirm": "Limpar todos os dados de execução? Vagas encontradas, candidaturas, pendências, sessões e feedback serão apagados permanentemente. Seu perfil, buscas e respostas são mantidos.",
```

- [ ] **Step 3: Handler + card no `Config.tsx`**

Handler (após `changeLanguage`, antes do `return`):

```tsx
  const resetRuns = () => {
    if (!confirm(t("config.resetRunsConfirm"))) return;
    post("/reset-runs").then(invalidate).catch((e) => alert(String(e)));
  };
```

Card antes do `<Button variant="primary" onClick={save}>`:

```tsx
      <Card>
        <CardHeader title={t("config.dangerZone")} />
        <div className="divide-y divide-edge">
          <Row label={t("config.resetRuns")} hint={t("config.resetRunsHint")}>
            <Button variant="danger" onClick={resetRuns}>
              {t("config.resetRuns")}
            </Button>
          </Row>
        </div>
      </Card>
```

- [ ] **Step 4: Build/test**

Run: `make test`
Expected: PASS (inclui web build do Vite).

- [ ] **Step 5: Commit**

```bash
git add web-ui/src/pages/Config.tsx web-ui/src/locales/en.json web-ui/src/locales/pt-BR.json
git commit -m "feat(web-ui): add danger-zone button to clear execution data"
```

---

## Self-Review

- Camada de dados `clear_runs()` → Task 1.
- Backend `POST /api/reset-runs` → Task 2.
- CLI `--reset-runs` → Task 3.
- Web UI botão + i18n (en + pt-BR) → Task 4.
- Testes (popula, limpa, verifica execução vazia + preservação + idempotência) → Task 1.
- Fora de escopo (reset total, auto-limpeza, apagar cv_reviews) — não implementado.
