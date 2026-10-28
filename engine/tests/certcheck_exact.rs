//! Tests for the independent exact-rational certificate checker (`certcheck-exact`).
//!
//! Validates that the Rust exact-rational checker agrees with the Lean checker specification
//! across all 40+ positive, negative, and malformed samples in `verify/samples/`:
//! - Accepted bundles exit with status 0
//! - Tampered bundles (*.tampered-*.json) exit with status 1
//! - Malformed format (*.badformat*.json) exit with status 2

use dedaliano_engine::certify::checker_exact::{check_bundle_file, Verdict};
use std::fs;
use std::path::PathBuf;

fn samples_dir() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../verify/samples"))
}

#[test]
fn test_exact_checker_on_all_verify_samples() {
    let dir = samples_dir();
    assert!(dir.is_dir(), "samples directory must exist: {:?}", dir);

    let mut checked = 0;
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        if !name.ends_with(".json") || name.ends_with(".input.json") {
            continue;
        }

        let want_status = if name.contains(".tampered-") {
            1
        } else if name.contains(".badformat") {
            2
        } else {
            0
        };

        let (status, verdict) = check_bundle_file(&path);

        match want_status {
            0 => {
                assert_eq!(
                    status, 0,
                    "Expected ACCEPT for sample {}, got status {} with verdict {:?}",
                    name, status, verdict
                );
                match verdict {
                    Verdict::Accept(ref msg) => {
                        assert!(!msg.is_empty(), "Accept message should not be empty");
                    }
                    _ => panic!("Expected Verdict::Accept for {}", name),
                }
            }
            1 => {
                assert_eq!(
                    status, 1,
                    "Expected REJECT for tampered sample {}, got status {} with verdict {:?}",
                    name, status, verdict
                );
                match verdict {
                    Verdict::Reject(ref msg) => {
                        assert!(!msg.is_empty(), "Reject message should describe why");
                    }
                    _ => panic!("Expected Verdict::Reject for {}", name),
                }
            }
            2 => {
                assert_eq!(
                    status, 2,
                    "Expected BAD_FORMAT (2) for malformed sample {}, got status {} with verdict {:?}",
                    name, status, verdict
                );
                match verdict {
                    Verdict::BadFormat(ref msg) => {
                        assert!(!msg.is_empty(), "BadFormat message should describe parse error");
                    }
                    _ => panic!("Expected Verdict::BadFormat for {}", name),
                }
            }
            _ => unreachable!(),
        }

        checked += 1;
    }

    assert!(
        checked >= 30,
        "Expected at least 30 samples to be checked in verify/samples, found {}",
        checked
    );
}

#[test]
fn test_exact_checker_specific_linear_tampered() {
    let dir = samples_dir();
    let mu_tampered = dir.join("ex-3d-portal-frame.tampered-mu.json");
    let (status, verdict) = check_bundle_file(&mu_tampered);
    assert_eq!(status, 1);
    match verdict {
        Verdict::Reject(msg) => {
            assert!(
                msg.contains("Gershgorin") || msg.contains("mu") || msg.contains("lambda_min"),
                "Expected rejection reason to mention Gershgorin or mu, got: {}",
                msg
            );
        }
        _ => panic!("Expected reject"),
    }
}

