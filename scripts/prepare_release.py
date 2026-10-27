#!/usr/bin/env python3
"""Prepare the next patch release after an approved, merged pull request."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import re
import subprocess
from pathlib import Path


TAG_RE = re.compile(r"^v(\d+)\.(\d+)\.(\d+)$")


def next_patch_tag(tags: list[str]) -> tuple[str, str]:
    versions = [(tuple(map(int, match.groups())), tag) for tag in tags if (match := TAG_RE.fullmatch(tag))]
    if not versions:
        raise ValueError("no semantic-version release tag found")
    (major, minor, patch), previous = max(versions)
    return previous, f"v{major}.{minor}.{patch + 1}"


def update_manifests(cargo_toml: str, cargo_lock: str, previous: str, new: str) -> tuple[str, str]:
    old_version, new_version = previous[1:], new[1:]
    toml_pattern = r'(?m)^(version = ")([^"\n]+)(")'
    toml_match = re.search(toml_pattern, cargo_toml)
    if not toml_match or toml_match.group(2) != old_version:
        raise ValueError(f"Cargo.toml package version must be {old_version}")
    cargo_toml = cargo_toml[: toml_match.start(2)] + new_version + cargo_toml[toml_match.end(2) :]

    lock_pattern = r'(?m)^(name = "jobrabbit"\nversion = ")([^"\n]+)(")'
    lock_match = re.search(lock_pattern, cargo_lock)
    if not lock_match or lock_match.group(2) != old_version:
        raise ValueError(f"Cargo.lock jobrabbit version must be {old_version}")
    cargo_lock = cargo_lock[: lock_match.start(2)] + new_version + cargo_lock[lock_match.end(2) :]
    return cargo_toml, cargo_lock


def update_changelog(changelog: str, tag: str, date: str, pr_number: int, title: str, repo: str) -> str:
    header = re.search(r"(?m)^## \[Unreleased\]\s*$", changelog)
    if not header:
        raise ValueError("CHANGELOG.md needs an Unreleased section")
    following = re.search(r"(?m)^## \[\d+\.\d+\.\d+\]", changelog[header.end() :])
    if not following:
        raise ValueError("CHANGELOG.md needs a previous numbered release")
    split = header.end() + following.start()
    unreleased = changelog[header.end() : split].strip()
    title = " ".join(title.split())
    release_body = f"### Merged PR\n- [#{pr_number}](https://github.com/{repo}/pull/{pr_number}): {title}"
    if unreleased:
        release_body = f"{unreleased}\n\n{release_body}"
    changelog = (
        changelog[: header.start()]
        + f"## [Unreleased]\n\n## [{tag[1:]}] - {date}\n\n{release_body}\n\n"
        + changelog[split:]
    )
    compare_link = f"[Unreleased]: https://github.com/{repo}/compare/{tag}...HEAD"
    changelog, count = re.subn(r"(?m)^\[Unreleased\]: .*$", compare_link, changelog, count=1)
    if count != 1:
        raise ValueError("CHANGELOG.md needs an Unreleased compare link")
    changelog = changelog.rstrip() + f"\n[{tag[1:]}]: https://github.com/{repo}/releases/tag/{tag}\n"
    return changelog


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], text=True).strip()


def existing_tag_for_pr(pr_number: int, tags: list[str]) -> str | None:
    marker = f"Release-PR: {pr_number}"
    for tag in tags:
        if git("cat-file", "-t", f"refs/tags/{tag}") != "tag":
            continue
        message = git("for-each-ref", "--format=%(contents)", f"refs/tags/{tag}")
        if marker in message.splitlines():
            return tag
    return None


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pr", type=int, required=True)
    parser.add_argument("--title", required=True)
    parser.add_argument("--repo", default="renanmpimentel/jobrabbit")
    args = parser.parse_args()
    if args.pr < 1:
        parser.error("--pr must be positive")

    tags = git("tag", "--list", "v*").splitlines()
    existing = existing_tag_for_pr(args.pr, tags)
    if existing:
        print(json.dumps({"tag": existing, "created": False}))
        return

    previous, tag = next_patch_tag(tags)
    root = Path(__file__).resolve().parents[1]
    toml_path, lock_path, changelog_path = (root / name for name in ("Cargo.toml", "Cargo.lock", "CHANGELOG.md"))
    toml, lock = update_manifests(toml_path.read_text(), lock_path.read_text(), previous, tag)
    changelog = update_changelog(
        changelog_path.read_text(), tag, dt.datetime.now(dt.timezone.utc).date().isoformat(),
        args.pr, args.title, args.repo,
    )
    toml_path.write_text(toml)
    lock_path.write_text(lock)
    changelog_path.write_text(changelog)
    print(json.dumps({"tag": tag, "created": True}))


if __name__ == "__main__":
    main()
