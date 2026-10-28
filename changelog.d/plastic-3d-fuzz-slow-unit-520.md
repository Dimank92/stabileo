### Fixed
#### Prevent runaway 3D plastic polygon-LP bisection and per-pivot clones (#520)

In 3D plastic collapse certification (`certify_plastic_3d`), outer-polygon refinement now gates vertex bisection on active dual hinges ($\sum \mu > 10^{-9} \mu_{\max}$), terminates early when yield violations are cleared and $\lambda$ has converged, skips redundant LP re-solves when already tight, and eliminates per-pivot vector allocations in the simplex tableau via in-place row splitting. Resolves fuzz timeouts and ASan memory pressure (#520).
