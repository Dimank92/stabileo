### Fixed
#### Mutation test gaps in dynamic and modal certificates (#748) (2026-10-29)

Added comprehensive unit tests across modal, spectral, harmonic, and time-history certificate validation routines (mode isolation bounds, spectral range checks, Rayleigh stability rules, and harmonic response sweep verification), closing 214 surviving mutation gaps and configuring justified exclusions for heuristic power-iteration starting vectors in `.cargo/mutants.toml`.
