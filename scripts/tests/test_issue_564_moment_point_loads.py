import sys
from pathlib import Path

repo_root = Path(__file__).resolve().parent.parent.parent
sys.path.insert(0, str(repo_root / "tools" / "differential"))

from common.model import moment_point_loads

def test_moment_point_loads_folds_point_loads():
    model = {
        "is3d": True,
        "nodes": {
            1: {"id": 1, "x": 0.0, "y": 0.0, "z": 0.0},
            2: {"id": 2, "x": 3.5639, "y": 0.0, "z": 0.0},
        },
        "materials": {
            1: {"id": 1, "e": 210.0, "nu": 0.3},
        },
        "sections": {
            1: {"id": 1, "area": 0.01, "iy": 1e-4, "iz": 1e-4, "j": 1e-4},
        },
        "elements": {
            7: {
                "id": 7,
                "nodeI": 1,
                "nodeJ": 2,
                "type": "frame",
                "materialId": 1,
                "sectionId": 1,
            }
        },
        "loads": [
            {
                "type": "pointOnElement",
                "data": {
                    "elementId": 7,
                    "a": 1.2406794154794913,
                    "py": 16.6524,
                    "pz": -0.1219,
                },
            },
            {
                "type": "pointOnElement",
                "data": {
                    "elementId": 7,
                    "a": 1.2493840776582141,
                    "py": -4.5116,
                    "pz": -57.1761,
                },
            },
        ],
    }

    folded = moment_point_loads(model)

    # 1. Point loads on frame member 7 must be converted to 4 station point loads
    p_loads = [ld for ld in folded["loads"] if ld["type"] == "pointOnElement" and ld["data"]["elementId"] == 7]
    assert len(p_loads) == 4, f"Expected 4 folded point loads, got {len(p_loads)}"

    L = 3.5639
    stations = sorted([ld["data"]["a"] for ld in p_loads])
    expected_stations = [0.2 * L, 0.4 * L, 0.6 * L, 0.8 * L]
    for actual, exp in zip(stations, expected_stations):
        assert abs(actual - exp) < 1e-6, f"Station {actual} != {exp}"

    orig_py = 16.6524 - 4.5116
    orig_pz = -0.1219 - 57.1761
    folded_py = sum(ld["data"].get("py", 0.0) for ld in p_loads)
    folded_pz = sum(ld["data"].get("pz", 0.0) for ld in p_loads)
    assert abs(folded_py - orig_py) < 1e-9, f"Total Py {folded_py} != {orig_py}"
    assert abs(folded_pz - orig_pz) < 1e-9, f"Total Pz {folded_pz} != {orig_pz}"

    orig_m1_y = 16.6524 * 1.2406794154794913 - 4.5116 * 1.2493840776582141
    folded_m1_y = sum(ld["data"].get("py", 0.0) * ld["data"]["a"] for ld in p_loads)
    assert abs(folded_m1_y - orig_m1_y) < 1e-9, f"First moment y {folded_m1_y} != {orig_m1_y}"

    print("test_moment_point_loads_folds_point_loads: PASSED")

if __name__ == "__main__":
    test_moment_point_loads_folds_point_loads()
