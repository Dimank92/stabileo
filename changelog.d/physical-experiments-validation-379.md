### Added
#### Validation against physical experiments for nonlinear material models (2026-10-28)

Added experimental validation against published laboratory benchmarks following ASME V&V 10-2019 and NAFEMS practice (#379):
- **RC Beams:** Bresler-Scordelis (UC Berkeley OA-1) load-deflection and diagonal tension cracking capacity.
- **RC Columns:** Burns-Siess and Saatcioglu-Razvi cyclic axial-flexural response and plastic hinge rotation capacity.
- **Shake-Table Frames:** E-Defense full-scale multi-story dynamic period elongation and inelastic drift response.
- **Vecchio-Collins Panels:** University of Toronto Shell Element Tester (PV19) pure shear capacity and compression softening.
- **Connection Tests:** SAC Joint Venture and Popov cyclic moment connection contact gap and plastic moment capacity.
- Documented in `docs/VERIFICATION.md` Section 10 and implemented under `engine/tests/validation/physical_experiments.rs`.
