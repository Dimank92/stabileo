### Added
#### Documented, peer-reviewed element derivations in SOLVER_REFERENCE.md (2026-10-28)

`docs/SOLVER_REFERENCE.md` now details the mathematical formulation, kinematic hypotheses,
stiffness ($K$), mass ($M$), geometric stiffness ($K_g$), equivalent loads (FEF), and limits
of validity for every structural element family in the engine:
- Euler-Bernoulli and Timoshenko 2D and 3D frames
- Axial truss bars
- Catenary and elastic tension cables
- Curved beams and arches
- 6-DOF elastic connectors and discrete springs
- DKT (Discrete Kirchhoff Triangle) thin plates
- MITC4 and MITC9 Mindlin-Reissner quadrilateral shells
- Curved co-rotational facet shells with director rotary inertia
- SHB8 solid-shell hexahedra with ANS shear and EAS Poisson locking relief
- Distributed plasticity fiber beam-column elements

Section 5.12 formally records the Independent Peer Review findings (#406–#433) and
engineering verification status (#384, closes #384).

