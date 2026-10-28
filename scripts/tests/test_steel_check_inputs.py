import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent


class TestSteelCheckInputs(unittest.TestCase):
    def test_steel_inputs_module_exists(self):
        src = ROOT / "engine" / "src" / "postprocess" / "steel_inputs.rs"
        self.assertTrue(src.is_file(), f"steel_inputs.rs missing: {src}")

    def test_postprocess_mod_exports_steel_inputs(self):
        mod_rs = ROOT / "engine" / "src" / "postprocess" / "mod.rs"
        text = mod_rs.read_text(encoding="utf-8")
        self.assertIn("pub mod steel_inputs;", text)

    def test_steel_inputs_functions(self):
        src = ROOT / "engine" / "src" / "postprocess" / "steel_inputs.rs"
        self.assertTrue(src.is_file(), f"steel_inputs.rs missing: {src}")
        text = src.read_text(encoding="utf-8")
        self.assertIn("pub fn compute_moment_gradient_cb", text)
        self.assertIn("pub fn compute_steel_section_constants", text)
        self.assertIn("pub fn compute_member_unbraced_lengths", text)
        self.assertIn("pub fn evaluate_torsional_buckling_e4", text)
        self.assertIn("pub fn evaluate_flange_local_buckling_f62", text)
        self.assertIn("pub fn compute_effective_net_area_d3", text)
        self.assertIn("pub fn resolve_material_family", text)

    def test_ts_engine_logic_doc_updated(self):
        doc = ROOT / "docs" / "verification" / "TS_ENGINE_LOGIC.md"
        self.assertTrue(doc.is_file(), f"TS_ENGINE_LOGIC.md missing: {doc}")
        text = doc.read_text(encoding="utf-8")
        self.assertIn("steel_inputs.rs", text)


if __name__ == "__main__":
    unittest.main()
