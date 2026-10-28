"""Tests for scripts/check-numerics-deps.py

Validates acceptance criteria of #388:
- Dependency upgrades (including Dependabot) pass the same required checks as code changes.
- Major upgrades of numerics-related crates require bit-identical results on the golden/certificate corpus.
"""

import os
import unittest
import importlib.util

HERE = os.path.dirname(os.path.abspath(__file__))
SCRIPT_PATH = os.path.join(HERE, "..", "check-numerics-deps.py")

class TestCheckNumericsDeps(unittest.TestCase):
    def setUp(self):
        if not os.path.exists(SCRIPT_PATH):
            self.fail(f"Required script {SCRIPT_PATH} does not exist yet (RED)")
        spec = importlib.util.spec_from_file_location("cnd", SCRIPT_PATH)
        self.cnd = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.cnd)

    def test_script_exists_and_can_be_imported(self):
        self.assertTrue(os.path.exists(SCRIPT_PATH), "check-numerics-deps.py must exist")

    def test_detects_duplicate_numerics_crates(self):
        # A lockfile text with two versions of num-bigint (the #313 bug scenario)
        bad_lock = """
[[package]]
name = "num-bigint"
version = "0.4.8"

[[package]]
name = "num-bigint"
version = "0.5.1"

[[package]]
name = "num-rational"
version = "0.4.2"
"""
        duplicates = self.cnd.find_duplicate_numerics_crates(bad_lock)
        self.assertIn("num-bigint", duplicates)
        self.assertEqual(duplicates["num-bigint"], ["0.4.8", "0.5.1"])

    def test_clean_lockfile_has_no_duplicates(self):
        clean_lock = """
[[package]]
name = "num-bigint"
version = "0.4.8"

[[package]]
name = "num-rational"
version = "0.4.2"

[[package]]
name = "num-traits"
version = "0.2.19"
"""
        duplicates = self.cnd.find_duplicate_numerics_crates(clean_lock)
        self.assertEqual(duplicates, {})

    def test_detects_major_upgrade_of_numerics_crate(self):
        old_crates = {"num-bigint": "0.4.8", "num-rational": "0.4.2"}
        new_crates = {"num-bigint": "0.5.1", "num-rational": "0.4.2"}
        major_bumps = self.cnd.detect_major_numerics_upgrades(old_crates, new_crates)
        self.assertIn("num-bigint", major_bumps)
        self.assertEqual(major_bumps["num-bigint"], ("0.4.8", "0.5.1"))

    def test_minor_upgrade_not_flagged_as_major(self):
        old_crates = {"num-bigint": "0.4.8"}
        new_crates = {"num-bigint": "0.4.9"}
        major_bumps = self.cnd.detect_major_numerics_upgrades(old_crates, new_crates)
        self.assertEqual(major_bumps, {})

    def test_check_current_repo_lockfile_passes(self):
        repo_root = os.path.dirname(os.path.dirname(HERE))
        lock_path = os.path.join(repo_root, "Cargo.lock")
        with open(lock_path, "r", encoding="utf-8") as f:
            lock_content = f.read()
        duplicates = self.cnd.find_duplicate_numerics_crates(lock_content)
        self.assertEqual(duplicates, {}, f"Current repo lockfile has duplicate numerics crates: {duplicates}")

if __name__ == "__main__":
    unittest.main()
