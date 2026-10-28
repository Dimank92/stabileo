### Added
#### End-to-end certificates for shells, plates and solids (#484, 2026-10-28)

Extended end-to-end certification to cover DKT triangular plates, MITC4 and MITC9 quadrilateral shells,
SHB8-ANS solid shell elements, and curved shell elements. Independent reassembly in interval arithmetic
reconstructs element stiffness matrices and equivalent nodal loads (pressure, thermal, self-weight body forces,
and edge loads) directly from the model JSON, verifying equilibrium residual and agreement against the engine's
assembly so that every displayed number is certified against the model itself.
