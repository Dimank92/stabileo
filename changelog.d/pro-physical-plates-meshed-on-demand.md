### Added
#### PRO plates as physical objects, meshed on demand, with Diagnostics on the Analyse row (2026-10-27)

A plate is drawn as an outline of any number of nodes, with holes, material, thickness and area
loads kept on the plate, and holds no shells until it is meshed: from its panel or in a batch, as
free quadrilaterals or triangles or as a grid of rectangles or squares of chosen sides, after a
plan that lists what changes. Meshing before the analysis (on by default) meshes only plates with
no mesh or an out-of-date one, once per solve; a plate whose old mesh carries loads put on its
elements is never remeshed there, the analysis names it instead. A plate is picked whole or by
element, shows one local-axis triad when picked whole, and has per-plate extremes of its moments,
membrane forces and deflection in its own axes. Diagnostics now has its own command on the
Analyse row. Only objects drawn this way are plates: existing shells and the slabs of the building
generators are unchanged, and no saved result moves.
