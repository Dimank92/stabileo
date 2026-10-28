### Added
#### Nonlinear and dynamic robustness with energy-drift monitoring and limit-point tracking (#381, 2026-10-28)

Audited convergence criteria across material-nonlinear, corotational, and arc-length solvers.
Time-history dynamic solvers now compute and report `DynamicEnergyBalance` (kinetic energy,
strain energy, external work, damping dissipation, and relative drift) in results. Emits an
actionable diagnostic warning when relative energy drift exceeds 5%. Added a dedicated test suite
and documentation verifying force/displacement/work convergence criteria, load-step sensitivity,
second-order time-step convergence, and path following past limit points (snap-through) via
displacement control.
