### Changed
#### More trusted merge-queue checks use Stabileo runners

Research WASM, Results Snapshot and Lint Repo now join the existing engine and WASM jobs on
Stabileo runners when `STABILEO_SELF_HOSTED` is enabled. Pull requests still use GitHub runners.
Self-hosted jobs explicitly clean checkout workspaces and use the installed Nix toolchain
without running the Nix installer or GitHub-only disk cleanup and build-cache actions.
Timing-sensitive gates (including Web Unit) stay on GitHub, as do Docker smoke and browser
jobs: Docker is not guaranteed on the host, and the flake does not yet provide Playwright
browsers and their runtime libraries.
