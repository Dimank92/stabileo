### Changed
#### Browser CI uses Nix on trusted self-hosted runs (2026-10-29)

Web Build, E2E Site, all ten E2E shards, quarantined E2E tests and Determinism can run
on Stabileo's Arch Linux runners when STABILEO_SELF_HOSTED is enabled, only for the
merge queue and pushes to main. Pull requests keep the Ubuntu path. A dedicated Nix
shell pins Playwright 1.60.0 browsers and their libraries, including Firefox and
WebKit, and supplies fallback fonts without apt or sudo. The Linux host preflight
is `scripts/ci/check-playwright-nix.sh`.
