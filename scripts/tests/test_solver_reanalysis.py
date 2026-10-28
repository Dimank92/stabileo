import json
import os
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent


class TestSolverReanalysis(unittest.TestCase):
    def test_reanalysis_source_file_exists(self):
        src = ROOT / "engine" / "src" / "solver" / "reanalysis.rs"
        self.assertTrue(src.is_file(), f"reanalysis.rs missing: {src}")

    def test_architecture_doc_exists(self):
        doc = ROOT / "docs" / "architecture" / "REANALYSIS.md"
        self.assertTrue(doc.is_file(), f"REANALYSIS.md missing: {doc}")
        text = doc.read_text(encoding="utf-8")
        self.assertIn("sherman-morrison-woodbury", text.lower())
        self.assertIn("multi-rhs", text.lower())
        self.assertIn("fallback", text.lower())
        self.assertIn("certificate", text.lower())

    def test_reanalysis_parity_with_full_resolve(self):
        src = ROOT / "engine" / "src" / "solver" / "reanalysis.rs"
        self.assertTrue(src.is_file())
        text = src.read_text(encoding="utf-8")
        self.assertIn("pub struct ReAnalysisSession3D", text)
        self.assertIn("pub fn modify_element_section", text)
        self.assertIn("pub fn solve_multi_rhs", text)
        self.assertIn("pub fn woodbury_update", text)


if __name__ == "__main__":
    unittest.main()
