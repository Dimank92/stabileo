### Added
#### Steel check inputs computed in Rust engine (#391)

Migrates steel design check inputs from unverified TypeScript to the Rust
engine (`engine/src/postprocess/steel_inputs.rs`): Cb moment gradient factor
(CIRSOC 301 §F.1.1), section constants J, Cw, Zx, Zy (AISC DG 9, CIRSOC 301
§F.2), unbraced lengths L and Lb for member chains and flange bracing,
torsional buckling feasibility and Fez (CIRSOC 301 §E.4), flange local
buckling slenderness and classification (CIRSOC 301 §F.6.2, Tabla B.4.1b),
effective net area Ae = An · U from end connections (CIRSOC 301 §D.3,
§B.4.2.2), and structural steel material family classification.
