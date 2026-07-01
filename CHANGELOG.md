# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.0] - 2026-07-01

### Added
- **Vagas tab** (renamed from Candidaturas) with an Available/Applied filter;
  applied jobs show status, a proof screenshot thumbnail, and a manual **tracking
  pipeline** (stage: Applied → In review → Interview → Offer / Rejected) with notes.
- **Continue session**: when the agent is blocked (captcha/login/field), resume the
  previous `claude` session with a typed command after you handle it in Chrome.
- **Work model** setting (Remote/On-site/Hybrid, default Remote) filtering the search.
- **Reliable screenshot proof**: the agent saves the confirmation screenshot as a
  real PNG (base64 → file) associated with the applied job.

### Changed
- **Full UI redesign**: minimal clean SaaS look with a left sidebar and a
  light/dark theme toggle (persisted).
- The UI language now drives the backend locale on load, so the agent's output
  (ATS/search/feedback) always follows the language you see; CV/cover letter follow
  the job's language.
- **Clear execution data** never deletes already-applied jobs (and their proofs).

## [1.0.0] - 2026-06-30

### Added
- **Applications tab**: paste a job link and apply directly to that single vacancy.
  The agent opens the URL, reads the description, **detects its language and
  generates the CV/cover letter/answers in that language**, then applies. The tab
  also shows the application count and the list of applications.
- **Brazilian identity fields** (CPF, mobile with area code, full name, birth
  date, city/state) in the answer bank, editable in Config, with an explicit
  agent policy to fill the user's own provided identity data (never inventing
  document numbers). Birth-date field has a `DD/MM/AAAA` input mask.
- **inHire** ATS detection with a dedicated playbook.
- **Clear execution data**: wipe found jobs, applications, pending actions,
  sessions and feedback (Config "Danger Zone" button and `--reset-runs` CLI flag),
  keeping profile, searches and answers.
- **Screenshot proof**: best-effort screenshot of the submission confirmation,
  stored per application and viewable from the Applications tab.

### Changed
- **ATS evaluation** now targets a minimum score of 90/100, with the improve
  flow iterating (bounded) until the rewritten CV reaches the bar.
- Already-applied jobs no longer appear in the jobs list.
- Session tab auto-scroll reliably follows the live stream and resumes when you
  scroll back to the bottom.

## [0.1.0] - 2026-06-30

First public release. 🐇

jobRabbit auto-applies to jobs from your terminal: it orchestrates the **Claude Code CLI**
(`claude`) which, via the **Claude in Chrome** extension, browses job sites in your real
logged-in Chrome, scores each role's *fit* against your profile, generates a tailored CV /
cover letter, and tries to apply — pausing for you (via a desktop notification) on captchas,
logins, or fields it can't fill.

### Added
- **Two front-ends from a single Rust binary**: a local **web UI** (React + Vite + Tailwind,
  served by an Axum backend, the default) and a classic **TUI** (ratatui/crossterm, `--tui`).
- **Real-browser agent**: spawns `claude` over a PTY, reads `--output-format stream-json`, and
  drives Claude in Chrome (no headless, no Playwright). Agent → app communication uses an NDJSON
  protocol (`job` / `application` / `pending` / `answer` / `feedback` / `profile` / `cv_review`)
  persisted to SQLite.
- **Profile import** from a résumé (PDF / DOCX / TXT) or a LinkedIn URL, headless via
  `--import-cv` / `--import-linkedin` or from the UI.
- **Fit scoring** (0.0–1.0) per job against the profile.
- **ATS-aware playbooks** (Gupy, LinkedIn, Greenhouse, Lever, Workday, generic) with a
  per-locale, user-overridable layout (`<data_dir>/playbooks/<locale>/<slug>.md`).
- **Answer bank**: reusable, English-keyed screening answers used to fill forms; the agent can
  learn new ones during a run.
- **Apply modes**: `review` (prepare → you approve), `autonomous` (auto-apply on high fit),
  `hybrid` (auto above a threshold), plus a global `dry-run`.
- **ATS résumé checker**: scores a CV 0–100 with an actionable report, general or against a
  target job; can also generate an ATS-optimized version (export to PDF/DOCX).
- **Internationalization**: English-first with selectable pt-BR — for the UI *and* the agent.
  Locale-aware prompts (`src/agent/prompts.rs`), playbooks (`src/playbooks/{en,pt-br}/`), and
  web-ui resources (`web-ui/src/locales/{en,pt-BR}.json`). The web language switcher updates the
  backend locale so the agent operates in the chosen language.
- **Linux integrations**: idle detection (`user-idle`), desktop notifications
  (`notify-rust` / D-Bus), keyring (`keyring` v3 / Secret Service).
- **CLI helpers**: `--snapshot` (render TUI screens as text), `--doctor` (environment
  diagnostics), `--selftest-agent` (real end-to-end check through the whole pipeline).
- **Tooling**: Docker-based build (`make` targets; the host needs no Rust/Node), GitHub Actions
  CI (fmt, clippy `-D warnings`, tests, web typecheck/build), and a `build.rs` that keeps a fresh
  clone compiling without a pre-built frontend.
- Project meta: MIT license, bilingual README (EN + pt-BR), `CONTRIBUTING`, `CODE_OF_CONDUCT`,
  `SECURITY`, issue/PR templates.

### Notes
- Existing local databases are migrated automatically on open: legacy Portuguese answer-bank
  keys (e.g. `pretensao_salarial`) become their English equivalents (e.g. `salary_expectation`),
  preserving saved values.

[Unreleased]: https://github.com/renanmpimentel/jobrabbit/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/renanmpimentel/jobrabbit/releases/tag/v0.1.0
