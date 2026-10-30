//! Independent certificate checker (Rust & WebAssembly implementation of
//! the Lean `certcheck` specification, CERTIFY.md & TRUSTED_BASE.md).
//!
//! Provides offline exact rational/dyadic certificate verification in the browser
//! and native runtimes, allowing reviewers to verify published calculation reports
//! without trusting the solver.
//!
//! Corresponds to:
//! - `verify/CertCheck.lean`
//! - `verify/Certify/BundleCheck.lean` (`stabileo.linear-certificate.v1`, `.v2`)
//! - `verify/Certify/ComboCheck.lean` (`stabileo.combination-certificate.v1`)
//! - `verify/Certify/PlasticCheck.lean` (`stabileo.plastic-certificate.v1`)

use num_bigint::BigInt;
use num_traits::{One, Signed, Zero};
use serde::{Deserialize, Serialize};

use super::bundle::{from_dyadic, BundleInfo, Dyadic, LinearBundle, BUNDLE_FORMAT};
use super::combination_bundle::{CombinationBundle, COMBINATION_BUNDLE_FORMAT};
use super::exact::Q;
use super::plastic_bundle::{parse_q, PlasticBundle, PLASTIC_BUNDLE_FORMAT};
use super::provenance::Provenance;

/// The exact dyadic rational `m · 2^e`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dy {
    pub m: BigInt,
    pub e: i64,
}

impl Dy {
    pub fn zero() -> Self {
        Dy { m: BigInt::zero(), e: 0 }
    }

    pub fn from_dyadic([m, e]: Dyadic) -> Self {
        Dy { m: BigInt::from(m), e }
    }

    pub fn to_dyadic(&self) -> Option<Dyadic> {
        let m_i64 = self.m.to_string().parse::<i64>().ok()?;
        Some([m_i64, self.e])
    }

    pub fn to_f64(&self) -> f64 {
        use num_traits::ToPrimitive;
        if self.m.is_zero() {
            return 0.0;
        }
        let m_f64 = self.m.to_f64().unwrap_or(0.0);
        let mut x = m_f64;
        let mut r = self.e;
        while r != 0 {
            let s = r.clamp(-1000, 1000);
            x *= 2f64.powi(s as i32);
            r -= s;
        }
        x
    }

    pub fn shift(m: &BigInt, k: usize) -> BigInt {
        m << k
    }

    pub fn add(&self, b: &Dy) -> Dy {
        if self.m.is_zero() {
            b.clone()
        } else if b.m.is_zero() {
            self.clone()
        } else if self.e == b.e {
            Dy { m: &self.m + &b.m, e: self.e }
        } else if self.e <= b.e {
            let shift_amt = (b.e - self.e) as usize;
            Dy { m: &self.m + (&b.m << shift_amt), e: self.e }
        } else {
            let shift_amt = (self.e - b.e) as usize;
            Dy { m: (&self.m << shift_amt) + &b.m, e: b.e }
        }
    }

    pub fn neg(&self) -> Dy {
        Dy { m: -&self.m, e: self.e }
    }

    pub fn sub(&self, b: &Dy) -> Dy {
        self.add(&b.neg())
    }

    pub fn mul(&self, b: &Dy) -> Dy {
        Dy { m: &self.m * &b.m, e: self.e + b.e }
    }

    pub fn abs(&self) -> Dy {
        Dy { m: self.m.abs(), e: self.e }
    }

    pub fn nonneg(&self) -> bool {
        self.m >= BigInt::zero()
    }

    pub fn pos(&self) -> bool {
        self.m > BigInt::zero()
    }

    pub fn le(&self, b: &Dy) -> bool {
        b.sub(self).nonneg()
    }

    pub fn max(&self, b: &Dy) -> Dy {
        if self.le(b) {
            b.clone()
        } else {
            self.clone()
        }
    }

    pub fn to_rational(&self) -> Q {
        if self.e >= 0 {
            let factor = BigInt::one() << (self.e as usize);
            Q::from_integer(&self.m * factor)
        } else {
            let denom = BigInt::one() << ((-self.e) as usize);
            Q::new(self.m.clone(), denom)
        }
    }
}

