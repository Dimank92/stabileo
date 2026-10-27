"""
Regression unit tests for differential model generators (_rich_2d and _rich_3d)
in tools/opensees/gen_models.py.

Verifies conformance with Stabileo AC-10 specification:
- Refuse prescribed displacements on DOFs the support leaves free or spring-softened.
- Never assign transverse member loads (distributed / point-on-element) to truss members.
Addresses #518 (opensees-diff), #516 (codeaster-diff), #515 (kratos).
"""

import sys
import os

# Add repo tools to path
repo_root = os.path.abspath(os.path.join(os.path.dirname(__file__), "../.."))
sys.path.insert(0, repo_root)

from tools.opensees.gen_models import _rich_2d, _rich_3d


def test_rich_2d_settlement_restraints():
    # Seeds that produced false_reject in CI
    failing_seeds = [753002, 753003, 753006, 753007, 753010, 753011, 753014, 753016, 753017, 753019]
    restraint_map = {
        "fixed": (True, True, True),
        "pinned": (True, True, False),
        "rollerX": (False, True, False),
        "rollerZ": (True, False, False),
        "guidedX": (False, True, True),
        "guidedZ": (True, False, True),
    }

    for seed in failing_seeds:
        m = _rich_2d(seed)
        for s_id, s in m["supports"].items():
            stype = s.get("type")
            if stype == "spring":
                for k in ("dx", "dz", "dry"):
                    assert k not in s or s[k] == 0.0, f"Seed {seed}: spring support {s_id} has settlement {k}={s[k]}"
            elif stype in restraint_map:
                rx, rz, rry = restraint_map[stype]
                if not rx:
                    assert "dx" not in s or s["dx"] == 0.0, f"Seed {seed}: support {s_id} ({stype}) unrestrained dx has settlement {s['dx']}"
                if not rz:
                    assert "dz" not in s or s["dz"] == 0.0, f"Seed {seed}: support {s_id} ({stype}) unrestrained dz has settlement {s['dz']}"
                if not rry:
                    assert "dry" not in s or s["dry"] == 0.0, f"Seed {seed}: support {s_id} ({stype}) unrestrained dry has settlement {s['dry']}"


def test_rich_3d_settlement_restraints():
    # Seeds that produced false_reject in CI
    failing_seeds = [753001, 753003, 753006, 753013, 753016, 753020, 753021, 753022, 753034, 753036]
    dof_checks = [
        ("dx", "rx", "kx"),
        ("dy", "ry", "ky"),
        ("dz", "rz", "kz"),
        ("drx", "rrx", "krx"),
        ("dry", "rry", "kry"),
        ("drz", "rrz", "krz"),
    ]

    for seed in failing_seeds:
        m = _rich_3d(seed)
        for s_id, s in m["supports"].items():
            for disp_key, r_flag, k_spring in dof_checks:
                if disp_key in s and s[disp_key] != 0.0:
                    assert s.get(r_flag) is True, (
                        f"Seed {seed}: support {s_id} prescribed {disp_key}={s[disp_key]} on unflagged DOF ({r_flag}=False)"
                    )
                    assert k_spring not in s, (
                        f"Seed {seed}: support {s_id} prescribed {disp_key}={s[disp_key]} on elastic spring DOF ({k_spring}={s[k_spring]})"
                    )


def test_rich_3d_no_member_loads_on_truss():
    # Verify no distributed or point loads on truss members
    for seed in range(753000, 753050):
        m = _rich_3d(seed)
        elems = m["elements"]
        for load in m.get("loads", []):
            if load.get("type") in ("distributed", "pointOnElement"):
                eid = load["data"]["elementId"]
                elem = elems[str(eid)]
                assert elem["type"] != "truss", (
                    f"Seed {seed}: invalid transverse load {load['type']} on truss element {eid}"
                )


if __name__ == "__main__":
    print("Running test_rich_2d_settlement_restraints...")
    test_rich_2d_settlement_restraints()
    print("PASS: test_rich_2d_settlement_restraints")

    print("Running test_rich_3d_settlement_restraints...")
    test_rich_3d_settlement_restraints()
    print("PASS: test_rich_3d_settlement_restraints")

    print("Running test_rich_3d_no_member_loads_on_truss...")
    test_rich_3d_no_member_loads_on_truss()
    print("PASS: test_rich_3d_no_member_loads_on_truss")

    print("ALL TESTS PASSED!")

