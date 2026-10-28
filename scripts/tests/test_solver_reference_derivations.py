#!/usr/bin/env python3
"""Verification test for element derivations and independent peer review record in SOLVER_REFERENCE.md (Issue #384)."""

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parent.parent.parent
SOLVER_REF_PATH = ROOT / "docs" / "SOLVER_REFERENCE.md"

REQUIRED_ELEMENT_FORMULATIONS = [
    ("Frame 2D and 3D", ["Euler-Bernoulli", "Timoshenko", "shear deformation", "Hermitian"]),
    ("Truss", ["axial bar", "small strain", "AE/L"]),
    ("Cable", ["catenary", "Ernst", "tension-only"]),
    ("Curved Beam", ["curvature", "in-plane", "out-of-plane"]),
    ("Connector and Discrete Spring", ["6-DOF", "penalty", "local axis"]),
    ("DKT Plate", ["Discrete Kirchhoff", "Batoz", "drilling"]),
    ("MITC4 Shell", ["Mindlin-Reissner", "tying points", "shear locking", "Bathe"]),
    ("MITC9 Shell", ["9-node", "Bucalem-Bathe", "curved"]),
    ("Curved Shell", ["co-rotational", "director", "invariance"]),
    ("SHB8 Solid-Shell", ["prismatic", "hexahedral", "assumed natural strain", "locking"]),
    ("Fiber Beam-Column", ["distributed plasticity", "uniaxial", "cross-section"]),
]

REQUIRED_REVIEW_COLUMNS = [
    "Issue",
    "Module / Formulation",
    "Finding Description",
    "Severity / Safety",
    "Status / Resolution",
]

class TestSolverReferenceDerivations(unittest.TestCase):
    def setUp(self):
        self.assertTrue(SOLVER_REF_PATH.is_file(), f"Missing {SOLVER_REF_PATH}")
        self.content = SOLVER_REF_PATH.read_text(encoding="utf-8")

    def test_element_derivations_present_in_solver_reference(self):
        """Each element formulation must have documented derivation and assumptions in SOLVER_REFERENCE.md."""
        for formulation_name, keywords in REQUIRED_ELEMENT_FORMULATIONS:
            with self.subTest(formulation=formulation_name):
                self.assertIn(
                    formulation_name.lower(),
                    self.content.lower(),
                    f"Formulation '{formulation_name}' not documented in docs/SOLVER_REFERENCE.md",
                )
                for kw in keywords:
                    self.assertIn(
                        kw.lower(),
                        self.content.lower(),
                        f"Keyword '{kw}' for formulation '{formulation_name}' missing in docs/SOLVER_REFERENCE.md",
                    )

    def test_independent_peer_review_record_present(self):
        """Independent peer review record with findings #406-#433 must be documented in SOLVER_REFERENCE.md."""
        self.assertIn(
            "independent peer review record",
            self.content.lower(),
            "Missing 'Independent Peer Review Record' section in docs/SOLVER_REFERENCE.md",
        )
        for col in REQUIRED_REVIEW_COLUMNS:
            self.assertIn(
                col.lower(),
                self.content.lower(),
                f"Missing column '{col}' in peer review record table in docs/SOLVER_REFERENCE.md",
            )

        # Must record representative findings from #406 to #433
        key_issues = ["#406", "#408", "#409", "#412", "#413", "#414", "#417", "#419", "#420", "#426", "#428", "#430", "#432", "#433"]
        for issue_tag in key_issues:
            self.assertIn(
                issue_tag,
                self.content,
                f"Finding issue {issue_tag} not recorded in Peer Review Record in docs/SOLVER_REFERENCE.md",
            )

if __name__ == "__main__":
    unittest.main()

