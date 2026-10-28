# Stabileo Research

Research you can run: computational-mechanics papers whose numerical experiments are reproduced,
re-implemented and made interactive, with the evidence for each claim kept next to the code.

This directory is **research, not engineering design**. The application may read the catalogue and published results data, but never research code;
no result here is a design result. A benchmark agreement is not a
certification.

The plan every page and contribution follows is [DESIGN.md](DESIGN.md).

## Layout

| Path | What |
|------|------|
| `papers.yaml` | Curated catalog. Bibliographic fields are checked against Crossref by `tools/catalog.py` |
| `fracture/shauer-2025/` | Phase-field fracture (AT2), after Shauer (2025), reference code and Rust implementation |
| `templates/` | Starting point for a new paper entry |
| `tools/` | Catalog checker and other research-only tooling |

## Isolation from the product

- Research crates are their own Cargo workspaces (`[workspace]` in their manifests); they are not
  members of the engine's or the backend's workspaces and have their own lockfiles.
- A research crate may depend on `engine/` through a path dependency. The engine and the web
  application never import research code. The web allowlist is `research/papers.json`,
  `research/<field>/<paper>/VALIDATION.json`, and CSV/JSON files in that paper’s
  `reference-results/`; the engine has no exceptions. `tools/check_isolation.sh` enforces this.
- Research CI is `.github/workflows/research.yml`, path-filtered, not part of the required checks.

## Reproducibility statuses

Statuses in `papers.yaml` are assigned from evidence in the repository, not by hand-waving:

| Status | Evidence required |
|--------|-------------------|
| `proposed` | Catalogued; metadata verified against Crossref |
| `reference-reproduced` | The authors' code ran here; its outputs are committed with the exact environment |
| `implementation-in-progress` | A Stabileo implementation exists but its validation tests do not yet pass |
| `numerically-validated` | Tests compare our implementation with the reproduced reference within documented tolerances, and pass in CI. This is agreement with the reference code; whether published numbers were reproduced is the separate `reproduced_published_results` field |
| `interactive` | Validated, and runnable on stabileo.com/research |

## Product route size baseline

From `web/`, run `nix develop .. -c node scripts/check-research-route-size.mjs`.
The script makes a production build with a manifest and measures uncompressed dependency
bytes for `/en/`, `/es/`, `/pt/` and every `/app/*` mode. Landing routes count the entry's
static closure; app routes also count App's static and lazy product dependencies (a conservative
budget shared by all modes). Explicit lazy research routes are excluded; static research
imports fail. HTML, bundled JS/CSS, manifest assets and statically referenced public files are counted once.
Runtime-generated asset URLs and the engine binary are outside this gate; a fixed engine stub makes the baseline
independent of locally built WASM. Browser smoke tests separately check actual route loading.
Each route may grow at most 128 bytes. Research CI runs this gate.

For an intentional product change, run the same command with `--update`, review the old/new
per-route totals and explain every increase in the PR. Do not update it to hide research
bundle leakage. The initial baseline records this checkout, not a comparison with a historical
revision. Use the pinned Node version and `npm ci` dependencies.
