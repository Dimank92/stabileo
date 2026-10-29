### Fixed
#### Research shell desktop pane scrollable on short windows (2026-10-29)

In `web/src/components/research/shell/ExperimentShell.svelte`, the sticky right desktop pane had a fixed height with `overflow: hidden`, causing panels like Network and Parameters to be clipped without scroll on short desktop windows. Configured `max-height` and `overflow-y: auto` with internal tabpanel scrolling, and made `PaneTabs.svelte` sticky at top so tabs remain visible when scrolling panels. Preserves the one-scroll page. Issue #802.
