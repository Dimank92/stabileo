import os
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent


class TestBuildingSubstructuring(unittest.TestCase):
    def setUp(self):
        self.lib_rs = REPO_ROOT / "engine" / "src" / "lib.rs"
        self.doc_path = REPO_ROOT / "docs" / "architecture" / "SUBSTRUCTURING.md"
        self.benchmark_src = (
            REPO_ROOT
            / "engine"
            / "tests"
            / "validation"
            / "benchmarks"
            / "reduction_benchmark.rs"
        )

    def test_lib_rs_exports_3d_reduction(self):
        """engine/src/lib.rs must export guyan_reduce_3d and craig_bampton_3d."""
        content = self.lib_rs.read_text(encoding="utf-8")
        self.assertIn("pub fn guyan_reduce_3d", content)
        self.assertIn("pub fn craig_bampton_3d", content)

    def test_substructuring_architecture_doc_exists(self):
        """docs/architecture/SUBSTRUCTURING.md must document building floor superelements and internal result recovery."""
        self.assertTrue(self.doc_path.exists(), f"Missing doc: {self.doc_path}")
        content = self.doc_path.read_text(encoding="utf-8")
        self.assertIn("# Substructuring and Model Reduction for Buildings", content)
        self.assertIn("superelement", content.lower())
        self.assertIn("craig-bampton", content.lower())
        self.assertIn("internal results", content.lower())

    def test_building_substructuring_benchmarks_exist(self):
        """reduction_benchmark.rs must include multi-storey building floor superelement tests."""
        content = self.benchmark_src.read_text(encoding="utf-8")
        self.assertIn("building_floor_superelement_static_parity", content)
        self.assertIn("building_floor_craig_bampton_dynamics", content)


if __name__ == "__main__":
    unittest.main()
