### Added
#### PRO DXF templates over several levels, CAD-style snaps, assisted conversion, and per-floor plans in the CAD wizard (2026-10-27)

A DXF with several plans is prepared as templates: its regions found, each laid on a level by a
point, locked, aligned by two pairs of points (a scale mismatch is reported, never applied), its
layers shown or hidden and snapped or not. Object snaps on templates (endpoint, midpoint, center,
quadrant, intersection, perpendicular, nearest) are turned on one by one from the draw bar and
remembered per browser; F3 turns them off and on. Part of a template converts to members, arcs and
columns after a plan that reports duplicates, joins, T joints and loose ends. The CAD draft wizard
finds the separate plans of a drawing, lays each on the one below by its columns, and gives every
floor its plan and height in a table; the provenance records each floor's plan and the point each
plan was laid by. The DXF and IFC import dialogs use the application's palette.
