### Added
#### PRO use zones per floor, their live loads from the CIRSOC 101 table, and finishes (2026-10-27)

Use zones are drawn per floor or taken from a template's closed outlines, independent of the mesh.
Each zone takes an occupancy from the CIRSOC 101 table already in the application or a stated
value; names found in the drawing are offered as suggestions to confirm, never applied on their
own. "No load" is a decision of its own, shown apart from a zone with no use yet. The zones' live
loads, and finishes as a dead load of their own, are written as load definitions after a plan
that shows what is added, changed and removed, and running it again adds nothing; loads entered
by hand are never touched. Zones are copied between floors with their exceptions. A zone over a
plate not yet meshed loads the slab once it is meshed; finishes in a model with no dead-load case
are named in the plan and in Diagnostics.