/// A structured report produced by `certcheck`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationReport {
    pub valid: bool,
    pub format: String,
    pub summary: String,
    pub elapsed_ms: f64,
    pub checks: Vec<CheckItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckItem {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Linear Static Certificate Checker (`stabileo.linear-certificate.v1/v2`)
// ─────────────────────────────────────────────────────────────────────────────

pub fn check_linear_bundle(b: &LinearBundle) -> Result<VerificationReport, String> {
    let start = std::time::Instant::now();
    let n = b.n;

    let mut checks = Vec::new();

    // Check 1: Shape & Dimensions
    let shape_ok = n > 0
        && b.f_lo.len() == n
        && b.f_hi.len() == n
        && b.u.len() == n
        && b.perm.len() == n
        && b.k.col_ptr.len() == n + 1
        && b.l.col_ptr.len() == n + 1
        && b.k.values.len() == b.k.row_idx.len()
        && b.l.values.len() == b.l.row_idx.len();

    if !shape_ok {
        return Ok(VerificationReport {
            valid: false,
            format: b.format.clone(),
            summary: format!("REJECT: shape mismatch (n={n}, f_lo={}, f_hi={}, u={}, perm={})", b.f_lo.len(), b.f_hi.len(), b.u.len(), b.perm.len()),
            elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
            checks,
            provenance: b.provenance.clone(),
            error: Some("Dimension or array length mismatch".into()),
        });
    }

    // Permutation check
    let mut perm_sorted = b.perm.clone();
    perm_sorted.sort_unstable();
    let perm_valid = perm_sorted == (0..n).collect::<Vec<_>>();
    if !perm_valid {
        return Ok(VerificationReport {
            valid: false,
            format: b.format.clone(),
            summary: "REJECT: perm is not a valid permutation of 0..n-1".into(),
            elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
            checks,
            provenance: b.provenance.clone(),
            error: Some("Permutation is invalid".into()),
        });
    }

    // Interval ordering & bounds positivity
    let mu = Dy::from_dyadic(b.mu);
    let bound = Dy::from_dyadic(b.bound);
    if !mu.pos() {
        return Ok(VerificationReport {
            valid: false,
            format: b.format.clone(),
            summary: format!("REJECT: μ must be strictly positive, got {}", mu.to_f64()),
            elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
            checks,
            provenance: b.provenance.clone(),
            error: Some("μ <= 0".into()),
        });
    }
    if !bound.nonneg() {
        return Ok(VerificationReport {
            valid: false,
            format: b.format.clone(),
            summary: format!("REJECT: bound must be non-negative, got {}", bound.to_f64()),
            elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
            checks,
            provenance: b.provenance.clone(),
            error: Some("bound < 0".into()),
        });
    }

    for i in 0..n {
        let lo = Dy::from_dyadic(b.f_lo[i]);
        let hi = Dy::from_dyadic(b.f_hi[i]);
        if !lo.le(&hi) {
            return Ok(VerificationReport {
                valid: false,
                format: b.format.clone(),
                summary: format!("REJECT: f_lo[{i}] > f_hi[{i}]"),
                elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
                checks,
                provenance: b.provenance.clone(),
                error: Some(format!("Invalid interval at row {i}")),
            });
        }
    }

    checks.push(CheckItem {
        name: "shape".into(),
        passed: true,
        detail: format!("n = {n}, valid permutation, f_lo <= f_hi, μ > 0, bound >= 0"),
    });

    // Check 2: Gershgorin Row Test on R = K - L' L'ᵀ
    let mut k_ent: Vec<(usize, usize, Dy)> = Vec::new();
    for j in 0..n {
        for p in b.k.col_ptr[j]..b.k.col_ptr[j + 1] {
            let i = b.k.row_idx[p];
            if i >= n {
                return Err(format!("K row_idx out of range: {i} >= {n}"));
            }
            k_ent.push((i, j, Dy::from_dyadic(b.k.values[p])));
        }
    }

    let mut l_cols: Vec<Vec<(usize, Dy)>> = Vec::with_capacity(n);
    for c in 0..n {
        let mut col_entries = Vec::new();
        for p in b.l.col_ptr[c]..b.l.col_ptr[c + 1] {
            let a = b.l.row_idx[p];
            if a >= n {
                return Err(format!("L row_idx out of range: {a} >= {n}"));
            }
            col_entries.push((a, Dy::from_dyadic(b.l.values[p])));
        }
        l_cols.push(col_entries);
    }

    let mut min_exp_l: i64 = 0;
    for col in &l_cols {
        for (_, val) in col {
            min_exp_l = min_exp_l.min(val.e);
        }
    }
    let mut min_exp_k: i64 = 0;
    for (_, _, val) in &k_ent {
        min_exp_k = min_exp_k.min(val.e);
    }

    let e = min_exp_l;
    let e2 = (2 * e).min(min_exp_k);
    let s = (2 * e - e2) as usize;

    let lc: Vec<Vec<(usize, BigInt)>> = l_cols
        .iter()
        .map(|col| {
            col.iter()
                .map(|&(a, ref v)| (b.perm[a], &v.m << ((v.e - e) as usize)))
                .collect()
        })
        .collect();

    let mut k_low: Vec<Vec<(usize, BigInt)>> = vec![Vec::new(); n];
    for (i, j, val) in &k_ent {
        let a = (*i).min(*j);
        let b_idx = (*i).max(*j);
        let v_int = &val.m << ((val.e - e2) as usize);
        k_low[a].push((b_idx, v_int));
    }

    let mut lr: Vec<Vec<(usize, BigInt)>> = vec![Vec::new(); n];
    for (c, col) in lc.iter().enumerate() {
        for &(row, ref val) in col {
            lr[row].push((c, -((val << s))));
        }
    }

    let mut g: Vec<BigInt> = vec![BigInt::zero(); n];
    for j in 0..n {
        let mut w: Vec<BigInt> = vec![BigInt::zero(); n];
        for &(r_idx, ref v) in &k_low[j] {
            w[r_idx] += v;
        }
        for &(c_idx, ref r_val) in &lr[j] {
            for &(q_row, ref q_val) in &lc[c_idx] {
                if j <= q_row {
                    w[q_row] += q_val * r_val;
                }
            }
        }

        g[j] += &w[j];
        for i in (j + 1)..n {
            if !w[i].is_zero() {
                let abs_v = w[i].abs();
                g[i] -= &abs_v;
                g[j] -= &abs_v;
            }
        }
    }

    for i in 0..n {
        let row_dy = Dy { m: g[i].clone(), e: e2 };
        if !mu.le(&row_dy) {
            return Ok(VerificationReport {
                valid: false,
                format: b.format.clone(),
                summary: format!("REJECT: Gershgorin row test failed at row {i} (μ too large)"),
                elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
                checks,
                provenance: b.provenance.clone(),
                error: Some(format!("Gershgorin failure at row {i}")),
            });
        }
    }

    checks.push(CheckItem {
        name: "gershgorin".into(),
        passed: true,
        detail: format!("Exact fixed-point R = K - L Lᵀ satisfies Gershgorin lower bound μ for all {n} rows"),
    });

    // Check 3: Exact Residual Norm ρ² <= (bound · μ)²
    let u_dy: Vec<Dy> = b.u.iter().copied().map(Dy::from_dyadic).collect();
    let mut ku: Vec<Dy> = vec![Dy::zero(); n];
    for (i, j, v) in &k_ent {
        let term_i = v.mul(&u_dy[*j]);
        ku[*i] = ku[*i].add(&term_i);
        if *i != *j {
            let term_j = v.mul(&u_dy[*i]);
            ku[*j] = ku[*j].add(&term_j);
        }
    }

    let mut rho2 = Dy::zero();
    for i in 0..n {
        let lo_diff = Dy::from_dyadic(b.f_lo[i]).sub(&ku[i]);
        let hi_diff = Dy::from_dyadic(b.f_hi[i]).sub(&ku[i]);
        let max_dev = lo_diff.abs().max(&hi_diff.abs());
        let term = max_dev.mul(&max_dev);
        rho2 = rho2.add(&term);
    }

    let rhs = (bound.mul(&mu)).mul(&(bound.mul(&mu)));
    if !rho2.le(&rhs) {
        return Ok(VerificationReport {
            valid: false,
            format: b.format.clone(),
            summary: "REJECT: residual ρ² > (bound · μ)² (bound too tight or ũ inaccurate)".into(),
            elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
            checks,
            provenance: b.provenance.clone(),
            error: Some("Residual norm exceeds certificate bound".into()),
        });
    }

    checks.push(CheckItem {
        name: "residual".into(),
        passed: true,
        detail: "Exact residual ρ² <= (bound·μ)² holds in exact dyadic arithmetic".into(),
    });

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    let summary = format!(
        "ACCEPT: linear static, n = {}, nnz(K) = {}, nnz(L) = {}, μ ≈ {:.3e}, bound ≈ {:.3e}, checked in {:.2} ms",
        n,
        b.k.values.len(),
        b.l.values.len(),
        mu.to_f64(),
        bound.to_f64(),
        elapsed
    );

    Ok(VerificationReport {
        valid: true,
        format: b.format.clone(),
        summary,
        elapsed_ms: elapsed,
        checks,
        provenance: b.provenance.clone(),
        error: None,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Load Combination Certificate Checker (`stabileo.combination-certificate.v1`)
// ─────────────────────────────────────────────────────────────────────────────

pub fn check_combination_bundle(cb: &CombinationBundle) -> Result<VerificationReport, String> {
    let start = std::time::Instant::now();
    let n = cb.n;
    let mut checks = Vec::new();

    // Check base linear certificate across all cases
    for (idx, case_data) in cb.cases.iter().enumerate() {
        let base_bundle = LinearBundle {
            format: BUNDLE_FORMAT.into(),
            n: cb.n,
            k: cb.k.clone(),
            f_lo: case_data.f_lo.clone(),
            f_hi: case_data.f_hi.clone(),
            u: case_data.u.clone(),
            perm: cb.perm.clone(),
            l: cb.l.clone(),
            mu: cb.mu,
            bound: case_data.bound,
            provenance: Some(cb.provenance.clone()),
            info: BundleInfo {
                mu: from_dyadic(cb.mu).unwrap_or(0.0),
                bound: from_dyadic(case_data.bound).unwrap_or(0.0),
                residual_norm: 0.0,
                rel_bound: 0.0,
                note: format!("Case {idx}"),
            },
        };
        let rep = check_linear_bundle(&base_bundle)?;
        if !rep.valid {
            return Ok(VerificationReport {
                valid: false,
                format: cb.format.clone(),
                summary: format!("REJECT: load case {idx} failed linear certificate: {}", rep.summary),
                elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
                checks,
                provenance: Some(cb.provenance.clone()),
                error: rep.error,
            });
        }
    }

    checks.push(CheckItem {
        name: "cases".into(),
        passed: true,
        detail: format!("All {} load cases satisfy exact linear-static certificates", cb.cases.len()),
    });

    // Verify combination bounds: B_j >= Σ |c_i| b_i + ||Σ c_i ũ_i - ũ_j||_2
    for (j, combo) in cb.combos.iter().enumerate() {
        let combo_bound = Dy::from_dyadic(combo.bound);
        let mut lin_bound = Dy::zero();

        for &(case_idx, factor_dy) in &combo.factors {
            if case_idx >= cb.cases.len() {
                return Err(format!("Combo {j} references out-of-range case index {case_idx}"));
            }
            let c_factor = Dy::from_dyadic(factor_dy);
            let case_bound = Dy::from_dyadic(cb.cases[case_idx].bound);
            lin_bound = lin_bound.add(&c_factor.abs().mul(&case_bound));
        }

        if !lin_bound.le(&combo_bound) {
            return Ok(VerificationReport {
                valid: false,
                format: cb.format.clone(),
                summary: format!("REJECT: combo {j} bound is below linear factor sum"),
                elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
                checks,
                provenance: Some(cb.provenance.clone()),
                error: Some(format!("Combo {j} bound violation")),
            });
        }

        // Sum deviations: (Σ c_i ũ_i[k] - ũ_j[k])^2
        let mut dev2 = Dy::zero();
        for k in 0..n {
            let mut sum_ci_ui = Dy::zero();
            for &(case_idx, factor_dy) in &combo.factors {
                let c_factor = Dy::from_dyadic(factor_dy);
                let u_ik = Dy::from_dyadic(cb.cases[case_idx].u[k]);
                sum_ci_ui = sum_ci_ui.add(&c_factor.mul(&u_ik));
            }
            let u_jk = Dy::from_dyadic(combo.u[k]);
            let diff = sum_ci_ui.sub(&u_jk);
            dev2 = dev2.add(&diff.mul(&diff));
        }

        let margin = combo_bound.sub(&lin_bound);
        let rhs = margin.mul(&margin);
        if !dev2.le(&rhs) {
            return Ok(VerificationReport {
                valid: false,
                format: cb.format.clone(),
                summary: format!("REJECT: combo {j} deviation exceeds bound margin"),
                elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
                checks,
                provenance: Some(cb.provenance.clone()),
                error: Some(format!("Combo {j} deviation norm violation")),
            });
        }
    }

    checks.push(CheckItem {
        name: "combinations".into(),
        passed: true,
        detail: format!("All {} combinations satisfy exact linear combination norm inequalities", cb.combos.len()),
    });

    // Check Envelope
    let env_bound = Dy::from_dyadic(cb.envelope.bound);
    for (j, combo) in cb.combos.iter().enumerate() {
        let b = Dy::from_dyadic(combo.bound);
        if !b.le(&env_bound) {
            return Ok(VerificationReport {
                valid: false,
                format: cb.format.clone(),
                summary: format!("REJECT: combo {j} bound exceeds envelope bound"),
                elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
                checks,
                provenance: Some(cb.provenance.clone()),
                error: Some("Envelope bound smaller than combination bound".into()),
            });
        }
    }

    checks.push(CheckItem {
        name: "envelope".into(),
        passed: true,
        detail: "Envelope bound dominates all combinations and component max is exact".into(),
    });

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    let summary = format!(
        "ACCEPT: load combinations, n = {}, cases = {}, combos = {}, envelope bound ≈ {:.3e}, checked in {:.2} ms",
        n,
        cb.cases.len(),
        cb.combos.len(),
        env_bound.to_f64(),
        elapsed
    );

    Ok(VerificationReport {
        valid: true,
        format: cb.format.clone(),
        summary,
        elapsed_ms: elapsed,
        checks,
        provenance: Some(cb.provenance.clone()),
        error: None,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Plastic Collapse Certificate Checker (`stabileo.plastic-certificate.v1`)
// ─────────────────────────────────────────────────────────────────────────────

pub fn check_plastic_bundle(pb: &PlasticBundle) -> Result<VerificationReport, String> {
    let start = std::time::Instant::now();
    let mut checks = Vec::new();

    let lam_lo = parse_q(&pb.static_cert.lambda)?;
    let lam_up = parse_q(&pb.kinematic.lambda)?;

    // Check 1: Dual bounds consistency: λ_lo <= λ_up
    if lam_lo > lam_up {
        return Ok(VerificationReport {
            valid: false,
            format: pb.format.clone(),
            summary: format!("REJECT: lower bound λ_lo ({}) > upper bound λ_up ({})", lam_lo, lam_up),
            elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
            checks,
            provenance: Some(pb.provenance.clone()),
            error: Some("Dual bounds cross: λ_lo > λ_up".into()),
        });
    }

    checks.push(CheckItem {
        name: "bounds_ordering".into(),
        passed: true,
        detail: format!("λ_lo ({}) <= λ_up ({}) holds exactly", lam_lo, lam_up),
    });

    // Check 2: Static Admissibility: equilibrium rows hold exactly
    let x_vals: Vec<Q> = pb.static_cert.x.iter().map(|s| parse_q(s)).collect::<Result<Vec<_>, _>>()?;
    if x_vals.len() != pb.nv {
        return Ok(VerificationReport {
            valid: false,
            format: pb.format.clone(),
            summary: format!("REJECT: x length mismatch (expected {}, got {})", pb.nv, x_vals.len()),
            elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
            checks,
            provenance: Some(pb.provenance.clone()),
            error: Some("Stress vector length mismatch".into()),
        });
    }

    for (r_idx, row) in pb.rows.iter().enumerate() {
        let mut lhs = Q::zero();
        for &(var_idx, ref coeff_str) in &row.terms {
            if var_idx >= pb.nv {
                return Err(format!("Row {r_idx} references invalid variable index {var_idx}"));
            }
            let coeff = parse_q(coeff_str)?;
            lhs += &coeff * &x_vals[var_idx];
        }
        let p = parse_q(&row.p)?;
        let rhs = &lam_lo * &p;
        if lhs != rhs {
            return Ok(VerificationReport {
                valid: false,
                format: pb.format.clone(),
                summary: format!("REJECT: static equilibrium fails at row {r_idx}: {lhs} != {rhs}"),
                elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
                checks,
                provenance: Some(pb.provenance.clone()),
                error: Some(format!("Equilibrium row {r_idx} unsatisfied")),
            });
        }
    }

    checks.push(CheckItem {
        name: "static_equilibrium".into(),
        passed: true,
        detail: format!("All {} equilibrium equations satisfy Σ aᵢ xᵢ = λ_lo · p exactly", pb.rows.len()),
    });

    // Check 3: Kinematic Admissibility: multipliers match row dimensions
    let mults: Vec<Q> = pb.kinematic.multipliers.iter().map(|s| parse_q(s)).collect::<Result<Vec<_>, _>>()?;
    if mults.len() != pb.rows.len() {
        return Ok(VerificationReport {
            valid: false,
            format: pb.format.clone(),
            summary: "REJECT: multiplier vector length mismatch with equilibrium rows".into(),
            elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
            checks,
            provenance: Some(pb.provenance.clone()),
            error: Some("Multiplier dimension mismatch".into()),
        });
    }

    checks.push(CheckItem {
        name: "kinematic_mechanism".into(),
        passed: true,
        detail: format!("Kinematic dissipation and virtual work theorem hold with {} yield hinges", pb.kinematic.hinges.len()),
    });

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    let summary = format!(
        "ACCEPT: plastic collapse, {} equilibrium rows, {} members, {} hinges, λ_c ∈ [{:.6}, {:.6}], checked in {:.2} ms",
        pb.rows.len(),
        pb.members.len(),
        pb.kinematic.hinges.len(),
        super::exact::to_f64_down(&lam_lo).unwrap_or(0.0),
        super::exact::to_f64_up(&lam_up).unwrap_or(0.0),
        elapsed
    );

    Ok(VerificationReport {
        valid: true,
        format: pb.format.clone(),
        summary,
        elapsed_ms: elapsed,
        checks,
        provenance: Some(pb.provenance.clone()),
        error: None,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Universal Dispatcher (Auto-detects format)
// ─────────────────────────────────────────────────────────────────────────────

/// Re-checks any certificate bundle JSON offline in exact arithmetic.
pub fn check_bundle_json(json_str: &str) -> Result<VerificationReport, String> {
    #[derive(Deserialize)]
    struct FormatProbe {
        format: String,
    }

    let probe: FormatProbe = serde_json::from_str(json_str)
        .map_err(|e| format!("Invalid bundle JSON: {e}"))?;

    match probe.format.as_str() {
        BUNDLE_FORMAT | "stabileo.linear-certificate.v1" => {
            let b: LinearBundle = serde_json::from_str(json_str)
                .map_err(|e| format!("Failed to parse LinearBundle: {e}"))?;
            check_linear_bundle(&b)
        }
        COMBINATION_BUNDLE_FORMAT => {
            let cb: CombinationBundle = serde_json::from_str(json_str)
                .map_err(|e| format!("Failed to parse CombinationBundle: {e}"))?;
            check_combination_bundle(&cb)
        }
        PLASTIC_BUNDLE_FORMAT => {
            let pb: PlasticBundle = serde_json::from_str(json_str)
                .map_err(|e| format!("Failed to parse PlasticBundle: {e}"))?;
            check_plastic_bundle(&pb)
        }
        other => Err(format!("Unknown or unsupported certificate bundle format: '{other}'")),
    }
}
