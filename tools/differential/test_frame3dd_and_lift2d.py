#!/usr/bin/env python3
"""Unit tests for differential frame3dd adapter, lift_2d settlement validation, and gen_models."""

import os
import sys
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "common"))
sys.path.insert(0, os.path.join(HERE, "frame3dd"))
sys.path.insert(0, os.path.join(HERE, "..", "opensees"))

import model as M
import gen_models as GM
import adapter as frame3dd_adapter


class Lift2DSettlementTests(unittest.TestCase):
    def _base_2d_model(self, sup_type, **settlements):
        return {
            "nodes": {
                "1": {"id": 1, "x": 0.0, "z": 0.0},
                "2": {"id": 2, "x": 4.0, "z": 0.0}
            },
            "materials": {"1": {"id": 1, "e": 200000.0, "nu": 0.3}},
            "sections": {"1": {"id": 1, "a": 0.01, "iz": 1e-4}},
            "elements": {
                "1": {"id": 1, "type": "frame", "nodeI": 1, "nodeJ": 2, "materialId": 1, "sectionId": 1}
            },
            "supports": {
                "1": {"id": 1, "nodeId": 1, "type": sup_type, **settlements}
            },
            "loads": []
        }

    def test_lift_2d_rejects_unrestrained_dz_on_guided_z(self):
        # guidedZ restrains ux (x) and ry, but NOT uz (z)
        m = self._base_2d_model("guidedZ", dz=-0.005)
        with self.assertRaises(M.Unsupported) as ctx:
            M.lift_2d(m)
        self.assertIn("settlement on an unrestrained 2D DOF", str(ctx.exception))

    def test_lift_2d_rejects_unrestrained_dx_on_roller_x(self):
        # rollerX restrains uz (z), but NOT ux (x)
        m = self._base_2d_model("rollerX", dx=0.007)
        with self.assertRaises(M.Unsupported) as ctx:
            M.lift_2d(m)
        self.assertIn("settlement on an unrestrained 2D DOF", str(ctx.exception))

    def test_lift_2d_rejects_unrestrained_dry_on_pinned(self):
        # pinned restrains ux and uz, but NOT ry
        m = self._base_2d_model("pinned", dry=-0.001)
        with self.assertRaises(M.Unsupported) as ctx:
            M.lift_2d(m)
        self.assertIn("settlement on an unrestrained 2D DOF", str(ctx.exception))

    def test_lift_2d_accepts_valid_settlements(self):
        # fixed restrains all DOFs (ux, uz, ry)
        m = self._base_2d_model("fixed", dx=0.002, dz=-0.003, dry=0.0005)
        lifted = M.lift_2d(m)
        self.assertIsNotNone(lifted)
        sup1 = lifted["supports"]["1"]
        self.assertEqual(sup1.get("dx"), 0.002)
        self.assertEqual(sup1.get("dz"), -0.003)
        self.assertEqual(sup1.get("dry"), -0.0005)

    def test_frame3dd_unsupported_flags_unrestrained_settlements(self):
        m = self._base_2d_model("rollerX", dx=0.007)
        why = frame3dd_adapter.unsupported(m, "static")
        self.assertTrue(any("settlement on an unrestrained 2D DOF" in w or "settlement on a free DOF" in w for w in why))

    def test_frame3dd_tol_includes_pdelta(self):
        self.assertTrue(hasattr(frame3dd_adapter, "TOL"))
        self.assertIn("pdelta", frame3dd_adapter.TOL)
        pdelta_tol, _ = frame3dd_adapter.TOL["pdelta"]
        self.assertGreaterEqual(pdelta_tol, 2e-3)

    def test_rich_2d_only_generates_settlements_on_restrained_dofs(self):
        sup_restraints = {
            "fixed": (True, True, True),
            "pinned": (True, True, False),
            "rollerX": (False, True, False),
            "rollerZ": (True, False, False),
            "guidedX": (False, True, True),
            "guidedZ": (True, False, True),
        }
        for seed in range(50):
            m = GM.rich_2d(seed)
            for s in m["supports"].values():
                stype = s.get("type")
                if stype in sup_restraints:
                    rx, rz, rry = sup_restraints[stype]
                    if s.get("dx"):
                        self.assertTrue(rx, f"Seed {seed}: dx generated on unrestrained {stype}")
                    if s.get("dz"):
                        self.assertTrue(rz, f"Seed {seed}: dz generated on unrestrained {stype}")
                    if s.get("dry"):
                        self.assertTrue(rry, f"Seed {seed}: dry generated on unrestrained {stype}")


if __name__ == "__main__":
    unittest.main()
