import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent


class TestAdaptiveRefinement(unittest.TestCase):
    def test_adaptive_module_exists(self):
        adaptive_src = ROOT / "engine" / "src" / "postprocess" / "discretization" / "adaptive.rs"
        self.assertTrue(adaptive_src.is_file(), f"adaptive.rs missing: {adaptive_src}")

    def test_discretization_mod_exports_adaptive(self):
        mod_rs = ROOT / "engine" / "src" / "postprocess" / "discretization" / "mod.rs"
        text = mod_rs.read_text(encoding="utf-8")
        self.assertIn("pub mod adaptive;", text, "discretization/mod.rs does not export pub mod adaptive;")

    def test_adaptive_doc_exists(self):
        doc = ROOT / "docs" / "architecture" / "ADAPTIVE_REFINEMENT.md"
        self.assertTrue(doc.is_file(), f"docs/architecture/ADAPTIVE_REFINEMENT.md missing: {doc}")
        text = doc.read_text(encoding="utf-8")
        self.assertIn("Adaptive Refinement", text)
        self.assertIn("Scordelis–Lo", text)
        self.assertIn("stopping criteria", text.lower())

    def test_adaptive_types_and_functions(self):
        adaptive_src = ROOT / "engine" / "src" / "postprocess" / "discretization" / "adaptive.rs"
        text = adaptive_src.read_text(encoding="utf-8")
        self.assertIn("pub struct AdaptiveOptions", text)
        self.assertIn("pub struct AdaptiveRefinementResult", text)
        self.assertIn("pub fn run_adaptive_refinement", text)
        self.assertIn("target_error", text)


if __name__ == "__main__":
    unittest.main()
