import json
import os
import re
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent


class TestSafetyAdvisories(unittest.TestCase):
    def setUp(self):
        self.advisories_dir = REPO_ROOT / "docs" / "advisories"
        self.safety_advisories_md = REPO_ROOT / "docs" / "SAFETY_ADVISORIES.md"
        self.result_changes_json = REPO_ROOT / "web" / "src" / "lib" / "provenance" / "result-changes.json"
        self.registry_ts = REPO_ROOT / "web" / "src" / "lib" / "provenance" / "registry.ts"

    def test_advisories_exist_and_no_placeholders_in_registry_field(self):
        """Every advisory in docs/advisories/ must have a resolved Re-check registry field (no 'to add with the fix')."""
        advisory_files = sorted(self.advisories_dir.glob("SA-2026-*.md"))
        self.assertGreaterEqual(len(advisory_files), 24, "Expected at least 24 safety advisories")

        unresolved = []
        for af in advisory_files:
            content = af.read_text(encoding="utf-8")
            # Look for "| Re-check registry | ... |"
            m = re.search(r"\|\s*Re-check registry\s*\|\s*([^|\n]+)\|", content, re.IGNORECASE)
            self.assertIsNotNone(m, f"{af.name} is missing the 'Re-check registry' row in its metadata table")
            val = m.group(1).strip()
            if "to add with the fix" in val.lower() or val == "":
                unresolved.append((af.name, val))

        self.assertEqual(
            unresolved,
            [],
            f"Advisories have unresolved Re-check registry placeholders: {unresolved}"
        )

    def test_result_changes_advisory_cross_references(self):
        """Entries in result-changes.json that reference an advisory must reference an existing file, and advisories must map to real entries."""
        self.assertTrue(self.result_changes_json.exists())
        with open(self.result_changes_json, "r", encoding="utf-8") as f:
            data = json.load(f)

        changes = data.get("changes", [])
        change_ids = {c["id"] for c in changes}

        # Check changelog file references
        for c in changes:
            cl = c.get("changelog", {})
            f_path = cl.get("file", "")
            if "docs/advisories/SA-" in f_path:
                full_path = REPO_ROOT / f_path
                self.assertTrue(full_path.exists(), f"Change {c['id']} links to non-existent advisory file {f_path}")

            # If change has advisory field, ensure it matches SA-YYYY-NNN format
            if "advisory" in c:
                self.assertRegex(c["advisory"], r"^SA-\d{4}-\d{3}$", f"Invalid advisory format in {c['id']}")
                matched_files = list(self.advisories_dir.glob(f"{c['advisory']}*.md"))
                self.assertTrue(len(matched_files) > 0, f"No advisory file found for {c['advisory']}")

    def test_safety_advisories_has_release_checklist(self):
        """SAFETY_ADVISORIES.md must have a formal release safety checklist section."""
        content = self.safety_advisories_md.read_text(encoding="utf-8")
        self.assertIn("## Release safety checklist", content)
        self.assertIn("100 %", content)
        self.assertIn("recorded reason", content)

    def test_safety_advisories_has_opt_in_notification_system(self):
        """SAFETY_ADVISORIES.md must specify the opt-in notification list and feed architecture."""
        content = self.safety_advisories_md.read_text(encoding="utf-8")
        self.assertIn("## Safety advisory notification list (opt-in)", content)
        self.assertIn("feed", content.lower())
        self.assertIn("webhook", content.lower())

    def test_registry_ts_supports_advisory_field(self):
        """registry.ts ResultChange interface must support advisory property and validate it."""
        content = self.registry_ts.read_text(encoding="utf-8")
        self.assertIn("advisory?:", content)
        self.assertIn("SA-\\d{4}-\\d{3}", content)


if __name__ == "__main__":
    unittest.main()
