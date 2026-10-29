# PR #776 deferred work

## Responsive live computation and precomputed replay (item 6) - Resolved (#801)

Resolved in #801:
- Strip and periodic solves (including prestress relaxation) execute inside a dedicated research module worker (`springs.worker.ts`).
- Worker terminates unconditionally on `Stop`, superseding inputs, and page destruction; rejects late results and restarts with a fresh worker.
- WASM is loaded inside the worker with deployment-safe asset URLs.
- The live autoplay sweep is driven by precomputed frames (`sweep-frames.json`) produced by `precompute_sweep.rs` from the same solver, performing zero WASM solver calls during autoplay.
- UI provides Recompute live, Stop, and Restart controls with accurate source badges (`replay`, `cached`, `live`) and busy indicators.
- Verified with browser E2E acceptance tests (`e2e/research-springs.spec.ts`) using the real WASM bundle: UI heartbeat and navigation remain responsive, Stop terminates computation, obsolete results never publish, and Restart completes. Autoplay performs 0 solves, and research worker/WASM assets never load on non-research routes.

## Other deferred items (unchanged)

- Crossref remains only in the standalone workflow, by deliberate user decision.
- Immutable deployed assets across deploys: covered by the #777 redesign.
- Sticky pane overflow on short windows: shared-shell follow-up.
