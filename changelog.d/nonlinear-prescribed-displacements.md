### Added
#### Nonlinear solvers: prescribed displacements and rotations (#858)

Support prescribed nodal displacements and rotations as a load stage in
`solve_corotational_2d`, `solve_arc_length`, and `solve_displacement_control`.
Includes the tangent stiffness prescribed-DOF coupling $f_t = f_{\text{ref}} - K_{fr} u_r$
in both dense and sparse paths, updates displacement and reaction recovery, and adds
rigorous analytical benchmarks for an elastica strip bent into a clamped arch by pure
symmetric end rotation and driven through snap-through under displacement control.
