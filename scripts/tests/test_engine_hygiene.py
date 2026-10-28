#!/usr/bin/env python3
"""Verification tests for engine code hygiene (Issue #383)."""

import pathlib
import re
import subprocess
import unittest

ROOT = pathlib.Path(__file__).resolve().parent.parent.parent
WORKSPACE_TOML = ROOT / "Cargo.toml"
ENGINE_LIB_RS = ROOT / "engine" / "src" / "lib.rs"
SUPPLY_CHAIN_YML = ROOT / ".github" / "workflows" / "supply-chain.yml"
DASHBOARD_YML = ROOT / ".github" / "workflows" / "verification-dashboard.yml"
CHECK_SCRIPT = ROOT / "scripts" / "check-engine-hygiene.py"

class TestEngineCodeHygiene(unittest.TestCase):
    def test_hygiene_checker_script_exists_and_passes(self):
        """scripts/check-engine-hygiene.py must exist and exit 0."""
        self.assertTrue(CHECK_SCRIPT.is_file(), f"Missing {CHECK_SCRIPT}")
        res = subprocess.run(["python", str(CHECK_SCRIPT)], capture_output=True, text=True)
        self.assertEqual(res.returncode, 0, f"check-engine-hygiene.py failed:\n{res.stdout}\n{res.stderr}")

    def test_overflow_checks_configured_in_workspace(self):
        """Workspace root Cargo.toml must have overflow-checks = true under release, test, and bench profiles."""
        self.assertTrue(WORKSPACE_TOML.is_file())
        content = WORKSPACE_TOML.read_text(encoding="utf-8")
        self.assertIn("[profile.release]", content)
        self.assertIn("overflow-checks = true", content)

    def test_forbid_unsafe_code_in_engine_lib(self):
        """engine/src/lib.rs must forbid unsafe code."""
        self.assertTrue(ENGINE_LIB_RS.is_file())
        content = ENGINE_LIB_RS.read_text(encoding="utf-8")
        self.assertTrue(
            content.startswith("#![forbid(unsafe_code)]") or "#![forbid(unsafe_code)]" in content[:200],
            "Missing #![forbid(unsafe_code)] at top of engine/src/lib.rs",
        )

    def test_no_unsafe_blocks_in_engine_src(self):
        """Engine source files must contain 0 unsafe blocks."""
        engine_src = ROOT / "engine" / "src"
        unsafe_files = []
        for path in engine_src.rglob("*.rs"):
            txt = path.read_text(encoding="utf-8")
            if re.search(r"\bunsafe\s*\{", txt) or re.search(r"\bunsafe\s+fn\b", txt):
                unsafe_files.append(str(path.relative_to(ROOT)))
        self.assertEqual(unsafe_files, [], f"Found unsafe code in engine: {unsafe_files}")

    def test_cargo_audit_and_deny_in_ci(self):
        """supply-chain.yml must enforce cargo audit and cargo deny."""
        self.assertTrue(SUPPLY_CHAIN_YML.is_file())
        content = SUPPLY_CHAIN_YML.read_text(encoding="utf-8")
        self.assertIn("cargo audit", content)
        self.assertIn("cargo deny", content)

    def test_llvm_cov_configured_in_ci(self):
        """verification-dashboard.yml must configure cargo llvm-cov with branch coverage."""
        self.assertTrue(DASHBOARD_YML.is_file())
        content = DASHBOARD_YML.read_text(encoding="utf-8")
        self.assertIn("cargo-llvm-cov", content)
        self.assertIn("--branch", content)

if __name__ == "__main__":
    unittest.main()

