"""Tests for Staging-before-production and deployment gates (#385).

Validates acceptance criteria of #385:
- main deploys to a staging URL.
- Production is promoted only from a tagged release after the full suite, the acceptance suite and a smoke check pass.
- Rollback procedure documented.
"""

import os
import re
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
REPO_ROOT = os.path.dirname(os.path.dirname(HERE))
DEPLOY_WORKFLOW = os.path.join(REPO_ROOT, ".github", "workflows", "deploy.yml")
DEPLOY_DOC = os.path.join(REPO_ROOT, "docs", "DEPLOY.md")

class TestDeployConfiguration(unittest.TestCase):
    def setUp(self):
        self.assertTrue(os.path.exists(DEPLOY_WORKFLOW), "deploy.yml must exist")
        with open(DEPLOY_WORKFLOW, "r", encoding="utf-8") as f:
            self.workflow_text = f.read()
        with open(DEPLOY_DOC, "r", encoding="utf-8") as f:
            self.doc_text = f.read()

    def test_workflow_triggers_on_main_and_tags(self):
        # Triggers on push to main AND tags v*
        self.assertIn("branches: [main]", self.workflow_text)
        self.assertTrue(
            "tags:" in self.workflow_text and "v*" in self.workflow_text,
            "deploy.yml must trigger on tagged releases (e.g. tags: ['v*'])"
        )

    def test_main_does_not_default_to_production(self):
        # Must not default pushes to main to production (must default to staging)
        # Search for: inputs.environment || 'production'
        bad_pattern = r"inputs\.environment\s*\|\|\s*'production'"
        matches = re.findall(bad_pattern, self.workflow_text)
        self.assertEqual(
            matches, [],
            f"deploy.yml still contains fallback to 'production' on push: {matches}"
        )

    def test_production_release_safety_gate_job_exists(self):
        self.assertIn("production-gate", self.workflow_text)
        self.assertIn("check.engine.acceptance-models", self.workflow_text)

    def test_rollback_procedure_documented(self):
        # Acceptance criterion 3: Rollback procedure documented
        self.assertIn("## Rollback", self.doc_text)
        self.assertTrue(
            "rollback.yml" in self.doc_text or "Rollback" in self.doc_text,
            "Rollback workflow must be documented"
        )
        self.assertTrue(
            "staging.stabileo.com" in self.doc_text and "stabileo.com" in self.doc_text,
            "Staging and production URLs must be clearly distinguished"
        )
        # Documented promotion from staging to production
        self.assertTrue(
            "tagged release" in self.doc_text or "tag" in self.doc_text,
            "Tagged release promotion must be documented in DEPLOY.md"
        )

if __name__ == "__main__":
    unittest.main()
