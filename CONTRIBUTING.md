# Contributing to jobRabbit

Thanks for your interest in improving jobRabbit! 🐇 This guide covers how to build, test, and
submit changes.

## Code of conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md). By participating, you agree
to uphold it.

## Ground rules

- **Code is English-only.** Comments, identifiers, doc-comments and commit messages are in English.
  User-facing strings live in the i18n layers (see below) — not hardcoded.
- **Keep the test suite green.** `make test` must pass (and CI runs `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, `cargo test`, plus the web-ui typecheck/build).
- Keep changes focused; one logical change per PR.

## Prerequisites

Everything builds via **Docker** — your host doesn't need Rust or Node.

- [Docker](https://www.docker.com/) (with Compose).
- To *run* the app (not just build) you also need, on the host: an authenticated `claude` CLI,
  Google Chrome + the **Claude in Chrome** extension, and `libxcb1 libxss1 libdbus-1-3`.

## Build & test

```bash
make web-install   # install the web-ui dependencies (first time)
make test          # run the Rust test suite
make build         # debug build
make snapshot      # render the TUI screens as text (no TTY)
make release       # build ./dist/jobrabbit for the host
make fmt           # format the code
```

Web UI:

```bash
cd web-ui
npm install
npm run typecheck
npm run build
npm run dev        # Vite dev server (or `make web-dev`)
```

> **rust-embed note:** the Rust crate embeds `web-ui/dist` at compile time. That folder is
> git-ignored; `build.rs` re-creates it so `cargo build` works on a fresh clone (serving a
> "frontend not embedded" stub) until you run `make web-build` / `make` to produce the real bundle.

## Internationalization (i18n)

jobRabbit is English-first with pt-BR selectable. When adding user-facing text:

- **Rust (TUI / CLI):** add the string in English. Locale-aware content (agent prompts, ATS
  playbooks) lives in `src/locale.rs`, `src/agent/prompts.rs`, and `src/playbooks/{en,pt-br}/`.
- **Web UI:** add a key to **both** `web-ui/src/locales/en.json` and `web-ui/src/locales/pt-BR.json`
  (identical key structure) and reference it via `t("...")`.

New locales are very welcome — mirror the existing `en` / `pt-BR` sets.

New ATS playbooks are also welcome: add `src/playbooks/<locale>/<slug>.md` and wire the slug in
`src/ats.rs`.

## Pull requests

1. Fork and create a feature branch.
2. Make your change with tests; run `make test` and the web build.
3. Open a PR describing **what** and **why**. Fill in the PR template.
4. CI must be green before review.

## Reporting bugs / requesting features

Use the issue templates. For security issues, **do not** open a public issue — see
[SECURITY.md](SECURITY.md).
