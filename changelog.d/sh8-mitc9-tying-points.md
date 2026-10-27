### Fixed

#### SH-8: Document and benchmark MITC9 Gauss-Lobatto tying points variant (#412)

Documented the MITC9 transverse shear tying formulation in `quad9.rs` and `docs/solver/shells.md` as a DOCUMENTED-CHOICE: transverse shear is evaluated on a 2×3 Gauss-Lobatto product grid ($\xi = \pm 1/\sqrt{3}, \eta \in \{-1,0,1\}$), evaluating tying points directly along element boundary edges and the center axis. Added distorted-mesh thin-shell benchmark `sh8_mitc9_distorted_mesh_thin_shell_benchmark` in `engine/tests/derivation_shells.rs` verifying that the element does not shear lock on distorted meshes in the extreme thin limit ($L/t = 1000$) and maintains exact rigid body modes.
