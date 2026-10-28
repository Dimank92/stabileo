import json
import os
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent


class TestSolverProfilingHarness(unittest.TestCase):
    def test_harness_script_exists(self):
        script = ROOT / "scripts" / "solver-profile.py"
        self.assertTrue(script.is_file(), f"Harness script missing: {script}")

    def test_baseline_report_exists(self):
        report = ROOT / "docs" / "performance" / "SOLVER_PROFILING.md"
        self.assertTrue(report.is_file(), f"Baseline report missing: {report}")
        content = report.read_text(encoding="utf-8")
        self.assertIn("dominant cost", content.lower())
        self.assertIn("assembly", content.lower())
        self.assertIn("factorization", content.lower())
        self.assertIn("certificates", content.lower())

    def test_harness_quick_run_and_json_structure(self):
        script = ROOT / "scripts" / "solver-profile.py"
        self.assertTrue(script.is_file())
        output_json = ROOT / "solver-profile-test.json"
        if output_json.is_file():
            output_json.unlink()

        try:
            cmd = [sys.executable, str(script), "--quick", "--output", str(output_json)]
            res = subprocess.run(cmd, cwd=str(ROOT), capture_output=True, text=True, timeout=60)
            self.assertEqual(res.returncode, 0, f"Harness quick run failed: {res.stderr}\n{res.stdout}")
            self.assertTrue(output_json.is_file(), "Output summary JSON not created")

            with open(output_json, "r", encoding="utf-8") as f:
                data = json.load(f)

            self.assertIn("models", data)
            self.assertIn("dominant_cost", data)
            self.assertIn("total_memory_peak_mb", data)
            self.assertGreater(len(data["models"]), 0)

            first = data["models"][0]
            self.assertIn("name", first)
            self.assertIn("dofs", first)
            self.assertIn("phases_ms", first)
            self.assertIn("assembly", first["phases_ms"])
            self.assertIn("factorization", first["phases_ms"])
            self.assertIn("solve", first["phases_ms"])
            self.assertIn("certificates", first["phases_ms"])
            self.assertIn("memory_peak_mb", first)
            self.assertIn("iterations", first)
        finally:
            if output_json.is_file():
                output_json.unlink()

    def test_harness_regression_gate(self):
        script = ROOT / "scripts" / "solver-profile.py"
        self.assertTrue(script.is_file())
        # Passing gate with large budget
        res_pass = subprocess.run(
            [sys.executable, str(script), "--quick", "--max-time-ms", "100000"],
            cwd=str(ROOT), capture_output=True, text=True, timeout=60
        )
        self.assertEqual(res_pass.returncode, 0, f"Passing gate failed: {res_pass.stderr}")

        # Failing gate with impossible tight budget (e.g. 0.0001 ms)
        res_fail = subprocess.run(
            [sys.executable, str(script), "--quick", "--max-time-ms", "0.00001"],
            cwd=str(ROOT), capture_output=True, text=True, timeout=60
        )
        self.assertNotEqual(res_fail.returncode, 0, "Failing gate should have non-zero exit code on slowdown regression")

    def test_verification_metrics_dashboard_integration(self):
        vm_script = ROOT / "scripts" / "verification-metrics.py"
        content = vm_script.read_text(encoding="utf-8")
        self.assertIn("solver_profile", content, "verification-metrics.py must support solver_profile metric")


if __name__ == "__main__":
    unittest.main()
