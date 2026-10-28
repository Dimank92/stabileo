"""Tests for Issue #441: Safety advisories and in-app notices.

Enforces:
1. Every advisory referenced in docs/advisories/README.md exists on disk.
2. Every advisory file has all required sections according to docs/SAFETY_ADVISORIES.md.
3. SA-2026-023 and SA-2026-024 have corresponding in-app notice entries in result-changes.json.
4. docs/WORKFLOW.md release checklist enforces: every safety issue closed by a release has an advisory or recorded reason.
5. docs/SAFETY_ADVISORIES.md documents the opt-in notification subscription channel.
"""

import glob
import json
import os
import re
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
ADVISORIES_DIR = os.path.join(ROOT, "docs", "advisories")
RESULT_CHANGES_PATH = os.path.join(ROOT, "web", "src", "lib", "provenance", "result-changes.json")


class SafetyAdvisoriesTests(unittest.TestCase):
    def test_all_referenced_advisories_exist(self):
        """Every advisory link in docs/advisories/README.md must resolve to an existing file."""
        readme_path = os.path.join(ADVISORIES_DIR, "README.md")
        self.assertTrue(os.path.exists(readme_path), "docs/advisories/README.md must exist")
        with open(readme_path, "r", encoding="utf-8") as f:
            content = f.read()

        links = re.findall(r"\[(SA-2026-\d{3})\]\((SA-2026-\d{3}[^)]*\.md)\)", content)
        self.assertGreaterEqual(len(links), 24, "Must reference at least 24 advisories")
        for sa_id, filename in links:
            target = os.path.join(ADVISORIES_DIR, filename)
            self.assertTrue(os.path.exists(target), f"Advisory file {filename} referenced for {sa_id} must exist")

    def test_advisories_have_required_sections(self):
        """Every advisory file must contain required headings and metadata."""
        pattern = os.path.join(ADVISORIES_DIR, "SA-*.md")
        files = glob.glob(pattern)
        self.assertGreaterEqual(len(files), 24)

        for afile in files:
            fname = os.path.basename(afile)
            with open(afile, "r", encoding="utf-8") as f:
                body = f.read().lower()

            self.assertIn("severity", body, f"{fname} missing severity")
            self.assertIn("status", body, f"{fname} missing status")
            self.assertIn("affected features", body, f"{fname} missing affected features")
            self.assertIn("what was wrong", body, f"{fname} missing what was wrong")
            self.assertTrue(
                "how to tell if" in body and "affected" in body,
                f"{fname} missing affected check instructions"
            )
            self.assertIn("fixed in", body, f"{fname} missing fixed in")
            self.assertIn("workaround", body, f"{fname} missing workaround")

    def test_result_changes_includes_latest_advisories(self):
        """web/src/lib/provenance/result-changes.json must register SA-2026-023 and SA-2026-024."""
        self.assertTrue(os.path.exists(RESULT_CHANGES_PATH), f"{RESULT_CHANGES_PATH} must exist")
        with open(RESULT_CHANGES_PATH, "r", encoding="utf-8") as f:
            data = json.load(f)

        change_ids = {c["id"] for c in data.get("changes", [])}
        self.assertIn("column-moment-sign", change_ids, "result-changes.json must contain column-moment-sign (SA-2026-023)")
        self.assertIn("design-on-unfactored-results", change_ids, "result-changes.json must contain design-on-unfactored-results (SA-2026-024)")

    def test_workflow_release_checklist_advisories(self):
        """docs/WORKFLOW.md must include the release checklist rule for safety advisories."""
        workflow_path = os.path.join(ROOT, "docs", "WORKFLOW.md")
        self.assertTrue(os.path.exists(workflow_path), f"{workflow_path} must exist")
        with open(workflow_path, "r", encoding="utf-8") as f:
            content = f.read().lower()

        self.assertIn("safety issue closed by a release has an advisory or a recorded reason", content)

    def test_opt_in_notification_documented(self):
        """docs/SAFETY_ADVISORIES.md must document the opt-in notification mechanism."""
        sa_doc = os.path.join(ROOT, "docs", "SAFETY_ADVISORIES.md")
        self.assertTrue(os.path.exists(sa_doc), f"{sa_doc} must exist")
        with open(sa_doc, "r", encoding="utf-8") as f:
            content = f.read().lower()

        self.assertIn("opt-in", content)
        self.assertTrue("notification" in content or "notifications" in content)


if __name__ == "__main__":
    unittest.main()
