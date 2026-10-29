use crate::{
    ana2, ana3, checks2, checks3,
    corpus::{self, Tier},
    tally::Tally,
};

#[test]
#[ignore = "safety #758"]
fn plastic_rigid_link_invariance() {
    let names = [
        "feature_matrix/2d-constraint--rigid-link--offset-beam-with",
        "fuzz_rich_2d/20840",
        "fuzz_rich_2d/20832",
        "fuzz_rich_2d/20772",
        "fuzz_rich_2d/20843",
    ];
    let cases: Vec<_> = corpus::cases_2d(Tier::Nightly, 20754)
        .into_iter()
        .filter(|c| names.contains(&c.name.as_str()))
        .collect();
    assert_eq!(cases.len(), names.len());
    let a: Vec<_> = ana2::rows()
        .into_iter()
        .filter(|a| a.name == "plastic collapse")
        .collect();
    let t = Tally::default();
    for c in cases {
        checks2::check_case(
            &checks2::Ctx {
                tally: &t,
                rng_seed: 20754,
            },
            &c,
            &a,
            crate::INV_2D,
        );
    }
    crate::finish("safety_758", &t);
}

#[test]
#[ignore = "safety #759"]
fn modal_warping_orthogonality() {
    let c = corpus::cases_3d(Tier::Nightly, 20754)
        .into_iter()
        .find(|c| c.name == "feature_matrix/3d-warping-torsion-restrained-cantilever-with")
        .unwrap();
    let a: Vec<_> = ana3::rows()
        .into_iter()
        .filter(|a| a.name == "modal")
        .collect();
    let t = Tally::default();
    checks3::check_case(
        &checks3::Ctx {
            tally: &t,
            rng_seed: 20754,
        },
        &c,
        &a,
        &["modal orthogonality"],
    );
    assert_eq!(
        t.symbol("modal", "modal orthogonality"),
        "✓",
        "{}",
        t.report()
    );
}

#[test]
#[ignore = "safety #760"]
fn time_history_reciprocity() {
    let c = corpus::cases_2d(Tier::Nightly, 20754)
        .into_iter()
        .find(|c| c.name == "fuzz_rich_2d/20843")
        .unwrap();
    let a: Vec<_> = ana2::rows()
        .into_iter()
        .filter(|a| a.name == "time history")
        .collect();
    let t = Tally::default();
    checks2::check_case(
        &checks2::Ctx {
            tally: &t,
            rng_seed: 20754,
        },
        &c,
        &a,
        crate::INV_2D,
    );
    assert_eq!(
        t.symbol("time history", "reciprocity"),
        "✓",
        "{}",
        t.report()
    );
}

#[test]
#[ignore = "safety #762"]
fn near_mechanism_units_and_orientation() {
    let c = corpus::cases_2d(Tier::Nightly, 20754)
        .into_iter()
        .find(|c| c.name == "fuzz_rich_2d/20800")
        .unwrap();
    let a: Vec<_> = ana2::rows()
        .into_iter()
        .filter(|a| a.name == "linear")
        .collect();
    let t = Tally::default();
    checks2::check_case(
        &checks2::Ctx {
            tally: &t,
            rng_seed: 20754,
        },
        &c,
        &a,
        crate::INV_2D,
    );
    assert_eq!(t.symbol("linear", "units"), "✓", "{}", t.report());
    crate::finish("safety_762", &t);
}
