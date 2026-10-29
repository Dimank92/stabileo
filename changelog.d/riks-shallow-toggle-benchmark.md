### Fixed
#### Riks shallow toggle benchmark: axial truss formulation and step continuation (#846) (2026-10-30)

Switch the Riks/Crisfield shallow toggle benchmark in engine validation CI to truss elements, matching the closed-form analytical axial equilibrium equation and eliminating spurious frame bending stiffness. Calibrate arc-length continuation step budget to 450 steps with maximum step size 0.25 to trace past the snap-through limit point, achieving 0.0007% maximum path error.
