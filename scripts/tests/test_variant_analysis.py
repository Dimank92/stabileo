"""Tests for Issue #448: Variant analysis after every bug.

Enforces:
1. docs/verification/VARIANT_ANALYSIS.md exists and documents the 5 bug classes:
   - frame / rotation / sign conventions
   - unit conversions
   - double counting
   - NaN / degenerate input accepted
   - silent drops
2. Each bug class has at least one class-level automated check in CI with a valid path.
3. docs/WORKFLOW.md mandates variant analysis notes on every bug fix PR.
4. docs/verification/DASHBOARD.md and dashboard.json list bug classes and their checks.
"""

import json
import os
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))


class VariantAnalysisTests(unittest.TestCase):
    def test_variant_analysis_doc_exists_and_covers_classes(self):
        """docs/verification/VARIANT_ANALYSIS.md must document the 5 core bug classes."""
        doc_path = os.path.join(ROOT, "docs", "verification", "VARIANT_ANALYSIS.md")
        self.assertTrue(os.path.exists(doc_path), f"{doc_path} must exist")
        with open(doc_path, "r", encoding="utf-8") as f:
            content = f.read().lower()

        required_classes = [
            "sign conventions",
            "unit conversions",
            "double counting",
            "degenerate input",
            "silent drops",
        ]
        for rc in required_classes:
            self.assertIn(rc, content, f"VARIANT_ANALYSIS.md must discuss class: {rc}")

    def test_workflow_requires_variant_analysis(self):
        """docs/WORKFLOW.md must require variant analysis for bug fixes."""
        workflow_path = os.path.join(ROOT, "docs", "WORKFLOW.md")
        self.assertTrue(os.path.exists(workflow_path), f"{workflow_path} must exist")
        with open(workflow_path, "r", encoding="utf-8") as f:
            content = f.read().lower()
        self.assertIn("variant analysis", content)

    def test_dashboard_json_tracks_variant_classes(self):
        """docs/verification/dashboard.json must track variant analysis bug classes and checks."""
        db_path = os.path.join(ROOT, "docs", "verification", "dashboard.json")
        self.assertTrue(os.path.exists(db_path), f"{db_path} must exist")
        with open(db_path, "r", encoding="utf-8") as f:
            data = json.load(f)

        self.assertIn("variant_analysis", data, "dashboard.json must contain 'variant_analysis'")
        va = data["variant_analysis"]
        self.assertIn("classes", va, "'variant_analysis' must contain 'classes'")
        classes = va["classes"]
        self.assertGreaterEqual(len(classes), 5, "Must track at least 5 bug classes")

        for cls_info in classes:
            self.assertIn("name", cls_info)
            self.assertIn("check_file", cls_info)
            self.assertIn("check_type", cls_info)
            # Ensure the referenced check file exists in the repo
            target_path = os.path.join(ROOT, cls_info["check_file"])
            self.assertTrue(
                os.path.exists(target_path),
                f"Referenced check file {cls_info['check_file']} must exist in repo"
            )

    def test_dashboard_md_mentions_variant_classes(self):
        """docs/verification/DASHBOARD.md must display the variant analysis classes table."""
        dash_md = os.path.join(ROOT, "docs", "verification", "DASHBOARD.md")
        self.assertTrue(os.path.exists(dash_md), f"{dash_md} must exist")
        with open(dash_md, "r", encoding="utf-8") as f:
            content = f.read().lower()
        self.assertIn("variant analysis", content)
        self.assertIn("bug classes", content)


if __name__ == "__main__":
    unittest.main()
