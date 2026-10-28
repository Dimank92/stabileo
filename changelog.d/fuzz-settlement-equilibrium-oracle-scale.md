### Fixed
#### Fuzz equilibrium oracle scale for support settlements (2026-10-28)

In the fuzzing harness (`check_equilibrium_2d` and `check_equilibrium_3d`), the equilibrium tolerance
scale now incorporates the stiffness times displacement magnitude (K * u) of connected elements.
This prevents false-positive assertion failures in certified settlement-only models where reactions are
analytically zero (such as rigid-body settlements) but floating-point round-off of order 1e-12 occurs.
