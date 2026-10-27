### Fixed

#### Imperfection analysis: document and un-ignore NL-9, NL-10, NL-11 regressions (#442)

Nonlinear specification `docs/solver/nonlinear.md` had left NL-9, NL-10, and NL-11 as `OPEN`, even though the engine already defaults 3D notional loads to Z, lumps member gravity loads, and explicitly refuses unapplied initial displacements and residual stresses. The documentation now marks these items as fixed and un-ignores the corresponding regression tests in `engine/tests/derivation_nonlinear.rs`.
