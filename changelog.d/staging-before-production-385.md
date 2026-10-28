### Changed
#### Staging before production: deploy main to staging, promote tagged releases (#385, 2026-10-28)

Release safety item 1: pushes to `main` now deploy continuously to the staging environment (`staging.stabileo.com`) instead of directly to production. Production (`stabileo.com`) is only promoted from a tagged release (`v*`) or intentional production dispatch after passing the production safety gates: `check.engine.acceptance-models` (acceptance suite) and `test.engine.gates` (correctness gates). Documented the full rollback procedure in `docs/DEPLOY.md` including automated rollback via `rollback.yml` and emergency manual SSH rollback. Added unit test `scripts/tests/test_deploy_configuration.py`.
