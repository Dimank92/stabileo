### Added
#### DXF 3D lines and nodes import without flattening (2026-10-28)

DXF import previously stripped Z coordinates from LINE, POLYLINE, POINT, and 3DFACE entities, merging 3D nodes sharing the same (x, y) coordinates into 2D projections and dropping vertical members as zero-length lines. 3D coordinates and elements are now preserved during parsing and mapping, spatial line splitting respects 3D distance, and importing 3D models automatically activates 3D workspace presentation.
