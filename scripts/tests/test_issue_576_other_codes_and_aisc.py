#!/usr/bin/env python3
"""
TDD RED reproducer test for Issue #576:
- certifyOtherCode drops unevaluated reasons
- AISC 360 CHS shear gap and I-shape flange slenderness
- steel_check.rs & certify/design.rs invalid input handling (Shear G2, LTB F2, Compression E3)
"""
import os
import sys

def test_steel_codes_ts_chs_shear_gap():
    path = os.path.join("web", "src", "lib", "engine", "design", "other-codes", "steel-codes.ts")
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    # In aiscGaps, CHS must not hardcode shear: false
    assert "shear: false" not in content, "steel-codes.ts still hardcodes CHS shear: false in aiscGaps"

def test_steel_codes_ts_flange_slenderness():
    path = os.path.join("web", "src", "lib", "engine", "design", "other-codes", "steel-codes.ts")
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    # AISC360.member must populate flangeSlenderness for I-shapes
    assert "flangeSlenderness" in content, "AISC360.member() does not populate flangeSlenderness"

def test_run_ts_certify_specific_unevaluated_reason():
    path = os.path.join("web", "src", "lib", "engine", "design", "other-codes", "run.ts")
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    # certifyOtherCode must pass m.unevaluated[0] and at.unevaluated[0], not hardcoded 'otherCodes.cert.partial'
    assert "m.unevaluated[0]" in content or "m.unevaluated" in content and "refuse(ctx.elementId, m.unevaluated[0])" in content, "certifyOtherCode does not pass specific m.unevaluated reason"
    assert "at.unevaluated[0]" in content or "at.unevaluated" in content and "refuse(id, at.unevaluated[0])" in content, "certifyOtherCode does not pass specific at.unevaluated reason"

def test_steel_check_rs_invalid_inputs_flagged():
    path = os.path.join("engine", "src", "postprocess", "steel_check.rs")
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    # Shear G2 must use ratio_if_loaded or flag when aw is Some
    assert "ratio_if_loaded" in content and "Shear G2" in content, "steel_check.rs does not use ratio_if_loaded for Shear G2"
    # Compression E3 must refuse when unbraced axis has ry <= 0 or rz <= 0
    assert "ky * m.lby > 0.0 && m.ry <= 0.0" in content or "m.ry <= 0.0" in content and "NAN" in content, "steel_check.rs does not refuse invalid unbraced compression slenderness"
    # LTB F2 must refuse when unbraced beam has iy <= 0 or rz <= 0
    assert "m.iy <= 0.0" in content and "NAN" in content, "steel_check.rs does not refuse invalid unbraced LTB parameters"

def test_certify_design_rs_loaded_ratio_for_shear():
    path = os.path.join("engine", "src", "certify", "design.rs")
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    # certify/design.rs must use loaded_ratio for shear when aw is Some, not branch(zero.lt(phi_vn), ...)
    assert "branch(zero.lt(phi_vn), || vy.div(phi_vn), || zero)" not in content, "certify/design.rs still swallows invalid shear inputs with branch()"

def main():
    print("Running test_issue_576_other_codes_and_aisc...")
    errors = []
    tests = [
        test_steel_codes_ts_chs_shear_gap,
        test_steel_codes_ts_flange_slenderness,
        test_run_ts_certify_specific_unevaluated_reason,
        test_steel_check_rs_invalid_inputs_flagged,
        test_certify_design_rs_loaded_ratio_for_shear,
    ]
    for t in tests:
        try:
            t()
            print(f"  [PASS] {t.__name__}")
        except Exception as e:
            print(f"  [FAIL] {t.__name__}: {e}")
            errors.append((t.__name__, e))

    if errors:
        print(f"\n{len(errors)} tests failed (EXPECTED IN TDD RED PHASE).")
        sys.exit(1)
    else:
        print("\nAll tests passed!")
        sys.exit(0)

if __name__ == "__main__":
    main()
