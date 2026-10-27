# Project working agreements

- Keep code, comments, commit messages, and release notes in English. Route user-facing UI text through i18n.
- Never add a `Co-Authored-By: Claude` trailer to commits. Never commit secrets; use `.env.example` and scan changes before pushing.
- Run Rust formatting, Clippy, tests, web typecheck, and web build before finishing a code change.
- After every approved PR is merged into `main` and its CI passes, publish a new GitHub Release with a matching Cargo version, tag, and all four platform binaries. The release workflow normally does this automatically; use its `workflow_dispatch` recovery path if a run is missed or interrupted. Approval alone, before merge, is not releasable code.
- Default to the next patch version for each PR. Choose a different SemVer increment explicitly when the change requires it, before the release is prepared.
