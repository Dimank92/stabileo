### Fixed
#### Close mutation test gap for catenary cable element (#747)

Comprehensive unit tests and equivalence annotations for all 8 functions in the cable element
kernel (`engine/src/element/cable.rs`): `catenary_ordinate`, `cable_length_catenary`,
`cable_end_tensions`, `irvine_parameter`, `ernst_equivalent_modulus`, `cable_natural_frequency`,
`ernst_tangent_modulus`, and `cable_chord_tension`. Closes the 119 missed mutant test gap from
nightly mutation testing (#747).
