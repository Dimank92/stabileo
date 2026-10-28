import os
import re
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent


class TestEngineCrossPlatform(unittest.TestCase):
    def setUp(self):
        self.cargo_toml = REPO_ROOT / "engine" / "Cargo.toml"
        self.binary_src = REPO_ROOT / "engine" / "src" / "bin" / "stabileo_engine.rs"
        self.doc_path = REPO_ROOT / "docs" / "architecture" / "SOLVER_RUNTIME.md"
        self.ci_path = REPO_ROOT / ".github" / "workflows" / "ci.yml"
        self.test_src = REPO_ROOT / "engine" / "tests" / "cross_platform_determinism.rs"

    def test_cargo_toml_declares_binary(self):
        """engine/Cargo.toml must declare the stabileo-engine binary target."""
        content = self.cargo_toml.read_text(encoding="utf-8")
        self.assertIn('[[bin]]', content)
        self.assertIn('name = "stabileo-engine"', content)

    def test_binary_source_exists(self):
        """engine/src/bin/stabileo_engine.rs must exist and provide CLI and server modes."""
        self.assertTrue(self.binary_src.exists(), f"Missing binary source: {self.binary_src}")
        content = self.binary_src.read_text(encoding="utf-8")
        self.assertIn("--solve", content)
        self.assertIn("--server", content)

    def test_solver_runtime_architecture_doc_exists(self):
        """docs/architecture/SOLVER_RUNTIME.md must document the large-model path and browser/desktop/server architecture."""
        self.assertTrue(self.doc_path.exists(), f"Missing architecture documentation: {self.doc_path}")
        content = self.doc_path.read_text(encoding="utf-8")
        self.assertIn("# One Engine Across Browser, Desktop and Server", content)
        self.assertIn("large-model", content.lower())
        self.assertIn("determinism", content.lower())

    def test_ci_builds_native_server_engine(self):
        """.github/workflows/ci.yml must build the native/server engine binary."""
        content = self.ci_path.read_text(encoding="utf-8")
        self.assertIn("stabileo-engine", content)

    def test_cross_platform_determinism_test_exists(self):
        """engine/tests/cross_platform_determinism.rs must exist."""
        self.assertTrue(self.test_src.exists(), f"Missing test source: {self.test_src}")


if __name__ == "__main__":
    unittest.main()
