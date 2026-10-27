### Added
#### PRO curved members as whole pieces, loads along the arc, plates with curved sides, IFC export in Project (2026-10-27)

An arc is picked whole (or one segment at a time), shows one local-axis triad, and has a panel with
its radius, angle, arc length, the chord error of its segments and Remake with another count,
keeping its id, section, material and end releases. Loads along the arc are given per metre of arc
in a global direction and spread on the segments keeping the total force. Results read N, V, T and
M against the length along the piece, and export as CSV. A plate side can follow an arc and share
the curved member's nodes. Project → Export offers IFC with design on or off.
