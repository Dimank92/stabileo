import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent


class TestDifferentiableAdjointSolver(unittest.TestCase):
    def test_adjoint_solver_module_exists(self):
        adjoint_src = ROOT / "engine" / "src" / "solver" / "adjoint.rs"
        self.assertTrue(adjoint_src.is_file(), f"solver/adjoint.rs missing: {adjoint_src}")

    def test_solver_mod_exports_adjoint(self):
        mod_rs = ROOT / "engine" / "src" / "solver" / "mod.rs"
        text = mod_rs.read_text(encoding="utf-8")
        self.assertIn("pub mod adjoint;", text, "solver/mod.rs does not export pub mod adjoint;")

    def test_adjoint_doc_exists(self):
        doc = ROOT / "docs" / "architecture" / "ADJOINT_SENSITIVITIES.md"
        self.assertTrue(doc.is_file(), f"docs/architecture/ADJOINT_SENSITIVITIES.md missing: {doc}")
        text = doc.read_text(encoding="utf-8")
        self.assertIn("Adjoint Sensitivity Analysis", text)
        self.assertIn("Finite Difference", text)

    def test_api_types_declared(self):
        types_file = ROOT / "engine" / "src" / "types" / "output.rs"
        text = types_file.read_text(encoding="utf-8")
        self.assertIn("AdjointSensitivityResult", text)


if __name__ == "__main__":
    unittest.main()
