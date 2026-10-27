### Added
#### PRO report: rebuild slab reinforcement section from slab design results (#373)

- Restored the PRO report's slab reinforcement section, sourced directly from the slab design's own result object:
  - Dimensions and geometry: span (lx × ly), thickness (e), and one-way/two-way classification.
  - Design parameters: factored load (qu), concrete compressive strength (f'c), steel yield strength (fy), and aspect ratio.
  - Reinforcement schedule: top and bottom layers in both directions with bar diameter, spacing, As,req, As,prov, and governing status.
  - Verification: one-way shear check (vu, φvc, and utilization percentage).
- When no slab design exists, the section is either omitted or rendered with an explanatory note rather than printing fabricated values.
- `ProReportDialog`: added toggle for the slab reinforcement section with status indicator.
