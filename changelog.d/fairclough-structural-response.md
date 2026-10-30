### Fixed
#### Validate Fairclough gridshell structural response and member forces in engine CI (#856) (2026-10-30)

Add independent structural response assertions to the Fairclough gridshell benchmark in engine validation CI. Beyond frame volume and equilibrium, independently validate apex vertical displacement and maximum displacement (1.267072e-6 m), 4-way corner reaction distribution and radial diagonal thrust symmetry with net-zero lateral reaction, pinned-support zero bending moments, and base/crown member axial forces.

