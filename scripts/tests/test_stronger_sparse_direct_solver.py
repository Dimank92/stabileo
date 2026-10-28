import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent


class TestStrongerSparseDirectSolver(unittest.TestCase):
    def test_nested_dissection_module_exists(self):
        src = ROOT / "engine" / "src" / "linalg" / "nested_dissection.rs"
        self.assertTrue(src.is_file(), f"nested_dissection.rs missing: {src}")

    def test_linalg_mod_exports_nested_dissection(self):
        mod_rs = ROOT / "engine" / "src" / "linalg" / "mod.rs"
        text = mod_rs.read_text(encoding="utf-8")
        self.assertIn("pub mod nested_dissection;", text)
        self.assertIn("nested_dissection_order", text)

    def test_amg_preconditioner_exists(self):
        src = ROOT / "engine" / "src" / "linalg" / "amg.rs"
        self.assertTrue(src.is_file(), f"amg.rs missing: {src}")
        text = src.read_text(encoding="utf-8")
        self.assertIn("pub struct AmgPreconditioner", text)

    def test_thin_shell_no_dense_fallback_documented(self):
        doc = ROOT / "docs" / "architecture" / "SPARSE_DIRECT_SOLVER.md"
        self.assertTrue(doc.is_file(), f"SPARSE_DIRECT_SOLVER.md missing: {doc}")
        text = doc.read_text(encoding="utf-8")
        self.assertIn("Nested Dissection", text)
        self.assertIn("Algebraic Multigrid", text)
        self.assertIn("Thin-Shell", text)
        self.assertIn("dense fallback", text.lower())

    def test_sparse_chol_has_iterative_refinement(self):
        src = ROOT / "engine" / "src" / "linalg" / "sparse_chol.rs"
        text = src.read_text(encoding="utf-8")
        self.assertIn("sparse_cholesky_solve_with_refinement", text)


if __name__ == "__main__":
    unittest.main()
