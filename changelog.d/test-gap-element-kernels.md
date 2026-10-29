### Fixed
#### Mutation test gaps in element kernels (#752) (2026-10-29)

Added comprehensive unit tests covering fixed-end forces (distributed torsion warping, 12-to-14 DOF expansion, partial span limits, and thermal gradient thresholds), fiber beam constitutive models (Hognestad concrete compression, tension cracking, bilinear steel kinematic hardening, and Gauss-Lobatto numerical quadrature rules), curved beam arc generation and line intersection branches, 3D truss stiffness tensors, 3D coordinate transform matrices and orthonormal frame fallbacks, and 3D beam local stiffness with torsional warping coupling and releases, closing 260 surviving mutation gaps.
