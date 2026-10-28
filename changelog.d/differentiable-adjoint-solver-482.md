### Added
#### Differentiable solver: adjoint sensitivities for linear static, modal, and buckling responses (#482, 2026-10-28)

Implemented adjoint sensitivity analysis (`engine/src/solver/adjoint.rs`) enabling calculation of structural
response gradients (nodal displacements, inter-storey drifts, reaction forces, modal natural frequencies,
and elastic buckling critical load factors) with respect to all design variables (Young's modulus $E$, section
area $A$, moments of inertia $I_y, I_z$, plate and shell thicknesses $t$, and spring stiffnesses $k$) in $O(1)$
operations per parameter. All analytical adjoint gradients are verified against central finite differences.
