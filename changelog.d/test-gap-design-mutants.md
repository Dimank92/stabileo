### Fixed
#### Close mutation test gap in design-check certificates (#750)

Comprehensive unit and integration test coverage for all 23 functions in the design-check
certificates (`engine/src/certify/design.rs`): `interaction_moment_enclosure`,
`steel_demands_from_combinations_3d`, `steel_demands_from_combinations_2d`,
`cirsoc301_ratio_enclosures`, `timber_ratio_enclosures`, `certify_members`, `around`,
`ec2_ratio_enclosures`, `loaded_ratio`, `required_ratio`, `cirsoc201_ratio_enclosures`,
`tri_or`, `cirsoc301_demands`, `rc_ratio_enclosures`, `cfs_ratio_enclosures`,
`ec3_ratio_enclosures`, `require_monotone`, `cirsoc201_demands`, `certify_cirsoc301_members`,
`certify_rc_members`, `timber_demands`, `cfs_demands`, and `members_uncertified`. Closes the
67 missed mutant test gap from nightly mutation testing (#750).
