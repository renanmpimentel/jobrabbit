# ADR 001: Release after each approved merge

Date: 2026-10-27

Status: Accepted

## Context

The project previously built four platform binaries only when someone pushed a `v*` tag. PR #2 was approved and merged without a new release. The package version, changelog, tag, and downloadable binaries must stay aligned for every approved PR.

## Decision

After a successful CI `push` run on `main`, identify the merged PR associated with that commit and require GitHub's final review decision to be `APPROVED`. A manual dispatch accepts an approved, merged PR number for recovery. Serialize release runs, increment the patch version by default, move the unreleased changelog notes into the new version, commit the version change, and push that commit and an annotated tag atomically. The tag records the PR number so a rerun reuses the same release.

The preparation workflow calls the binary release workflow directly. A tag pushed with `GITHUB_TOKEN` does not trigger another `push` workflow. The binary workflow builds Linux, macOS arm64, macOS x86_64, and Windows artifacts, and publishes the GitHub Release only after all four are present. Directly pushed release tags remain supported.

## Consequences

- Approval without merge or a green CI run does not publish code.
- A failed build leaves a tag that can be retried through the manual dispatch without incrementing the version again.
- More than 100 queued merges exceed GitHub Actions' concurrency queue limit and need manual recovery.
- Minor or major releases require an explicit versioning decision before the automatic patch release runs.
