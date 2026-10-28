"""Tests for bilingual release notes and changelog validation (#386).

Validates acceptance criteria of #386:
- Every release has a version.
- Release notes in English and Spanish.
- Prominent 'results may change' section listing result-affecting changes and who is affected.
"""

import os
import re
import unittest
import importlib.util

HERE = os.path.dirname(os.path.abspath(__file__))
CHANGELOG_SCRIPT = os.path.join(HERE, "..", "changelog.py")
REPO_ROOT = os.path.dirname(os.path.dirname(HERE))

class TestReleaseNotes(unittest.TestCase):
    def setUp(self):
        spec = importlib.util.spec_from_file_location("cl", CHANGELOG_SCRIPT)
        self.cl = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.cl)

    def test_changelog_module_has_validate_release_notes(self):
        self.assertTrue(
            hasattr(self.cl, "validate_release_notes"),
            "scripts/changelog.py must export validate_release_notes(version) (RED)"
        )

    def test_existing_releases_have_bilingual_notes_and_results_changed_section(self):
        # 0.2.0 and 0.3.0 must pass validation
        for ver in ["0.2.0", "0.3.0"]:
            problems = self.cl.validate_release_notes(ver)
            self.assertEqual(problems, [], f"Release notes validation failed for {ver}: {problems}")

    def test_missing_release_notes_detected(self):
        problems = self.cl.validate_release_notes("9.9.9")
        self.assertTrue(len(problems) >= 2, f"Should detect missing EN and ES notes: {problems}")
        self.assertTrue(any("v9.9.9.md" in p for p in problems))
        self.assertTrue(any("v9.9.9.es.md" in p for p in problems))

    def test_all_versions_in_changelog_are_validated(self):
        changelog_path = os.path.join(REPO_ROOT, "CHANGELOG.md")
        with open(changelog_path, "r", encoding="utf-8") as f:
            text = f.read()
        versions = re.findall(r"^##\s+(\d+\.\d+\.\d+)", text, re.M)
        self.assertTrue(len(versions) >= 2, f"Found versions in CHANGELOG: {versions}")
        for v in versions:
            problems = self.cl.validate_release_notes(v)
            self.assertEqual(problems, [], f"Changelog version {v} failed release notes validation: {problems}")

if __name__ == "__main__":
    unittest.main()
