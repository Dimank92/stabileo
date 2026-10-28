import os
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent


class TestCertificatesShellsSolids(unittest.TestCase):
    def test_reassembly_shell_module_exists(self):
        shell_src = ROOT / "engine" / "src" / "certify" / "reassembly" / "shell.rs"
        self.assertTrue(shell_src.is_file(), f"reassembly/shell.rs missing: {shell_src}")

    def test_model3d_allows_shells_and_plates(self):
        model3d = ROOT / "engine" / "src" / "certify" / "reassembly" / "model3d.rs"
        text = model3d.read_text(encoding="utf-8")
        self.assertNotIn(
            'return Err("out of scope: shell and solid elements".into());',
            text,
            "model3d.rs still unconditionally rejects shells and solids",
        )

    def test_certify_doc_updated_for_shells(self):
        doc = ROOT / "docs" / "verification" / "CERTIFY.md"
        text = doc.read_text(encoding="utf-8")
        self.assertIn("MITC4", text)
        self.assertIn("DKT", text)
        self.assertIn("solid shell", text.lower())

    def test_trusted_base_doc_updated(self):
        doc = ROOT / "docs" / "verification" / "TRUSTED_BASE.md"
        text = doc.read_text(encoding="utf-8")
        self.assertIn("shell", text.lower())


if __name__ == "__main__":
    unittest.main()
