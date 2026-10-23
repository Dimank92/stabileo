### Fixed
#### The default steel declares its ultimate strength, so a new model's steel checks run (2026-10-22)

The material every new model starts with, "Acero A36", declared Fy = 250 MPa but no Fu, so every
CIRSOC 301 check of a new model, the bundled examples included, read "not evaluated: the material
lacks Fu" until the user typed it. It now declares Fu = 400 MPa, ASTM A36's minimum and the value
the grade catalogue gives A36. Fu feeds the design checks only, not the analysis. Found by the new
end-to-end test of the everyday path (#597).
