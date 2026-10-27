### Fixed
#### A member cut by a mesh keeps its loads on its pieces (2026-10-27)

Meshing a region with "split members" (the quick quadrilateral mesher, slabs meshed at the
analysis, and now physical plates) cut the members after the loads had been committed: the pieces
came without their loads, and the loads kept naming the member that was gone. The analysis then
refused the model ("Load N is on member M, which the model does not have"), so no result was ever
computed on the wrong loads and none needs re-checking. Meshes made from now on keep each load on
the pieces of its member, with its total. A project saved after hitting the defect still carries
those loads on a member it no longer has, and still says so when solved: delete them and put them
on the pieces.
