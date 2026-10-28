### Changed
#### Versioned releases with bilingual notes and a 'results may change' section (#386, 2026-10-28)

Release safety item 2: every release has a version, release notes in English and Spanish, and a prominent 'results may change' section listing result-affecting changes and who is affected. Updated `scripts/changelog.py` with `validate_release_notes` to verify bilingual release notes (`docs/releases/v<version>.md` and `docs/releases/v<version>.es.md`) and the prominent results-changed notice across all releases during `--check` and `--release`. Added unit test suite `scripts/tests/test_release_notes.py`.
