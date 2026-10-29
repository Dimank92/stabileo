use springs_grossman_2026::{experiment::{run_periodic, PeriodicSpec}, moduli};

#[test]
fn oversized_json_specs_fail_before_building() {
    // Includes wasm32 multiplication overflow, native overflow, and excessive basis nodes/bonds.
    for (lattice, n1, n2) in [
        (r#"{"kind":"square","diagonals":true}"#, 60_001, 3),
        (r#"{"kind":"square","diagonals":false}"#, 65_536, 65_536),
        (r#"{"kind":"square","diagonals":false}"#, usize::MAX, 3),
        (r#"{"kind":"honeycomb","l":0.25,"k_radial":0.001}"#, 150, 150),
    ] {
        let json = format!(r#"{{"network":{{"lattice":{lattice}}},"n1":{n1},"n2":{n2}}}"#);
        let spec: PeriodicSpec = serde_json::from_str(&json).unwrap();
        let start = std::time::Instant::now();
        let error = run_periodic(&spec).unwrap_err();
        assert!(error.contains("limit") || error.contains("overflow"), "{error}");
        assert!(start.elapsed().as_secs_f64() < 1.0);
    }
}

#[test]
fn square_lattice_supports_axial_loads_but_not_shear_loading() {
    let spec: PeriodicSpec = serde_json::from_str(r#"{"network":{"lattice":{"kind":"square","diagonals":false}},"n1":4,"n2":4}"#).unwrap();
    let result = run_periodic(&spec).unwrap();
    for angle in [0.0, std::f64::consts::FRAC_PI_2, std::f64::consts::PI] {
        let (young, poisson) = moduli::uniaxial(&result.c, angle).unwrap();
        assert!((young - 1.0).abs() < 1e-10);
        assert!(poisson.abs() < 1e-10);
    }
    assert!(result.c[2][2].abs() < 1e-10);
    assert!(moduli::uniaxial(&result.c, std::f64::consts::FRAC_PI_4).is_none());
    // A transverse floppy mode makes Poisson's ratio indeterminate.
    assert!(moduli::uniaxial_x(&[[1.0, 0.0, 0.0], [0.0; 3], [0.0; 3]]).is_none());
}

#[test]
fn oversized_json_strips_fail_before_building() {
    use springs_grossman_2026::experiment::{run_strip, StripSpec};
    for width in [100_000.0, 1e100] {
        let json = format!(r#"{{"network":{{"lattice":{{"kind":"square","diagonals":true}}}},"width":{width},"height":100,"theta":0,"clamp_band":0.5,"strain":0.01}}"#);
        let spec: StripSpec = serde_json::from_str(&json).unwrap();
        let start = std::time::Instant::now();
        let error = run_strip(&spec).unwrap_err();
        assert!(error.contains("limit") || error.contains("overflow"), "{error}");
        assert!(start.elapsed().as_secs_f64() < 1.0);
    }
}
