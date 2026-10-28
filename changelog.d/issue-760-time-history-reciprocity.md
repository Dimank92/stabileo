### Fixed
#### Reciprocity in linear time history on models with massless DOFs (2026-10-28)

Models where some degrees of freedom carry no mass (e.g. unassigned density on elements or rotational DOFs) formerly caused an asymmetry during initial acceleration computation, as forces on massless DOFs were dropped from massive DOFs in initial acceleration. Initial acceleration now correctly accounts for static equilibrium transfer from massless DOFs to massive DOFs, preserving Maxwell-Betti reciprocity across all time steps.

