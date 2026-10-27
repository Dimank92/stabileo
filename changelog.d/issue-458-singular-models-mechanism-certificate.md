### Fixed
#### Refuse singular models from hinged 2D nodes tied by rigid links and guard certificate against mechanisms (2026-10-27)

Nodes tied by 2D rigid links with default empty degrees of freedom (`[0, 1, 2]`) and diaphragms are now correctly included in `rotation_tied_nodes_2d`, preventing artificial rotational stiffness from being added to slave nodes and masking mechanisms. In `refuse_moments_without_rotation_2d`, rotational stiffness is propagated through constraints only from stiffened nodes rather than assuming master nodes are always carried. Furthermore, the assembled-level linear certificates in `CertifiedResults2D::guard` and `CertifiedResults3D::guard` are downgraded when the independent end-to-end check detects a mechanism, non-positive-definite matrix, or failure (#458).
