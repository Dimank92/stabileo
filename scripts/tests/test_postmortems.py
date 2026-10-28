"""Tests for Issue #449: Postmortem for every safety bug (docs/postmortems/).

Enforces that:
1. `docs/postmortems/` exists and contains an index and template.
2. Every safety postmortem includes:
   - Timeline (introduced, released, found, contained, fixed)
   - Root cause
   - Why existing tests, certificates, differential runs and reviews missed it
   - New kind of test or check added
   - Variant analysis (#448)
   - Link to safety advisory
3. Index in `docs/postmortems/README.md` tracks all postmortems and their new test kinds.
4. `docs/WORKFLOW.md` documents the safety postmortem requirement within 10 working days.
"""

import glob
import os
import re
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
POSTMORTEMS_DIR = os.path.join(ROOT, "docs", "postmortems")


class PostmortemTests(unittest.TestCase):
    def test_postmortems_directory_and_template_exist(self):
        """docs/postmortems/ and TEMPLATE.md must exist."""
        self.assertTrue(os.path.isdir(POSTMORTEMS_DIR), f"{POSTMORTEMS_DIR} directory must exist")
        template_path = os.path.join(POSTMORTEMS_DIR, "TEMPLATE.md")
        self.assertTrue(os.path.exists(template_path), f"{template_path} must exist")
        with open(template_path, "r", encoding="utf-8") as f:
            tmpl = f.read().lower()
        self.assertIn("timeline", tmpl)
        self.assertIn("root cause", tmpl)
        self.assertIn("why existing tests missed it", tmpl)
        self.assertIn("new kind of test", tmpl)
        self.assertIn("variant analysis", tmpl)
        self.assertIn("advisory", tmpl)

    def test_index_lists_postmortems_with_test_kinds(self):
        """docs/postmortems/README.md must index postmortems and their added test kinds."""
        readme_path = os.path.join(POSTMORTEMS_DIR, "README.md")
        self.assertTrue(os.path.exists(readme_path), f"{readme_path} must exist")
        with open(readme_path, "r", encoding="utf-8") as f:
            content = f.read()
        self.assertIn("| Date", content)
        self.assertIn("Test kind added", content)
        self.assertIn("Advisory", content)

    def test_postmortems_have_required_sections(self):
        """Every YYYY-MM-DD-*.md postmortem must have all required sections."""
        pattern = os.path.join(POSTMORTEMS_DIR, "20[0-9][0-9]-[0-9][0-9]-[0-9][0-9]-*.md")
        files = glob.glob(pattern)
        self.assertGreater(len(files), 0, "There must be at least one postmortem in docs/postmortems/")

        for pfile in files:
            fname = os.path.basename(pfile)
            with open(pfile, "r", encoding="utf-8") as f:
                body = f.read().lower()
            self.assertIn("timeline", body, f"{fname} must contain a Timeline section")
            self.assertIn("root cause", body, f"{fname} must contain a Root cause section")
            self.assertTrue(
                "why existing" in body or "why tests missed" in body or "why missed" in body,
                f"{fname} must explain why existing checks/tests/reviews missed it"
            )
            self.assertTrue(
                "new test" in body or "new kind of test" in body or "check added" in body,
                f"{fname} must document the new test kind added"
            )
            self.assertIn("variant analysis", body, f"{fname} must include variant analysis")
            self.assertIn("advisories/", body, f"{fname} must link to its advisory")

    def test_workflow_documents_postmortem_sla(self):
        """docs/WORKFLOW.md must document the 10-day postmortem SLA for safety defects."""
        workflow_path = os.path.join(ROOT, "docs", "WORKFLOW.md")
        self.assertTrue(os.path.exists(workflow_path), f"{workflow_path} must exist")
        with open(workflow_path, "r", encoding="utf-8") as f:
            content = f.read().lower()
        self.assertIn("postmortem", content)
        self.assertIn("safety", content)


if __name__ == "__main__":
    unittest.main()
