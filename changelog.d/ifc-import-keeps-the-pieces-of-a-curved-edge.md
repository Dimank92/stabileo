### Changed
#### IFC import keeps every straight piece of a curved edge on its own points (2026-10-27)

The analytical model of an imported building joined consecutive members that were offset from
each other, which turned the short pieces of a curved slab edge into fewer chords off the edge.
Pieces are now joined only when they point the same way (within 0.5°). A building imported from
now on can have more analytical members along curved edges, with the same total length; projects
already imported are not touched.
