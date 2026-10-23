### Fixed
#### Editing a spectrum retires the results solved on the old one (2026-10-21)

A user spectrum feeds the spectral load cases of the main solve (#261), and the time history
its own run, but `setDynamics` recorded on the views undo channel, which retires nothing, and
the edit did not bump the model version: D + Ex solved on a 0.4 g spectrum stayed on screen
after the spectrum became 0.8 g, live calc did not re-run, and undoing the edit kept the 0.8 g
results (W3-01). The spectra and the time history are now a model edit: undoable, and the
results on hand are retired on the edit, its undo and its redo; a write that changes nothing
takes no step. Undo entries a session saved on the old channel restore them the same way.
Test: `web/src/lib/store/__tests__/spectrum-edit-retires-results.test.ts`.
