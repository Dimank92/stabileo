### Added

#### Substructuring and model reduction for multi-storey buildings (#485)

* Exported `guyan_reduce_3d` and `craig_bampton_3d` in the engine public API for multi-storey building substructuring and superelement model reduction.
* Added validation benchmarks verifying exact displacement and internal force recovery for multi-storey building floor superelements.
* Added architectural specification `docs/architecture/SUBSTRUCTURING.md` covering Guyan condensation, Craig-Bampton dynamic reduction, floor superelements, and parallel solver integration.
