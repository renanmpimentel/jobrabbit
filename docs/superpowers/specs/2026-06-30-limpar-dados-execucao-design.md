# Design — Limpar dados de execução ("clear runs")

**Data:** 2026-06-30
**Status:** Aprovado

## Objetivo

Permitir limpar os dados de execução do jobRabbit para que cada nova busca de
vagas possa ser tratada do zero. Acionável pela Web UI e pelo CLI.

## Escopo

### Apaga (dados de execução)
- `jobs` — vagas encontradas
- `applications` — candidaturas (cascade via FK ao apagar jobs; também apagadas explicitamente)
- `pending_actions` — pendências de intervenção humana
- `sessions` — sessões do agente
- `feedback` — análises/feedback periódico

### Preserva
- `profile` — perfil do candidato e CV base
- `search_variants` — variantes de busca cadastradas
- `answers` — banco de respostas de triagem
- `cv_reviews` / `cv_versions` — avaliações e reescritas ATS do currículo
  (consideradas dados de currículo, não de uma busca)

## Componentes

### 1. Camada de dados — `src/db/mod.rs`
Novo método `clear_runs()`:
- Executa numa única transação (atômico — ou tudo, ou nada).
- Apaga as 5 tabelas de execução em ordem segura de FK:
  `applications`, `pending_actions`, `sessions`, `feedback`, `jobs`.
- Idempotente: rodar com banco vazio não falha.
- Retorna `Result<()>` (ou uma contagem por tabela, se útil para o resumo do CLE).

### 2. Backend web — `src/web/mod.rs`
- Rota nova: `POST /api/reset-runs` registrada no `router()`.
- Handler `reset_runs` no padrão dos existentes (ex.: `delete_variant`):
  trava `s.db`, chama `clear_runs()`, mapeia erro com `internal`, retorna `ok()`.

### 3. CLI — `src/main.rs`
- Flag `--reset-runs` checada em `async_main`, no mesmo estilo de `--doctor`:
  abre o db (`Db::open(config::db_path()?)`), chama `clear_runs()`, imprime um
  resumo curto (ex.: `cleared: N jobs, M applications, …`) e sai sem iniciar a UI.

### 4. Web UI — `web-ui/src/pages/Config.tsx`
- Card separado "Zona de risco" ao final da página de Config.
- Botão variante `danger` "Limpar dados de execução".
- Ao clicar: `confirm()` nativo de confirmação → `post("/reset-runs")` →
  `invalidate()` para recarregar as telas. Erro tratado com `alert(String(e))`
  no mesmo padrão das outras mutações da página.
- Novas chaves i18n em `web-ui/src/locales/en.json` e `pt-BR.json`:
  label do card, label do botão, hint e texto de confirmação.

## Fluxo de dados

```
UI (botão) ─┐
            ├─→ POST /api/reset-runs → reset_runs → db.clear_runs()  ┐
CLI (--reset-runs) ──────────────────────────────→ db.clear_runs()  ┴─→ transação
                                                                         apaga tabelas
                                                                         de execução
```

Após a limpeza, a próxima busca não deduplica contra vagas antigas (a tabela
`jobs` tem `url UNIQUE`; sem registros antigos, o `upsert_job` reinsere tudo) e
começa de um estado limpo.

## Tratamento de erros
- `clear_runs()` em transação: falha reverte tudo, propaga `anyhow::Error`.
- Web: erro → `internal` → 500 `ApiError`; front mostra `alert`.
- CLI: erro propaga pelo `Result` de `main`, encerrando com mensagem.

## Testes (Rust)
Teste que:
1. Popula `profile`, `search_variants`, `answers`, e as 5 tabelas de execução.
2. Chama `clear_runs()`.
3. Verifica que jobs/applications/pending_actions/sessions/feedback estão vazias.
4. Verifica que profile/variants/answers (e cv_reviews/cv_versions) permanecem.
5. Chama `clear_runs()` de novo num banco já limpo (idempotência) — não falha.

## Fora de escopo
- Reset total (apagar perfil) — não solicitado.
- Limpeza automática antes de cada busca — não solicitado.
- Apagar `cv_reviews`/`cv_versions`.
