### Changed
#### ShauerPage split under the 600-line component ceiling (2026-10-29)

`web/src/components/research/shauer2025/ShauerPage.svelte` was 811 lines, over the project's 600-line component ceiling. The article snippet, simulation parameter controls, and results panels have been extracted into `ShauerArticle.svelte`, `ShauerParameters.svelte`, and `ShauerResultsPanels.svelte`. No behavior is changed; pinned by `shauer-ceiling.test.ts`. Issue #794.
