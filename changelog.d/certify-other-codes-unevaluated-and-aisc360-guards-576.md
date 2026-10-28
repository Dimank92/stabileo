### Fixed
#### AISC 360 and other-code design certificate dishonesty and input guards (#576, 2026-10-28)

`certifyOtherCode` in `run.ts` now propagates specific unevaluated reason keys from member
and station evaluations instead of generic strings, preventing incomplete design runs from
falsely certifying members as OK. In `steel-codes.ts`, `aiscGaps` accounts for Round HSS
(CHS) shear slenderness limits per AISC 360-22 G5 ($D/t > 1.191 (E/F_y)^{2/3}$ and the $0.45 E/F_y$
geometry cap), and `AISC360.member()` populates `flangeSlenderness` for I-shapes so that F3.2 and
F6.2 flange local buckling limits are enforced. The Rust engine checker (`steel_check.rs`) and interval
certificate verifier (`certify/design.rs`) guard against invalid inputs: Shear G2 requires positive
capacity via `ratio_if_loaded` / `loaded_ratio` when web area is supplied, unbraced compression (E3)
refuses non-positive radius of gyration instead of substituting zero slenderness, and unbraced
lateral-torsional buckling (F2) refuses non-positive moment of inertia ($I_y \le 0$) rather than
bypassing LTB and claiming full plastic moment capacity (#576).
