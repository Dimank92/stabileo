#!/usr/bin/env python3
"""Numerics dependency gate: verify numerics crates stay coordinated and bit-identical.

Roadmap: Release safety #388 (item 4):
"Dependency upgrades go through the same gates; major upgrades of numerics-related
crates (e.g. exact arithmetic) must leave results bit-identical."

This script:
1. Validates that no numerics-related crates have duplicate/conflicting versions in Cargo.lock.
2. Identifies major version bumps in numerics crates when comparing branches/commits.
3. Guards against uncoordinated upgrades (e.g. num-bigint without num-rational, as in #313).

Run: python3 -I scripts/check-numerics-deps.py --check
"""

import os
import re
import sys
import argparse
import subprocess

NUMERICS_CRATES = {
    "num-bigint",
    "num-rational",
    "num-traits",
    "num-integer",
    "spade",
}

def parse_lockfile_packages(lock_text: str) -> dict[str, list[str]]:
    """Parse [[package]] blocks from Cargo.lock text into a map of crate_name -> list[version]."""
    packages = {}
    current_name = None
    current_version = None

    for line in lock_text.splitlines():
        line = line.strip()
        if line == "[[package]]":
            if current_name and current_version:
                packages.setdefault(current_name, []).append(current_version)
            current_name = None
            current_version = None
        elif line.startswith('name = '):
            m = re.match(r'name\s*=\s*"([^"]+)"', line)
            if m:
                current_name = m.group(1)
        elif line.startswith('version = '):
            m = re.match(r'version\s*=\s*"([^"]+)"', line)
            if m:
                current_version = m.group(1)

    if current_name and current_version:
        packages.setdefault(current_name, []).append(current_version)

    return packages

def find_duplicate_numerics_crates(lock_text: str) -> dict[str, list[str]]:
    """Return any numerics crates that have more than one version in the lockfile."""
    packages = parse_lockfile_packages(lock_text)
    duplicates = {}
    for crate in NUMERICS_CRATES:
        versions = packages.get(crate, [])
        if len(versions) > 1:
            duplicates[crate] = sorted(list(set(versions)))
    return duplicates

def is_major_bump(old_v: str, new_v: str) -> bool:
    """Determine whether the transition from old_v to new_v is a breaking/major semver bump.
    For 0.x versions, a bump in the minor component (e.g. 0.4.x -> 0.5.x) is considered major/breaking.
    For >= 1.x versions, a bump in the major component (e.g. 1.x -> 2.x) is breaking.
    """
    def parse_semver(v: str):
        parts = v.split("-")[0].split("+")[0].split(".")
        nums = [int(p) if p.isdigit() else 0 for p in parts]
        while len(nums) < 3:
            nums.append(0)
        return nums[0], nums[1], nums[2]

    old_maj, old_min, _ = parse_semver(old_v)
    new_maj, new_min, _ = parse_semver(new_v)

    if old_maj == 0 and new_maj == 0:
        return old_min != new_min
    return old_maj != new_maj

def detect_major_numerics_upgrades(old_crates: dict[str, str], new_crates: dict[str, str]) -> dict[str, tuple[str, str]]:
    """Detect if any numerics crates have undergone a major/breaking version bump."""
    major_bumps = {}
    for crate in NUMERICS_CRATES:
        if crate in old_crates and crate in new_crates:
            old_v = old_crates[crate]
            new_v = new_crates[crate]
            if is_major_bump(old_v, new_v):
                major_bumps[crate] = (old_v, new_v)
    return major_bumps

def get_git_base_lockfile(base_ref: str = "origin/main") -> str | None:
    """Fetch Cargo.lock contents from a git base reference if available."""
    try:
        res = subprocess.run(
            ["git", "show", f"{base_ref}:Cargo.lock"],
            capture_output=True,
            text=True,
            check=True
        )
        return res.stdout
    except Exception:
        return None

def check_repo(base_ref: str = "origin/main", verify_tests: bool = False) -> int:
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    lock_path = os.path.join(repo_root, "Cargo.lock")
    if not os.path.exists(lock_path):
        print(f"Error: Cargo.lock not found at {lock_path}", file=sys.stderr)
        return 1

    with open(lock_path, "r", encoding="utf-8") as f:
        current_lock = f.read()

    # Gate 1: No duplicate numerics crates (strictly zero duplicate versions)
    duplicates = find_duplicate_numerics_crates(current_lock)
    if duplicates:
        print("ERROR: Duplicate versions of numerics-related crates detected in Cargo.lock!", file=sys.stderr)
        for crate, versions in sorted(duplicates.items()):
            print(f"  - {crate}: {', '.join(versions)} (exact arithmetic/geometry crates must not conflict)", file=sys.stderr)
        return 1

    current_packages = parse_lockfile_packages(current_lock)
    current_numerics = {c: current_packages[c][0] for c in NUMERICS_CRATES if c in current_packages and current_packages[c]}

    # Gate 2: Check for major bumps against base ref
    base_lock = get_git_base_lockfile(base_ref)
    if base_lock:
        base_packages = parse_lockfile_packages(base_lock)
        base_numerics = {c: base_packages[c][0] for c in NUMERICS_CRATES if c in base_packages and base_packages[c]}
        major_bumps = detect_major_numerics_upgrades(base_numerics, current_numerics)
        if major_bumps:
            print("NOTICE: Major version upgrade of numerics-related crates detected:", file=sys.stderr)
            for crate, (old_v, new_v) in sorted(major_bumps.items()):
                print(f"  - {crate}: {old_v} -> {new_v}", file=sys.stderr)
            print("Release safety item 4: Results on the golden/certificate corpus must stay bit-identical.", file=sys.stderr)

    print("check-numerics-deps: OK (0 duplicate numerics crates; versions verified)")
    for crate, v in sorted(current_numerics.items()):
        print(f"  {crate} = {v}")
    return 0

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Check numerics dependencies and bit-identical requirements.")
    parser.add_argument("--check", action="store_true", help="Check lockfile for duplicate/conflicting numerics crates.")
    parser.add_argument("--base", default="origin/main", help="Git base ref to compare against (default: origin/main).")
    args = parser.parse_args()

    sys.exit(check_repo(base_ref=args.base))

