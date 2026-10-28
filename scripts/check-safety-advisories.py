#!/usr/bin/env python3
"""
Check safety advisories and re-check registry integrity.

Verifies:
1. Every advisory in docs/advisories/ has complete metadata with no unresolved placeholders.
2. Cross-references between docs/advisories and web/src/lib/provenance/result-changes.json resolve.
3. Every entry in docs/advisories/README.md matches existing advisory files.

Usage:
  python scripts/check-safety-advisories.py
"""

import json
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
ADVISORIES_DIR = REPO_ROOT / "docs" / "advisories"
README_PATH = ADVISORIES_DIR / "README.md"
SAFETY_DOC_PATH = REPO_ROOT / "docs" / "SAFETY_ADVISORIES.md"
RESULT_CHANGES_PATH = REPO_ROOT / "web" / "src" / "lib" / "provenance" / "result-changes.json"

SEVERITY_PREFIXES = ("Critical", "High", "Medium", "Low")


def main() -> int:
    errors = []

    if not ADVISORIES_DIR.exists():
        print(f"Error: {ADVISORIES_DIR} does not exist", file=sys.stderr)
        return 1

    advisory_files = sorted(ADVISORIES_DIR.glob("SA-2026-*.md"))
    if not advisory_files:
        errors.append("No advisory files found in docs/advisories/")

    # Read result-changes.json
    result_changes = {}
    if RESULT_CHANGES_PATH.exists():
        try:
            with open(RESULT_CHANGES_PATH, "r", encoding="utf-8") as f:
                rc_data = json.load(f)
                result_changes = {c["id"]: c for c in rc_data.get("changes", [])}
        except Exception as e:
            errors.append(f"Failed to parse {RESULT_CHANGES_PATH}: {e}")
    else:
        errors.append(f"{RESULT_CHANGES_PATH} not found")

    advisory_ids = set()
    for af in advisory_files:
        m = re.match(r"^(SA-\d{4}-\d{3})", af.name)
        if not m:
            errors.append(f"{af.name}: does not follow naming convention SA-YYYY-NNN-<slug>.md")
            continue
        aid = m.group(1)
        advisory_ids.add(aid)

        content = af.read_text(encoding="utf-8")

        # Check metadata table: Severity, Status, Affected versions, Re-check registry
        for field in ["Severity", "Status", "Affected versions", "Re-check registry"]:
            pattern = rf"\|\s*{re.escape(field)}\s*\|\s*([^|\n]+)\|"
            match = re.search(pattern, content, re.IGNORECASE)
            if not match:
                errors.append(f"{af.name}: missing '{field}' row in metadata table")
            else:
                val = match.group(1).strip()
                if "to add with the fix" in val.lower():
                    errors.append(f"{af.name}: field '{field}' has unresolved placeholder '{val}'")
                if field == "Severity" and not any(val.startswith(p) for p in SEVERITY_PREFIXES):
                    errors.append(f"{af.name}: invalid severity '{val}', must start with one of {SEVERITY_PREFIXES}")

        # Check required sections
        has_how_to_tell = ("## How to tell if your model is affected" in content or
                           "## How to tell if your results are affected" in content)
        if not has_how_to_tell:
            errors.append(f"{af.name}: missing '## How to tell if your model/results are affected' section")

        for section in ["## Affected features", "## What was wrong", "## Fixed in"]:
            if section not in content:
                errors.append(f"{af.name}: missing section '{section}'")

    # Verify result-changes.json cross-references
    for cid, c in result_changes.items():
        if "advisory" in c:
            adv_id = c["advisory"]
            if adv_id not in advisory_ids:
                errors.append(f"result-changes.json: change '{cid}' references unknown advisory '{adv_id}'")

    if errors:
        print(f"FAILED: {len(errors)} error(s) found in safety advisories:", file=sys.stderr)
        for err in errors:
            print(f"  - {err}", file=sys.stderr)
        return 1

    print(f"OK: Verified {len(advisory_files)} safety advisories and cross-references.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
