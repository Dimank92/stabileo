### Added
#### Footings and CIRSOC 104 snow load generation in the Rust engine (2026-10-28)

CIRSOC 104-2005 roof snow load generation is moved to the Rust engine
(`engine/src/loads/cirsoc104_snow.rs`, op `cirsoc104Snow`), covering Cap. 3 flat-roof snow
`p_f = 0,7 C_e C_t I p_g` with §3.4 low-slope minimums, Cap. 4 slope factor `C_s` and `p_s`,
Cap. 6 §6.1 unbalanced gable snow with drift index β, Cap. 7 snow density γ, and Cap. 10 rain on snow.
`web/src/lib/codes/cirsoc104/snow.ts` delegates to the engine via `loadsRequest` with
`snow-oracle.ts` preserved as a differential oracle verified against thousands of property sweeps.
Spread footing checks support column-face flexure and deliberate column offset eccentricity via
`check_spread_footing_detailed` and `isolated_footing.rs` (#393).
