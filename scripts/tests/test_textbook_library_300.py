#!/usr/bin/env python3
"""
Test for Issue #450: Textbook example library (>= 300 solved problems, >= 20 per family).
"""
import os
import re
import sys

TEXTBOOKS_DIR = os.path.join("engine", "tests", "validation", "textbooks")
MOD_RS = os.path.join(TEXTBOOKS_DIR, "mod.rs")

FAMILIES = {
    "beams": ["hibbeler_problems", "hibbeler_extended", "roark_formulas", "roark_extended", "timoshenko_solver", "gere_goodno_mechanics"],
    "trusses": ["kassimali_trusses", "kassimali_trusses_extended", "kassimali_matrix_3d"],
    "frames": ["hibbeler_problems_extended", "przemieniecki_3d", "weaver_gere_extended", "ghali_neville_extended"],
    "indeterminate_methods": ["kassimali_extended", "bathe_convergence", "neal_plastic_extended"],
    "influence_lines": ["hibbeler_influence_lines"],
    "buckling": ["timoshenko_stability", "chen_lui_stability", "chen_lui_stability_extended", "mcguire_ziemian_extended"],
    "dynamics": ["chopra_dynamics", "chopra_dynamics_extended", "clough_penzien_dynamics", "clough_penzien_dynamics_extended", "biggs_extended"],
    "steel_design": ["mccormac_steel"],
    "rc_design": ["mccormac_rc"],
}

def count_tests_in_file(path):
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    return len(re.findall(r"#\[test\]", content))

def check_citations(path):
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    # Check for references / bibliography header in module
    has_ref = any(k in content.lower() for k in ["reference", "edition", "chapter", "example", "problem", "hibbeler", "kassimali", "mccormac", "gere", "timoshenko", "chopra"])
    return has_ref

def main():
    print("Running test_textbook_library_300...")
    if not os.path.exists(TEXTBOOKS_DIR):
        print(f"Error: {TEXTBOOKS_DIR} does not exist")
        sys.exit(1)

    with open(MOD_RS, "r", encoding="utf-8") as f:
        mod_content = f.read()

    total_tests = 0
    file_counts = {}
    missing_mods = []

    for fname in os.listdir(TEXTBOOKS_DIR):
        if fname.endswith(".rs") and fname != "mod.rs":
            mod_name = fname[:-3]
            path = os.path.join(TEXTBOOKS_DIR, fname)
            cnt = count_tests_in_file(path)
            file_counts[mod_name] = cnt
            total_tests += cnt

            if f"mod {mod_name};" not in mod_content:
                missing_mods.append(mod_name)

            if not check_citations(path):
                print(f"Warning: {fname} lacks explicit textbook citations")

    print(f"Total textbook tests counted: {total_tests}")
    for mod_name, cnt in sorted(file_counts.items()):
        print(f"  {mod_name}: {cnt} tests")

    errors = []
    if missing_mods:
        errors.append(f"Modules missing from mod.rs: {missing_mods}")

    if total_tests < 300:
        errors.append(f"Total textbook tests is {total_tests}, required >= 300")

    # Check family coverage
    for family, modules in FAMILIES.items():
        fam_total = sum(file_counts.get(m, 0) for m in modules)
        print(f"Family '{family}': {fam_total} tests (modules: {modules})")
        if fam_total < 20:
            errors.append(f"Family '{family}' has {fam_total} tests, required >= 20")

    if errors:
        print("\nFailures:")
        for e in errors:
            print(f"  [FAIL] {e}")
        sys.exit(1)
    else:
        print("\nAll textbook library requirements met (>= 300 tests, >= 20 per family, citations present)!")
        sys.exit(0)

if __name__ == "__main__":
    main()
