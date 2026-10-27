"""Regression checks for release version and changelog preparation."""

import importlib.util
import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "prepare_release.py"
SPEC = importlib.util.spec_from_file_location("prepare_release", SCRIPT)
release = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(release)


class PrepareReleaseTests(unittest.TestCase):
    def test_next_patch_uses_highest_semver_tag(self):
        self.assertEqual(release.next_patch_tag(["v1.4.0", "v1.5.0", "junk"]), ("v1.5.0", "v1.5.1"))

    def test_manifests_change_only_package_version(self):
        toml = '[package]\nname = "jobrabbit"\nversion = "1.5.0"\n\n[dependencies]\nexample = "1.5.0"\n'
        lock = '[[package]]\nname = "jobrabbit"\nversion = "1.5.0"\n\n[[package]]\nname = "other"\nversion = "1.5.0"\n'
        new_toml, new_lock = release.update_manifests(toml, lock, "v1.5.0", "v1.5.1")
        self.assertIn('version = "1.5.1"', new_toml)
        self.assertIn('example = "1.5.0"', new_toml)
        self.assertIn('name = "jobrabbit"\nversion = "1.5.1"', new_lock)
        self.assertIn('name = "other"\nversion = "1.5.0"', new_lock)

    def test_manifest_mismatch_fails_before_release(self):
        with self.assertRaisesRegex(ValueError, "Cargo.toml package version"):
            release.update_manifests('version = "1.4.0"\n', 'name = "jobrabbit"\nversion = "1.5.0"\n', "v1.5.0", "v1.5.1")

    def test_changelog_moves_unreleased_notes_and_records_pr(self):
        source = (
            "# Changelog\n\n## [Unreleased]\n\n### Fixed\n- Existing fix.\n\n"
            "## [1.5.0] - 2026-07-06\n\nPrior release.\n\n"
            "[Unreleased]: https://github.com/renanmpimentel/jobrabbit/compare/v1.5.0...HEAD\n"
        )
        updated = release.update_changelog(source, "v1.5.1", "2026-10-27", 2, "Fix Windows PTY", "renanmpimentel/jobrabbit")
        self.assertIn("## [Unreleased]\n\n## [1.5.1] - 2026-10-27", updated)
        self.assertIn("- Existing fix.", updated)
        self.assertIn("[#{0}](https://github.com/renanmpimentel/jobrabbit/pull/{0}): Fix Windows PTY".format(2), updated)
        self.assertIn("compare/v1.5.1...HEAD", updated)

    def test_existing_pr_tag_is_reused(self):
        with patch.object(release, "git", side_effect=["tag", "Release v1.5.1\nRelease-PR: 2"]):
            self.assertEqual(release.existing_tag_for_pr(2, ["v1.5.1"]), "v1.5.1")

    def test_cli_reuses_tag_after_a_retry(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "scripts").mkdir()
            shutil.copy(SCRIPT, root / "scripts" / "prepare_release.py")
            (root / "Cargo.toml").write_text('[package]\nname = "jobrabbit"\nversion = "1.5.0"\n')
            (root / "Cargo.lock").write_text('[[package]]\nname = "jobrabbit"\nversion = "1.5.0"\n')
            (root / "CHANGELOG.md").write_text(
                "# Changelog\n\n## [Unreleased]\n\n### Fixed\n- Prior fix.\n\n"
                "## [1.5.0] - 2026-07-06\n\nPrior release.\n\n"
                "[Unreleased]: https://github.com/renanmpimentel/jobrabbit/compare/v1.5.0...HEAD\n"
            )

            def run(*args):
                return subprocess.check_output(args, cwd=root, text=True).strip()

            run("git", "init", "-q")
            run("git", "config", "user.name", "Release Test")
            run("git", "config", "user.email", "release-test@example.invalid")
            run("git", "add", ".")
            run("git", "commit", "-qm", "initial")
            run("git", "tag", "v1.5.0")
            command = ("python3", "scripts/prepare_release.py", "--pr", "2", "--title", "Fix PTY")
            self.assertEqual(json.loads(run(*command)), {"tag": "v1.5.1", "created": True})
            run("git", "add", ".")
            run("git", "commit", "-qm", "release")
            run("git", "tag", "-a", "v1.5.1", "-m", "Release v1.5.1", "-m", "Release-PR: 2")
            self.assertEqual(json.loads(run(*command)), {"tag": "v1.5.1", "created": False})
            self.assertIn('version = "1.5.1"', (root / "Cargo.toml").read_text())


if __name__ == "__main__":
    unittest.main()
