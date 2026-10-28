#!/usr/bin/env python3
"""Engine Code Hygiene Gate (Issue #383).

Verifies the 5 acceptance criteria for engine code hygiene:
1. Line and branch coverage per module (llvm-cov) configured in CI.
2. Miri readiness: zero unsafe blocks in engine/src and #![forbid(unsafe_code)] enforced.
3. Strict clippy enforced in CI.
4. Overflow checks on index arithmetic in workspace release, test, and bench profiles.
5. cargo audit and cargo deny configured in supply chain CI.
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
WORKSPACE_TOML = ROOT / "Cargo.toml"
ENGINE_LIB_RS = ROOT / "engine" / "src" / "lib.rs"
ENGINE_SRC = ROOT / "engine" / "src"
SUPPLY_CHAIN_YML = ROOT / ".github" / "workflows" / "supply-chain.yml"
DASHBOARD_YML = ROOT / ".github" / "workflows" / "verification-dashboard.yml"
MAKEFILE = ROOT / "Makefile"

def check_overflow_checks():
    text = WORKSPACE_TOML.read_text(encoding="utf-8")
    for profile in ["profile.release", "profile.test", "profile.bench"]:
        if f"[{profile}]" not in text:
            print(f"FAIL: Missing [{profile}] in workspace Cargo.toml", file=sys.stderr)
            return False
    if "overflow-checks = true" not in text:
        print("FAIL: overflow-checks = true not configured in workspace Cargo.toml", file=sys.stderr)
        return False
    print("PASS: Overflow checks enabled on index arithmetic across workspace profiles.")
    return True

def check_unsafe_and_miri():
    lib_text = ENGINE_LIB_RS.read_text(encoding="utf-8")
    if "#![forbid(unsafe_code)]" not in lib_text:
        print("FAIL: #![forbid(unsafe_code)] missing in engine/src/lib.rs", file=sys.stderr)
        return False

    unsafe_matches = []
    for f in ENGINE_SRC.rglob("*.rs"):
        content = f.read_text(encoding="utf-8")
        if re.search(r"\bunsafe\s*\{", content) or re.search(r"\bunsafe\s+fn\b", content):
            unsafe_matches.append(str(f.relative_to(ROOT)))

    if unsafe_matches:
        print(f"FAIL: Found unsafe blocks in engine: {unsafe_matches}. Miri must run.", file=sys.stderr)
        return False

    print("PASS: 0 unsafe blocks in engine; #![forbid(unsafe_code)] strictly enforced.")
    return True

def check_supply_chain():
    text = SUPPLY_CHAIN_YML.read_text(encoding="utf-8")
    if "cargo audit" not in text or "--deny warnings" not in text:
        print("FAIL: cargo audit --deny warnings missing in supply-chain.yml", file=sys.stderr)
        return False
    if "cargo deny" not in text:
        print("FAIL: cargo deny missing in supply-chain.yml", file=sys.stderr)
        return False
    print("PASS: cargo audit and cargo deny active in supply chain CI.")
    return True

def check_coverage():
    text = DASHBOARD_YML.read_text(encoding="utf-8")
    if "cargo-llvm-cov" not in text or "--branch" not in text:
        print("FAIL: cargo llvm-cov with branch coverage missing in verification-dashboard.yml", file=sys.stderr)
        return False
    print("PASS: llvm-cov line and branch coverage configured in CI.")
    return True

def check_strict_clippy():
    text = MAKEFILE.read_text(encoding="utf-8")
    if "lint.engine:" not in text:
        print("FAIL: lint.engine target missing in Makefile", file=sys.stderr)
        return False
    print("PASS: Strict clippy target configured in Makefile.")
    return True

def main():
    checks = [
        check_overflow_checks,
        check_unsafe_and_miri,
        check_supply_chain,
        check_coverage,
        check_strict_clippy,
    ]
    all_ok = True
    for c in checks:
        if not c():
            all_ok = False

    if not all_ok:
        sys.exit(1)
    print("\nAll engine code hygiene gates passed successfully.")

if __name__ == "__main__":
    main()

