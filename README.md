<div align="center">

# jobRabbit 🐇

**Auto-apply to jobs from your terminal — driven by Claude Code + your real Chrome.**

**English** · [Português 🇧🇷](README.pt-BR.md)

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/React-20232A?style=for-the-badge&logo=react&logoColor=61DAFB)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?style=for-the-badge&logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Vite](https://img.shields.io/badge/Vite-646CFF?style=for-the-badge&logo=vite&logoColor=white)](https://vitejs.dev/)
[![SQLite](https://img.shields.io/badge/SQLite-07405E?style=for-the-badge&logo=sqlite&logoColor=white)](https://www.sqlite.org/)
[![Docker](https://img.shields.io/badge/Docker-2496ED?style=for-the-badge&logo=docker&logoColor=white)](https://www.docker.com/)
[![Linux](https://img.shields.io/badge/Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://www.kernel.org/)
[![Claude](https://img.shields.io/badge/Claude_Code-D97757?style=for-the-badge&logo=anthropic&logoColor=white)](https://claude.com/claude-code)

[![CI](https://img.shields.io/github/actions/workflow/status/renanmpimentel/jobrabbit/ci.yml?branch=main&style=flat-square&label=CI)](https://github.com/renanmpimentel/jobrabbit/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-22c55e?style=flat-square)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-115%20passing-22c55e?style=flat-square)](#-validation)
[![i18n](https://img.shields.io/badge/i18n-EN%20%C2%B7%20pt--BR-3178C6?style=flat-square)](#-languages--i18n)
[![PRs welcome](https://img.shields.io/badge/PRs-welcome-8b5cf6?style=flat-square)](#-contributing)
[![Made with Claude Code](https://img.shields.io/badge/made%20with-Claude%20Code-D97757?style=flat-square&logo=anthropic&logoColor=white)](https://claude.com/claude-code)

</div>

---

jobRabbit orchestrates the **Claude Code CLI** (`claude`) which, through the **Claude in
Chrome** extension, browses job sites in your already-logged-in Chrome, scores each role's
*fit* against your profile, generates a tailored CV / cover letter, and tries to apply —
asking for your help (via an **in-app alert** and a desktop notification) when it hits a
captcha, a login, or a field it can't fill on its own.

It ships **two front-ends from a single Rust binary**: a polished **local web UI** (the
default) and a classic **terminal UI** (`--tui`). Inspired by
[claudia-rh](https://github.com/JohnGabie/claudia-rh) (Windows + Tauri/React); jobRabbit is
a single Rust binary built for Linux.

<div align="center">

<img src="dashboard-light.png" width="49%" alt="Dashboard — light theme"> <img src="dashboard-dark.png" width="49%" alt="Dashboard — dark theme">
<img src="session-light.png" width="49%" alt="Live execution session"> <img src="applications-light.png" width="49%" alt="Applications">

</div>

## ✨ Features

| | |
|---|---|
| 🧭 **Two front-ends** | Local **web UI** (default, opens in your browser) + classic **TUI** (`--tui`) — same engine, same SQLite. |
| 🤖 **Real-browser agent** | Spawns `claude` over a PTY, reads `stream-json`, and drives **Claude in Chrome** (your real, logged-in browser — no headless, no Playwright). |
| 📄 **Profile import** | Build your profile from a **résumé** (PDF / DOCX / TXT) or your **LinkedIn URL** — the agent extracts background, base CV and suggested search variants. |
| 🎯 **Fit scoring** | Each job is scored 0.0–1.0 against your profile (seniority, stack, work model, requirements). |
| 🧩 **ATS-aware playbooks** | Detects **12 platforms** — Gupy, LinkedIn, Greenhouse, Lever, Workday, Ashby, SmartRecruiters, Indeed, Solides, Vagas.com.br, InfoJobs, inHire (+ a generic fallback) — with per-platform recipes so the agent knows how to navigate each site. |
| 🗂️ **Answer bank** | Reusable screening answers (salary expectation, notice period, work model, …) the agent uses to fill forms; learns new ones as it goes. |
| 🌐 **Job sources** | Pick which sites the agent searches — 12 known platforms seeded (LinkedIn, Gupy, Greenhouse, Lever, Workday, Indeed, …), plus add your own — from Config. |
| ⚖️ **Apply modes** | `review` (prepare → you approve), `autonomous` (auto-apply on high fit), `hybrid` (auto above a threshold). Plus a global **dry-run** and a master **human-review** gate (on by default) that always stops for your approval before filling or submitting — even in autonomous/hybrid. |
| 🔔 **Live pending alerts** | When the agent gets blocked (login, captcha, a screening question) you get an instant **in-app toast**, a **banner** on the Execution screen, and a **count badge** on Pending — plus the desktop notification. The Pending screen shows each blocker with the **full job context inline** (fit, company, expandable description). |
| 📎 **Reliable upload** | Résumés are uploaded as a **PDF rendered from your CV**, so uploads never fail because the site can't take a `.docx`. |
| 📊 **ATS résumé checker** | Score your CV 0–100 with an actionable report and a **keyword-gap analysis** (present vs. missing, by importance) you can apply to an improved CV in one click. |
| 🌍 **Bilingual** | **English by default**, **pt-BR** selectable — UI *and* agent language. See [Languages & i18n](#-languages--i18n). |

## 🏗️ Architecture

- **Front-ends** — a local **web UI** (React + Vite + Tailwind, served by an Axum backend) and a **TUI** (ratatui/crossterm). Both compose the same core.
- **Agent** — the app spawns `claude` in a PTY and reads `--output-format stream-json`. The agent emits an **NDJSON protocol** (`job` / `application` / `pending` / `answer` / `feedback` / `profile` / `cv_review`) that the app persists to SQLite. The agent only emits events; the DB is owned by the UI loop.
- **Linux integrations** — idle (`user-idle`), notifications (`notify-rust` / D-Bus), keyring (`keyring` v3 / Secret Service).

## 📋 Prerequisites (host / desktop)

- **Docker** — for building (the host doesn't need Rust or Node).
- To **actually run**: an authenticated `claude` CLI, **Google Chrome** + the **Claude in Chrome** extension, and the libs `libxcb1 libxss1 libdbus-1-3`.

```bash
sudo apt install libxcb1 libxss1 libdbus-1-3
```

## 🚀 Quick start

```bash
make            # default: test → build the web bundle + release binary → open the web UI
```

…or step by step:

```bash
make web-install   # install the web-ui dependencies (web-ui/)
make test          # run the Rust test suite
make release       # produce ./dist/jobrabbit for your HOST
./dist/jobrabbit   # run on your desktop (where claude + Chrome live)
```

By default `./dist/jobrabbit` starts the **web UI** in your browser. Want the classic
terminal UI instead? Run `./dist/jobrabbit --tui`.

## 🛠️ Build & dev (via Docker)

```bash
make build        # compile (debug)
make test         # run the tests
make snapshot     # render the TUI screens as text (no TTY)
make run          # web UI (needs claude + Chrome on the host)
make tui          # classic TUI (needs a TTY)
make web-dev      # Vite dev server (HMR) on :5173, proxies /api → backend
make release      # build ./dist/jobrabbit for the HOST
make fmt          # format the code
make shell        # open a shell in the dev container
```

## 📥 Import your profile (CV or LinkedIn)

Instead of typing your profile by hand, import it from a **résumé** (PDF/DOCX/TXT) or your
**LinkedIn URL** — `claude` structures it into background + base CV + suggested search
variants in the background.

In the **web UI**: the **Profile** page → *Import profile*. In the **TUI**: Profile tab
(`2`) → `m` (CV file) or `l` (LinkedIn URL).

Or headless via the CLI:

```bash
./dist/jobrabbit --import-cv ~/resume.pdf
./dist/jobrabbit --import-linkedin https://www.linkedin.com/in/your-profile
```

> The imported background + CV **replace** the current profile; suggested variants are
> **added** (no duplicates). LinkedIn import browses via your logged-in Chrome.

## 🌍 Languages & i18n

jobRabbit is **English-first** and fully internationalized:

- **Web UI** — switch language from the **Config** page (English · Português). The choice is
  persisted and also updates the **agent** language, so it searches and writes in the same
  language.
- **TUI** — the **Config** tab has a **Language** setting (cycle with `space`).
- **Agent** — prompts and ATS playbooks are locale-aware. With pt-BR selected, the agent
  searches Brazilian boards and writes CVs/cover letters in Portuguese (the original
  behavior); with English, it operates in English.

Translations live in `web-ui/src/locales/{en,pt-BR}.json` and `src/locale.rs` +
`src/playbooks/{en,pt-br}/`. New locales are welcome — see [Contributing](#-contributing).

## ✅ Validation

```bash
make test                              # 115 tests (parser, DB, protocol, prompts, sanitize, TUI)
./dist/jobrabbit --selftest-agent      # real E2E: runs claude (safe prompt, no browsing)
                                       # through the whole pipeline and checks SQLite
./dist/jobrabbit --snapshot            # preview the TUI screens as text (no TTY)
./dist/jobrabbit --doctor              # environment diagnostics (deps + config)
```

## 🧰 Troubleshooting

- **"`claude` not found"** — install Claude Code and authenticate (`claude`). jobRabbit warns at startup and when you try to run the agent.
- **Binary won't start on the host** — install the libs: `sudo apt install libxcb1 libxss1 libdbus-1-3`.
- **TUI has no colors / looks broken** — use a modern terminal (256-color / UTF-8).
- **Notifications / idle don't work** — they need D-Bus and a graphical session (X11/Wayland). They're best-effort; the rest of the app works without them.
- **Data / logs** — live in `~/.local/share/jobrabbit/` (`jobrabbit.db`, `settings.json`, `jobrabbit.log`). Custom playbooks: `~/.local/share/jobrabbit/playbooks/<locale>/<slug>.md`.

## ⚠️ Responsible use

Automating job applications may conflict with a site's Terms of Service. **Check the ToS of
each job site** regarding automation — you are responsible for how you use this tool.
jobRabbit keeps a human in the loop by default (`review` mode) and never bypasses captchas.

## 🤝 Contributing

PRs are welcome! Good first contributions: new ATS playbooks, additional locales, and UI
polish. Please keep the test suite green (`make test`) and the code English-only.

## 📄 License

MIT — see [LICENSE](LICENSE).

<div align="center">
<sub>Built with 🐇 and <a href="https://claude.com/claude-code">Claude Code</a>.</sub>
</div>
