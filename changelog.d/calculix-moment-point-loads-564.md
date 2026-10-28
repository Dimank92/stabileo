### Fixed
#### Fold interior point loads into moment-equivalent stations for CalculiX (#564)

In differential testing with CalculiX (`tools/differential/common/model.py`), `moment_point_loads(model)` now folds interior `pointOnElement` loads on frame members into four moment-equivalent stations alongside distributed loads. This prevents `nodalize` from generating microscopic sub-elements between closely spaced point loads that ill-condition the stiffness matrix and cause CalculiX's U1 element to lose precision (#564).
