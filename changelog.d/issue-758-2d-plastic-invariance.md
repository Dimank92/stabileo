### Fixed
#### 2D plastic collapse factor invariance under renumbering, reversal and units (2026-10-28)

2D plastic analysis now evaluates hinge unloading and reopening by rotation rate in parity with 3D plastic analysis, adopts invariant physical tie-breaking, suppresses spurious sliver hinges near member nodes, and prevents false kinematic mechanism suppression under rigid constraints. This guarantees that plastic collapse load factors remain invariant under node renumbering, element reversal, and unit scaling (#758).
