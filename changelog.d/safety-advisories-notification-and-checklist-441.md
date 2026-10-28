### Added
#### Safety advisory notifications, re-check registry integration and release checklist (#441)

Closes #441 (roadmap:release-safety, tracker #396). Provides complete lifecycle support for
engineering safety advisories when a released version produces an unconservative or wrong result:
- Enforced re-check registry cross-references across all 24 safety advisories (SA-2026-001 through
  SA-2026-024) in `docs/advisories/` and `web/src/lib/provenance/result-changes.json`, including
  the `advisory` field on `ResultChange` in `registry.ts`.
- Documented the opt-in notification architecture in `docs/SAFETY_ADVISORIES.md` covering GitHub
  Security Advisories, opt-in email notifications, and machine-readable JSON / RSS feeds.
- Formalised the release safety checklist in `docs/SAFETY_ADVISORIES.md` ensuring 100 % of closed
  `safety` issues have an advisory or recorded justification.
- Added automated verification script `scripts/check-safety-advisories.py` and test suite
  `scripts/tests/test_safety_advisories.py`.
