import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent


class TestWebGpuIterativeBatchSolver(unittest.TestCase):
    def test_webgpu_module_exists(self):
        src = ROOT / "engine" / "src" / "solver" / "webgpu.rs"
        self.assertTrue(src.is_file(), f"engine/src/solver/webgpu.rs missing: {src}")

    def test_solver_mod_exports_webgpu(self):
        mod_rs = ROOT / "engine" / "src" / "solver" / "mod.rs"
        text = mod_rs.read_text(encoding="utf-8")
        self.assertIn("pub mod webgpu;", text, "solver/mod.rs does not export pub mod webgpu;")

    def test_webgpu_doc_exists(self):
        doc = ROOT / "docs" / "architecture" / "WEBGPU_SOLVER.md"
        self.assertTrue(doc.is_file(), f"docs/architecture/WEBGPU_SOLVER.md missing: {doc}")
        text = doc.read_text(encoding="utf-8")
        self.assertIn("WebGPU", text)
        self.assertIn("Preconditioned Conjugate Gradient", text)
        self.assertIn("automatic cpu fallback", text.lower())
        self.assertIn("batch workloads", text.lower())
        self.assertIn("round-off", text.lower())

    def test_webgpu_types_and_functions(self):
        src = ROOT / "engine" / "src" / "solver" / "webgpu.rs"
        self.assertTrue(src.is_file(), f"webgpu.rs missing: {src}")
        text = src.read_text(encoding="utf-8")
        self.assertIn("pub struct WebGpuSolverOptions", text)
        self.assertIn("pub enum WebGpuExecutionMode", text)
        self.assertIn("pub struct BatchWorkloadConfig", text)
        self.assertIn("pub struct BatchSolveResult", text)
        self.assertIn("pub fn solve_iterative_webgpu", text)
        self.assertIn("pub fn solve_batch_workloads", text)
        self.assertIn("pub fn verify_gpu_cpu_roundoff_parity", text)
        self.assertIn("SPMV_WGSL", text)
        self.assertIn("VECTOR_OPS_WGSL", text)


if __name__ == "__main__":
    unittest.main()

