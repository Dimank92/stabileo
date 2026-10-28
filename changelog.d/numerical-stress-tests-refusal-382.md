### Added
#### Numerical stress tests and refusal beyond accuracy limits (2026-10-28)

Models with extreme stiffness contrast (diagonal ratio exceeding 1e16, where IEEE 754 f64 loses all significant precision) are now strictly refused before solve rather than returning silently corrupted results. Added systematic numerical stress tests covering extreme stiffness ratios, high-DOF grid scaling, unconstrained mechanisms, and unit-extreme models.
