#!/usr/bin/env python3
"""Unreleased changes live one per file in changelog.d/, and go into CHANGELOG.md at release.

Roadmap: Release safety #386 (item 2):
"Versioned releases with notes: every release has a version, release notes in English
and Spanish, and a prominent 'results may change' section listing result-affecting
changes and who is affected."

Every PR adds its own file:
    changelog.d/<short-slug>.md
        ### Fixed                      <- the section: one of SECTIONS below
        #### What changed (2026-10-22)  <- the entry's title, as before
        The body, as before.

    scripts/changelog.py --check                   # CI: fragments & bilingual notes well formed
    scripts/changelog.py --release 0.3.1 2026-10-23  # move them under "## 0.3.1 (2026-10-23)"
"""
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
FRAGMENTS = ROOT / "changelog.d"
CHANGELOG = ROOT / "CHANGELOG.md"
RELEASES_DIR = ROOT / "docs" / "releases"

# The order a release lists them in.
SECTIONS = ["BREAKING (results)", "Security", "Added", "Changed", "Fixed", "Performance"]
UNRELEASED = "## Unreleased\n\nOne file per change in [`changelog.d/`](changelog.d/README.md); `scripts/changelog.py --release` moves them here.\n"


def fragments():
    return sorted(p for p in FRAGMENTS.glob("*.md") if p.name != "README.md")


def parse(path):
    text = path.read_text(encoding="utf-8").strip() + "\n"
    first, _, rest = text.partition("\n")
    m = re.fullmatch(r"### (.+)", first.strip())
    problems = []
    if not m or m.group(1) not in SECTIONS:
        problems.append(f"{path.name}: the first line must be '### <section>', one of {SECTIONS}")
    if not re.search(r"^#### .+", rest, re.M):
        problems.append(f"{path.name}: needs a '#### <title>' line")
    if re.search(r"^##? ", rest, re.M) or re.search(r"^### ", rest, re.M):
        problems.append(f"{path.name}: only one '### <section>' line, at the top")
    return (m.group(1) if m else None), rest.strip() + "\n", problems


def validate_release_notes(version: str) -> list[str]:
    """Validate that bilingual release notes exist and meet Release safety #386 criteria."""
    problems = []
    en_file = RELEASES_DIR / f"v{version}.md"
    es_file = RELEASES_DIR / f"v{version}.es.md"

    if not en_file.is_file():
        problems.append(f"Release v{version}: missing English release notes {en_file.name}")
    else:
        text_en = en_file.read_text(encoding="utf-8")
        if not re.search(r"results\s+may\s+change|results\s+change", text_en, re.I):
            problems.append(f"{en_file.name}: missing prominent 'results may change' section")

    if not es_file.is_file():
        problems.append(f"Release v{version}: missing Spanish release notes {es_file.name}")
    else:
        text_es = es_file.read_text(encoding="utf-8")
        if not re.search(r"resultados\s+pueden\s+cambiar|resultados\s+cambian|resultados", text_es, re.I):
            problems.append(f"{es_file.name}: missing prominent results change section in Spanish")

    return problems


def check() -> int:
    problems = [p for f in fragments() for p in parse(f)[2]]

    # Also validate that all released versions in CHANGELOG.md have valid bilingual notes (#386)
    if CHANGELOG.is_file():
        text = CHANGELOG.read_text(encoding="utf-8")
        versions = re.findall(r"^##\s+(\d+\.\d+\.\d+)", text, re.M)
        for v in versions:
            problems.extend(validate_release_notes(v))

    for p in problems:
        print(p, file=sys.stderr)
    return 1 if problems else 0


def release(version: str, date: str) -> int:
    if check():
        return 1
    # Check that release notes exist for the new version
    note_problems = validate_release_notes(version)
    if note_problems:
        for p in note_problems:
            print(p, file=sys.stderr)
        return 1

    by_section = {s: [] for s in SECTIONS}
    for f in fragments():
        section, body, _ = parse(f)
        by_section[section].append(body)
    block = f"## {version} ({date})\n\n"
    block += f"Release notes: [`docs/releases/v{version}.md`](docs/releases/v{version}.md) (also in [Spanish](docs/releases/v{version}.es.md)).\n"
    for s in SECTIONS:
        if by_section[s]:
            block += f"\n### {s}\n\n" + "\n".join(by_section[s])
    log = CHANGELOG.read_text(encoding="utf-8")
    start = log.index("## Unreleased")
    end = log.index("\n## ", start + 1) + 1
    CHANGELOG.write_text(log[:start] + UNRELEASED + "\n" + block + "\n" + log[end:], encoding="utf-8")
    for f in fragments():
        f.unlink()
    print(f"CHANGELOG.md: {version} with {sum(map(len, by_section.values()))} entries")
    return 0


if __name__ == "__main__":
    if sys.argv[1:] == ["--check"]:
        sys.exit(check())
    if len(sys.argv) == 4 and sys.argv[1] == "--release":
        sys.exit(release(sys.argv[2], sys.argv[3]))
    print(__doc__)
    sys.exit(2)

