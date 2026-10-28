"""Audit test suite for Issue #451:
Every new check, test or CI gate ships with a negative case proving it can fail.

Runs as part of `python3 -m unittest discover -s scripts/tests -v`.
"""

import os
import sys
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))


class NegativeCaseAuditTests(unittest.TestCase):
    def test_pr_template_requires_negative_case(self):
        """PR template must have a line: 'negative case: <test name>'."""
        tmpl_path = os.path.join(ROOT, ".github", "pull_request_template.md")
        self.assertTrue(os.path.exists(tmpl_path), f"{tmpl_path} must exist")
        with open(tmpl_path, "r", encoding="utf-8") as f:
            content = f.read()
        self.assertIn("negative case:", content.lower())

    def test_workflow_review_checklist_requires_negative_case(self):
        """docs/WORKFLOW.md must include review checklist item for negative case."""
        workflow_path = os.path.join(ROOT, "docs", "WORKFLOW.md")
        self.assertTrue(os.path.exists(workflow_path), f"{workflow_path} must exist")
        with open(workflow_path, "r", encoding="utf-8") as f:
            content = f.read()
        self.assertIn("negative case", content.lower())

    def test_negative_cases_documentation_exists(self):
        """docs/verification/NEGATIVE_CASES.md must document the audit of existing gates."""
        doc_path = os.path.join(ROOT, "docs", "verification", "NEGATIVE_CASES.md")
        self.assertTrue(os.path.exists(doc_path), f"{doc_path} must exist")
        with open(doc_path, "r", encoding="utf-8") as f:
            content = f.read()
        self.assertIn("lean axioms", content.lower())
        self.assertIn("symbolic", content.lower())
        self.assertIn("feature matrix", content.lower())
        self.assertIn("differential", content.lower())
        self.assertIn("verification alert", content.lower())

    def test_lean_axiom_gate_negative_case(self):
        """Lean axiom gate must have an automated test (test-axiom-gate.sh) planting invalid axioms."""
        script_path = os.path.join(ROOT, "verify", "scripts", "test-axiom-gate.sh")
        self.assertTrue(os.path.exists(script_path), f"{script_path} must exist")
        with open(script_path, "r", encoding="utf-8") as f:
            code = f.read()
        self.assertIn("axiom tcbGateProbe", code)
        self.assertIn("@[implemented_by", code)
        self.assertIn("@[extern", code)

    def test_symbolic_check_negative_case(self):
        """tools/symbolic/gen.py must enforce --check flag that exits with code 1 on diff."""
        gen_path = os.path.join(ROOT, "tools", "symbolic", "gen.py")
        self.assertTrue(os.path.exists(gen_path), f"{gen_path} must exist")
        with open(gen_path, "r", encoding="utf-8") as f:
            code = f.read()
        self.assertIn("--check", code)
        self.assertIn("sys.exit(1)", code)

    def test_feature_matrix_negative_case(self):
        """engine/tests/feature_matrix.rs must fail if any ignored/wrong feature is not in KNOWN."""
        fm_path = os.path.join(ROOT, "engine", "tests", "feature_matrix.rs")
        self.assertTrue(os.path.exists(fm_path), f"{fm_path} must exist")
        with open(fm_path, "r", encoding="utf-8") as f:
            code = f.read()
        self.assertIn("IGNORED", code)
        self.assertIn("KNOWN", code)
        self.assertIn("UPDATE_FEATURE_MATRIX", code)

    def test_differential_exit_code_negative_case(self):
        """tools/differential/run.py must exit with code 1 when differences/mismatches occur."""
        diff_path = os.path.join(ROOT, "tools", "differential", "run.py")
        self.assertTrue(os.path.exists(diff_path), f"{diff_path} must exist")
        with open(diff_path, "r", encoding="utf-8") as f:
            code = f.read()
        self.assertIn("1 = a mismatch", code)
        self.assertIn("PROGRAMS", code)

    def test_verification_alert_negative_case(self):
        """.github/scripts/verification-alert.cjs must alert only on bad status and not on green."""
        alert_path = os.path.join(ROOT, ".github", "scripts", "verification-alert.cjs")
        self.assertTrue(os.path.exists(alert_path), f"{alert_path} must exist")
        with open(alert_path, "r", encoding="utf-8") as f:
            code = f.read()
        self.assertIn("failure", code)
        self.assertIn("timed_out", code)
        self.assertIn("verification-failure", code)
        self.assertIn("failed.size === 0", code)


if __name__ == "__main__":
    unittest.main()
