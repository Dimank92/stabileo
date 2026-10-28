#!/usr/bin/env python3
"""
Test for Issue #393: Footings and load generation (snow, combinations, eccentricity, flexure).
"""
import os
import sys

def test_rust_snow_module_exists():
    path = os.path.join("engine", "src", "loads", "cirsoc104_snow.rs")
    assert os.path.exists(path), f"Missing Rust snow module: {path}"

def test_rust_loads_mod_registers_snow():
    path = os.path.join("engine", "src", "loads", "mod.rs")
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    assert "cirsoc104_snow" in content, "cirsoc104_snow module not declared in engine/src/loads/mod.rs"
    assert "Cirsoc104Snow" in content, "Cirsoc104Snow operation not handled in LoadsRequest"

def test_foundation_check_supports_flexure_and_factored_loads():
    path = os.path.join("engine", "src", "postprocess", "foundation_check.rs")
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    assert "flexure" in content, "foundation_check.rs missing flexure check support"
    assert "check_spread_footing_detailed" in content, "foundation_check.rs missing detailed footing check with flexure"
    assert "factored" in content, "foundation_check.rs missing factored load separation"

def test_ts_snow_delegates_to_engine():
    path = os.path.join("web", "src", "lib", "codes", "cirsoc104", "snow.ts")
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    assert "loadsRequest" in content, "snow.ts must delegate to loadsRequest"
    oracle_path = os.path.join("web", "src", "lib", "codes", "cirsoc104", "__tests__", "snow-oracle.ts")
    assert os.path.exists(oracle_path), f"snow-oracle.ts oracle must exist at {oracle_path}"

def main():
    print("Running test_footings_and_snow_loads...")
    errors = []
    tests = [
        test_rust_snow_module_exists,
        test_rust_loads_mod_registers_snow,
        test_foundation_check_supports_flexure_and_factored_loads,
        test_ts_snow_delegates_to_engine,
    ]
    for t in tests:
        try:
            t()
            print(f"  [PASS] {t.__name__}")
        except Exception as e:
            print(f"  [FAIL] {t.__name__}: {e}")
            errors.append(e)

    if errors:
        print(f"\n{len(errors)} tests failed.")
        sys.exit(1)
    else:
        print("\nAll tests passed!")
        sys.exit(0)

if __name__ == "__main__":
    main()
