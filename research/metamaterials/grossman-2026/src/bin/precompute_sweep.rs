//! Compute the 26 precomputed frames for the Grossman 2026 disorder sweep (eta = 0.00 .. 0.50).
//! Output is written to web/src/components/research/grossman2026/sweep-frames.json.

use std::fs::File;
use std::io::Write;
use serde::Serialize;
use serde_json::json;
use springs_grossman_2026::experiment::{run_periodic, run_strip, NetworkSpec, PeriodicResult, PeriodicSpec, StripResult, StripSpec};
use springs_grossman_2026::lattice::LatticeKind;
use springs_grossman_2026::network::SpringLaw;

#[derive(Serialize)]
struct SweepFrame {
    input: serde_json::Value,
    strip: StripResult,
    cell: PeriodicResult,
    ordered: Option<()>,
}

fn main() {
    let sweep: Vec<f64> = (0..=25).map(|i| (i as f64) * 0.02).collect();
    let mut frames = Vec::with_capacity(sweep.len());

    for (i, &eta) in sweep.iter().enumerate() {
        eprintln!("Solving frame {}/26: eta = {:.2}", i + 1, eta);
        let network = NetworkSpec {
            lattice: LatticeKind::Triangular { phi: 1.0, psi: 1.0 },
            eta,
            keep: 1.0,
            seed: 0,
            spring_law: SpringLaw::Hookean,
            k: 1.0,
            prestress: 0.0,
        };
        let strip = run_strip(&StripSpec {
            network: network.clone(),
            width: 13.0,
            height: 52.0,
            theta: 0.0,
            clamp_band: 0.5,
            strain: 0.01,
        }).expect("strip solve failed");
        let cell = run_periodic(&PeriodicSpec {
            network: network.clone(),
            n1: 26,
            n2: 26,
            strain: [0.0; 3],
        }).expect("periodic solve failed");

        let input = json!({
            "network": {
                "lattice": { "kind": "triangular", "phi": 1, "psi": 1 },
                "eta": eta,
                "keep": 1,
                "seed": 0,
                "spring_law": "hookean",
                "k": 1,
                "prestress": 0
            },
            "label": {
                "family": "foam",
                "eta": eta,
                "keep": 1,
                "seed": 0,
                "strain": 0.01
            },
            "theta": 0,
            "cells": 26,
            "phi": 2,
            "psi": 0.5
        });

        frames.push(SweepFrame {
            input,
            strip,
            cell,
            ordered: None,
        });
    }

    let default_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../web/public/research-wasm/grossman-2026/sweep-frames.json");
    let out_path = std::env::args().nth(1).unwrap_or_else(|| default_path.to_string());
    let json_bytes = serde_json::to_vec(&frames).expect("failed to serialize frames");
    let mut file = File::create(&out_path).expect("failed to create output file");
    file.write_all(&json_bytes).expect("failed to write output file");
    eprintln!("Wrote {} frames ({} bytes) to {}", frames.len(), json_bytes.len(), out_path);
}

