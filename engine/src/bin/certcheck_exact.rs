//! Standalone executable for the independent exact-rational certificate checker (`certcheck-exact`).
//!
//! Re-checks certificate bundles with exact rational and dyadic arithmetic (docs/verification/CERTIFY.md).
//! Run beside the verified Lean checker (`lake exe certcheck`) to guarantee dual-checker agreement.
//!
//! Exit status:
//! - 0 if every bundle is accepted
//! - 1 if any bundle is rejected (tampered claims)
//! - 2 on a parse error or unknown format

use dedaliano_engine::certify::checker_exact::{check_bundle_file, Verdict};
use std::env;
use std::path::Path;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: certcheck-exact <bundle.json>...");
        exit(2);
    }

    let mut max_status: u32 = 0;

    for arg in &args {
        let path = Path::new(arg);
        let (status, verdict) = check_bundle_file(path);
        match verdict {
            Verdict::Accept(info) => {
                println!("ACCEPT {}: {}", arg, info);
            }
            Verdict::Reject(why) => {
                println!("REJECT {}: {}", arg, why);
            }
            Verdict::BadFormat(err) => {
                eprintln!("{}: parse error: {}", arg, err);
            }
        }
        max_status = max_status.max(status);
    }

    exit(max_status as i32);
}

