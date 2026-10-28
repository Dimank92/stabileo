### Fixed
#### Relative convergence criteria for P-Delta, fiber and corotational iterations (#761)

Replaced absolute floors in relative convergence tests (1e-20 in P-Delta and 1e-30 in fiber nonlinear / corotational) with model-scale relative criteria. Models with loads applied directly to support nodes now converge invariants-cleanly under mirror, rotation, and unit scaling.
