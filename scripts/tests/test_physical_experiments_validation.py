#!/usr/bin/env python3
"""Verification tests for Physical Experiments Validation (Issue #379)."""

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parent.parent.parent
VERIFICATION_DOC = ROOT / "docs" / "VERIFICATION.md"
VALIDATION_MAIN = ROOT / "engine" / "tests" / "validation" / "main.rs"
EXP_TEST_RS = ROOT / "engine" / "tests" / "validation" / "physical_experiments.rs"

REQUIRED_EXPERIMENTAL_DOMAINS = [
    ("RC beams", ["Bresler", "Scordelis", "flexural", "crack"]),
    ("RC columns", ["axial", "cyclic", "ductility", "hinge"]),
    ("Shake-table frames", ["dynamic", "frequency", "period", "hysteretic"]),
    ("Vecchio-Collins panels", ["membrane", "shear", "Toronto", "softening"]),
    ("Connection tests", ["moment", "contact", "slip", "gap"]),
]

REQUIRED_ASME_STANDARDS = [
    "ASME V&V 10",
    "NAFEMS",
    "validation metric",
    "acceptance tolerance",
]

class TestPhysicalExperimentsValidation(unittest.TestCase):
    def test_verification_doc_contains_physical_experiments(self):
        """docs/VERIFICATION.md must contain dedicated section on physical experiment validation."""
        self.assertTrue(VERIFICATION_DOC.is_file())
        content = VERIFICATION_DOC.read_text(encoding="utf-8")
        self.assertIn("validation against physical experiments", content.lower())

        for domain, keywords in REQUIRED_EXPERIMENTAL_DOMAINS:
            with self.subTest(domain=domain):
                self.assertIn(domain.lower(), content.lower())
                for kw in keywords:
                    self.assertIn(kw.lower(), content.lower())

        for std in REQUIRED_ASME_STANDARDS:
            self.assertIn(std.lower(), content.lower())

    def test_validation_rust_suite_registered_and_present(self):
        """physical_experiments.rs must exist and be registered in engine/tests/validation/main.rs."""
        self.assertTrue(VALIDATION_MAIN.is_file())
        main_content = VALIDATION_MAIN.read_text(encoding="utf-8")
        self.assertIn("mod physical_experiments;", main_content)
        self.assertTrue(EXP_TEST_RS.is_file(), f"Missing {EXP_TEST_RS}")

        exp_content = EXP_TEST_RS.read_text(encoding="utf-8")
        for fn_name in [
            "rc_beam_bresler_scordelis",
            "rc_column_cyclic_axial_flexure",
            "shake_table_frame_dynamic_plasticity",
            "vecchio_collins_panel_shear",
            "moment_connection_cyclic_contact",
        ]:
            self.assertIn(fn_name, exp_content)

if __name__ == "__main__":
    unittest.main()

