### Changed
#### Split ShauerPage.svelte under the 600-line component ceiling

Extracted article markup into `ShauerArticle.svelte`, force and convergence charts
into `ShauerResults.svelte`, and parameter controls into `ShauerParameters.svelte`,
reducing `ShauerPage.svelte` from 811 lines down to under 570 lines to respect the
600-line component ceiling without altering layout or behavior.
