//! Second, independent certificate checker in exact rational / dyadic arithmetic.
//!
//! Re-checks linear-static, plastic-collapse, and load-combination certificate bundles
//! independently of the Lean checker (`verify/CertCheck.lean`).
//!
//! Soundness:
//! - For linear static: exact dyadic arithmetic (`Dy`), Gram residual `R = K - L' L'^T`,
//!   Gershgorin row test `μ ≤ Rᵢᵢ − Σ_{j≠i} |Rᵢⱼ|`, and residual norm bound `ρ² ≤ (b μ)²`.
//! - For plastic collapse: exact rational arithmetic (`BigRational`), static equilibrium and
//!   Bernstein-polynomial yield bound, kinematic virtual-work identity and dissipation.
//! - For load combinations: exact case residuals, combination displacement/quantity bounds,
//!   and envelope bounds.
//!
//! Exit codes / status:
//! - 0: ACCEPT
//! - 1: REJECT (tampered / violated claim)
//! - 2: BAD FORMAT (unknown format / parse failure)

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Signed, Zero};
use serde_json::Value;
use std::cmp::Ordering;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    Accept(String),
    Reject(String),
    BadFormat(String),
}

// -----------------------------------------------------------------------------
// Dyadic Arithmetic: m · 2^e (exact, no gcd)
// -----------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dy {
    pub m: BigInt,
    pub e: i64,
}

impl Dy {
    pub fn zero() -> Self {
        Dy {
            m: BigInt::zero(),
            e: 0,
        }
    }

    pub fn new(m: BigInt, e: i64) -> Self {
        if m.is_zero() {
            Dy::zero()
        } else {
            Dy { m, e }
        }
    }

    pub fn is_zero(&self) -> bool {
        self.m.is_zero()
    }

    pub fn is_pos(&self) -> bool {
        self.m > BigInt::zero()
    }

    pub fn is_nonneg(&self) -> bool {
        self.m >= BigInt::zero()
    }

    pub fn neg(&self) -> Self {
        Dy::new(-&self.m, self.e)
    }

    pub fn abs(&self) -> Self {
        Dy::new(self.m.abs(), self.e)
    }

    pub fn add(&self, other: &Self) -> Self {
        if self.is_zero() {
            return other.clone();
        }
        if other.is_zero() {
            return self.clone();
        }
        match self.e.cmp(&other.e) {
            Ordering::Equal => Dy::new(&self.m + &other.m, self.e),
            Ordering::Less => {
                let shift = (other.e - self.e) as usize;
                Dy::new(&self.m + (&other.m << shift), self.e)
            }
            Ordering::Greater => {
                let shift = (self.e - other.e) as usize;
                Dy::new((&self.m << shift) + &other.m, other.e)
            }
        }
    }

    pub fn sub(&self, other: &Self) -> Self {
        self.add(&other.neg())
    }

    pub fn mul(&self, other: &Self) -> Self {
        if self.is_zero() || other.is_zero() {
            Dy::zero()
        } else {
            Dy::new(&self.m * &other.m, self.e + other.e)
        }
    }

    pub fn le(&self, other: &Self) -> bool {
        other.sub(self).is_nonneg()
    }

    pub fn max(&self, other: &Self) -> Self {
        if self.le(other) {
            other.clone()
        } else {
            self.clone()
        }
    }
}

// -----------------------------------------------------------------------------
// Rational Helpers for Plastic Collapse
// -----------------------------------------------------------------------------

fn parse_rat(v: &Value) -> Result<BigRational, String> {
    match v {
        Value::String(s) => {
            if let Some((n_str, d_str)) = s.split_once('/') {
                let n: BigInt = n_str
                    .trim()
                    .parse()
                    .map_err(|e| format!("bad rational numerator '{}': {}", n_str, e))?;
                let d: BigInt = d_str
                    .trim()
                    .parse()
                    .map_err(|e| format!("bad rational denominator '{}': {}", d_str, e))?;
                if d.is_zero() {
                    return Err(format!("zero denominator in rational '{}'", s));
                }
                Ok(BigRational::new(n, d))
            } else {
                let n: BigInt = s
                    .trim()
                    .parse()
                    .map_err(|e| format!("bad rational integer '{}': {}", s, e))?;
                Ok(BigRational::from_integer(n))
            }
        }
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(BigRational::from_integer(BigInt::from(i)))
            } else if let Some(f) = n.as_f64() {
                BigRational::from_float(f).ok_or_else(|| format!("non-finite float {}", f))
            } else {
                Err(format!("unsupported number format: {}", n))
            }
        }
        _ => Err(format!("expected string or number for rational, got {}", v)),
    }
}

fn parse_dyadic(v: &Value) -> Result<Dy, String> {
    let arr = v.as_array().ok_or_else(|| "expected dyadic [m, e] array".to_string())?;
    if arr.len() != 2 {
        return Err(format!("expected [m, e], got length {}", arr.len()));
    }
    let m = arr[0]
        .as_i64()
        .ok_or_else(|| "dyadic mantissa not i64".to_string())?;
    let e = arr[1]
        .as_i64()
        .ok_or_else(|| "dyadic exponent not i64".to_string())?;
    Ok(Dy::new(BigInt::from(m), e))
}

fn parse_dy_array(v: &Value) -> Result<Vec<Dy>, String> {
    let arr = v.as_array().ok_or_else(|| "expected array of dyadics".to_string())?;
    arr.iter().map(parse_dyadic).collect()
}

// -----------------------------------------------------------------------------
// Linear Static Checker
// -----------------------------------------------------------------------------

struct CscMatrixDy {
    col_ptr: Vec<usize>,
    row_idx: Vec<usize>,
    values: Vec<Dy>,
}

struct LinearCoreData {
    n: usize,
    k: CscMatrixDy,
    perm: Vec<usize>,
    l: CscMatrixDy,
    mu: Dy,
}

struct LinearBundleData {
    n: usize,
    k: CscMatrixDy,
    f_lo: Vec<Dy>,
    f_hi: Vec<Dy>,
    u: Vec<Dy>,
    perm: Vec<usize>,
    l: CscMatrixDy,
    mu: Dy,
    bound: Dy,
}

fn parse_csc(v: &Value, n: usize) -> Result<CscMatrixDy, String> {
    let cp_arr = v.get("colPtr").and_then(|v| v.as_array()).ok_or("missing colPtr")?;
    let ri_arr = v.get("rowIdx").and_then(|v| v.as_array()).ok_or("missing rowIdx")?;
    let vs_arr = v.get("values").and_then(|v| v.as_array()).ok_or("missing values")?;

    let col_ptr: Vec<usize> = cp_arr
        .iter()
        .map(|x| x.as_u64().map(|u| u as usize).ok_or("bad colPtr item"))
        .collect::<Result<_, _>>()?;
    let row_idx: Vec<usize> = ri_arr
        .iter()
        .map(|x| x.as_u64().map(|u| u as usize).ok_or("bad rowIdx item"))
        .collect::<Result<_, _>>()?;
    let values = parse_dy_array(&Value::Array(vs_arr.clone()))?;

    if col_ptr.len() != n + 1 {
        return Err(format!("colPtr length {} != n + 1 ({})", col_ptr.len(), n + 1));
    }
    if row_idx.len() != values.len() {
        return Err("rowIdx and values length mismatch".to_string());
    }
    if col_ptr[0] != 0 || col_ptr[n] != row_idx.len() {
        return Err("colPtr does not span rowIdx".to_string());
    }
    for j in 0..n {
        if col_ptr[j] > col_ptr[j + 1] || col_ptr[j + 1] > row_idx.len() {
            return Err(format!("colPtr not monotone at column {}", j));
        }
    }

    Ok(CscMatrixDy {
        col_ptr,
        row_idx,
        values,
    })
}

fn parse_linear_core(j: &Value) -> Result<LinearCoreData, String> {
    let n = j.get("n").and_then(|v| v.as_u64()).ok_or("missing or invalid n")? as usize;
    let k = parse_csc(j.get("k").ok_or("missing k")?, n)?;
    let l = parse_csc(j.get("l").ok_or("missing l")?, n)?;
    let perm_arr = j.get("perm").and_then(|v| v.as_array()).ok_or("missing perm")?;
    let perm: Vec<usize> = perm_arr
        .iter()
        .map(|x| x.as_u64().map(|u| u as usize).ok_or("bad perm item"))
        .collect::<Result<_, _>>()?;
    let mu = parse_dyadic(j.get("mu").ok_or("missing mu")?)?;

    Ok(LinearCoreData {
        n,
        k,
        perm,
        l,
        mu,
    })
}

fn parse_linear_bundle(j: &Value) -> Result<LinearBundleData, String> {
    let core = parse_linear_core(j)?;
    let f_lo = parse_dy_array(j.get("fLo").ok_or("missing fLo")?)?;
    let f_hi = parse_dy_array(j.get("fHi").ok_or("missing fHi")?)?;
    let u = parse_dy_array(j.get("u").ok_or("missing u")?)?;
    let bound = parse_dyadic(j.get("bound").ok_or("missing bound")?)?;

    Ok(LinearBundleData {
        n: core.n,
        k: core.k,
        f_lo,
        f_hi,
        u,
        perm: core.perm,
        l: core.l,
        mu: core.mu,
        bound,
    })
}

fn check_linear_bundle(d: &LinearBundleData) -> (u32, Verdict) {
    let n = d.n;
    if n == 0
        || d.f_lo.len() != n
        || d.f_hi.len() != n
        || d.u.len() != n
        || d.perm.len() != n
        || !d.mu.is_pos()
        || !d.bound.is_nonneg()
    {
        return (
            1,
            Verdict::Reject("shape (sizes, indices, fLo <= fHi, mu > 0, bound >= 0)".to_string()),
        );
    }

    // Permutation check: must be a permutation of 0..n
    let mut seen = vec![false; n];
    for &p in &d.perm {
        if p >= n || seen[p] {
            return (
                1,
                Verdict::Reject("perm is not a valid permutation".to_string()),
            );
        }
        seen[p] = true;
    }

    // fLo <= fHi
    for i in 0..n {
        if !d.f_lo[i].le(&d.f_hi[i]) {
            return (
                1,
                Verdict::Reject("shape: fLo[i] > fHi[i]".to_string()),
            );
        }
    }

    // Index check on K and L
    for &r in &d.k.row_idx {
        if r >= n {
            return (1, Verdict::Reject("K row index out of range".to_string()));
        }
    }
    for &r in &d.l.row_idx {
        if r >= n {
            return (1, Verdict::Reject("L row index out of range".to_string()));
        }
    }

    // 1. Gershgorin check on R = K - L' L'^T
    // Compute E and E2 scales exactly as in Lean BundleCheck
    let mut min_exp_l = 0i64;
    for v in &d.l.values {
        if !v.is_zero() {
            min_exp_l = min_exp_l.min(v.e);
        }
    }
    let mut min_exp_k = 0i64;
    for v in &d.k.values {
        if !v.is_zero() {
            min_exp_k = min_exp_k.min(v.e);
        }
    }
    let e = min_exp_l;
    let e2 = (2 * e).min(min_exp_k);
    let s = (2 * e - e2) as usize;

    // Fixed-point scaling of L'
    let mut lc: Vec<Vec<(usize, BigInt)>> = vec![Vec::new(); n];
    let mut lr: Vec<Vec<(usize, BigInt)>> = vec![Vec::new(); n];

    for c in 0..n {
        for p in d.l.col_ptr[c]..d.l.col_ptr[c + 1] {
            let row = d.perm[d.l.row_idx[p]];
            let val_dy = &d.l.values[p];
            let shift = (val_dy.e - e) as usize;
            let val_int = &val_dy.m << shift;
            lc[c].push((row, val_int.clone()));
            let neg_scaled = -(&val_int << s);
            lr[row].push((c, neg_scaled));
        }
    }

    // K lower triangle entries bucketed by min(a, b)
    let mut k_low: Vec<Vec<(usize, BigInt)>> = vec![Vec::new(); n];
    for col in 0..n {
        for p in d.k.col_ptr[col]..d.k.col_ptr[col + 1] {
            let row = d.k.row_idx[p];
            let a = col;
            let b = row;
            let c_min = a.min(b);
            let c_max = a.max(b);
            let val_dy = &d.k.values[p];
            let shift = (val_dy.e - e2) as usize;
            let val_int = &val_dy.m << shift;
            k_low[c_min].push((c_max, val_int));
        }
    }

    // Accumulate Gershgorin sums: g[i] * 2^e2 = R_ii - sum_{j!=i} |R_ij|
    let mut g: Vec<BigInt> = vec![BigInt::zero(); n];
    let mut res_dense = vec![BigInt::zero(); n];

    for j in 0..n {
        // Build dense lower-triangle column j of R: rows i >= j
        for &(i, ref v) in &k_low[j] {
            res_dense[i] += v;
        }
        for &(c, ref r_val) in &lr[j] {
            for &(row, ref q_val) in &lc[c] {
                if row >= j {
                    res_dense[row] += q_val * r_val;
                }
            }
        }

        // Add Gershgorin contributions
        g[j] += &res_dense[j];
        for i in (j + 1)..n {
            let abs_val = res_dense[i].abs();
            if !abs_val.is_zero() {
                g[i] -= &abs_val;
                g[j] -= &abs_val;
            }
            res_dense[i] = BigInt::zero();
        }
        res_dense[j] = BigInt::zero();
    }

    // Row check: mu <= g[i] * 2^e2
    for i in 0..n {
        let row_dy = Dy::new(g[i].clone(), e2);
        if !d.mu.le(&row_dy) {
            return (
                1,
                Verdict::Reject(format!(
                    "Gershgorin test on R = K − L Lᵀ fails for row {} (μ too large)",
                    i
                )),
            );
        }
    }

    // 2. Residual check: K * u
    let mut ku: Vec<Dy> = vec![Dy::zero(); n];
    for col in 0..n {
        for p in d.k.col_ptr[col]..d.k.col_ptr[col + 1] {
            let row = d.k.row_idx[p];
            let val = &d.k.values[p];
            let prod1 = val.mul(&d.u[col]);
            ku[row] = ku[row].add(&prod1);
            if row != col {
                let prod2 = val.mul(&d.u[row]);
                ku[col] = ku[col].add(&prod2);
            }
        }
    }

    // rho^2 = sum_i max(|fLo_i - (Ku)_i|, |fHi_i - (Ku)_i|)^2
    let mut rho2 = Dy::zero();
    for i in 0..n {
        let lo_diff = d.f_lo[i].sub(&ku[i]).abs();
        let hi_diff = d.f_hi[i].sub(&ku[i]).abs();
        let m = lo_diff.max(&hi_diff);
        let m2 = m.mul(&m);
        rho2 = rho2.add(&m2);
    }

    // Check rho^2 <= (bound * mu)^2
    let b_mu = d.bound.mul(&d.mu);
    let target = b_mu.mul(&b_mu);
    if !rho2.le(&target) {
        return (
            1,
            Verdict::Reject("residual: ρ² > (bound·μ)² (bound too small or ũ wrong)".to_string()),
        );
    }

    (
        0,
        Verdict::Accept(format!(
            "linear static, n = {}, nnz(K) = {}, nnz(L) = {}",
            n, d.k.values.len(), d.l.values.len()
        )),
    )
}

// -----------------------------------------------------------------------------
// Plastic Collapse Checker
// -----------------------------------------------------------------------------

struct Lin {
    terms: Vec<(usize, BigRational)>,
    lam: BigRational,
}

impl Lin {
    fn eval(&self, x: &[BigRational], lam: &BigRational) -> BigRational {
        let mut sum = BigRational::zero();
        for &(i, ref c) in &self.terms {
            if i < x.len() {
                sum += c * &x[i];
            }
        }
        sum + &self.lam * lam
    }
}

struct Cubic {
    c0: Lin,
    c1: Lin,
    c2: Lin,
    c3: Lin,
}

struct Comp {
    n: BigRational,
    mp_lo: BigRational,
    mp_hi: BigRational,
}

struct Piece {
    a: BigRational,
    b: BigRational,
    polys: Vec<Cubic>,
    cuts: Vec<BigRational>,
}

struct Member {
    comps: Vec<Comp>,
    pieces: Vec<Piece>,
}

struct Row {
    terms: Vec<(usize, BigRational)>,
    p: BigRational,
}

struct Hinge {
    member: usize,
    piece: usize,
    t: BigRational,
    theta: Vec<BigRational>,
    diss: BigRational,
}

struct PlasticBundleData {
    nv: usize,
    rows: Vec<Row>,
    members: Vec<Member>,
    lam_lo: BigRational,
    x: Vec<BigRational>,
    mult: Vec<BigRational>,
    hinges: Vec<Hinge>,
    lam_up: BigRational,
}

fn parse_lin(v: &Value) -> Result<Lin, String> {
    let terms_arr = v.get("terms").and_then(|v| v.as_array()).ok_or("missing lin terms")?;
    let mut terms = Vec::new();
    for item in terms_arr {
        let pair = item.as_array().ok_or("lin term not [idx, rat]")?;
        if pair.len() != 2 {
            return Err("lin term pair length != 2".to_string());
        }
        let idx = pair[0].as_u64().ok_or("lin index not u64")? as usize;
        let r = parse_rat(&pair[1])?;
        terms.push((idx, r));
    }
    let lam = parse_rat(v.get("lam").ok_or("missing lin lam")?)?;
    Ok(Lin { terms, lam })
}

fn parse_cubic(v: &Value) -> Result<Cubic, String> {
    let arr = v.as_array().ok_or("cubic not array")?;
    if arr.len() != 4 {
        return Err("cubic must have 4 coefficients".to_string());
    }
    Ok(Cubic {
        c0: parse_lin(&arr[0])?,
        c1: parse_lin(&arr[1])?,
        c2: parse_lin(&arr[2])?,
        c3: parse_lin(&arr[3])?,
    })
}

fn parse_plastic_bundle(j: &Value) -> Result<PlasticBundleData, String> {
    let nv = j.get("nv").and_then(|v| v.as_u64()).ok_or("missing nv")? as usize;

    let rows_arr = j.get("rows").and_then(|v| v.as_array()).ok_or("missing rows")?;
    let mut rows = Vec::new();
    for r_val in rows_arr {
        let terms_arr = r_val.get("terms").and_then(|v| v.as_array()).ok_or("row missing terms")?;
        let mut terms = Vec::new();
        for t in terms_arr {
            let pair = t.as_array().ok_or("row term not pair")?;
            let idx = pair[0].as_u64().ok_or("bad row term idx")? as usize;
            let val = parse_rat(&pair[1])?;
            terms.push((idx, val));
        }
        let p = parse_rat(r_val.get("p").ok_or("row missing p")?)?;
        rows.push(Row { terms, p });
    }

    let members_arr = j.get("members").and_then(|v| v.as_array()).ok_or("missing members")?;
    let mut members = Vec::new();
    for m_val in members_arr {
        let comps_arr = m_val.get("comps").and_then(|v| v.as_array()).ok_or("missing comps")?;
        let mut comps = Vec::new();
        for c in comps_arr {
            comps.push(Comp {
                n: parse_rat(c.get("n").ok_or("missing comp n")?)?,
                mp_lo: parse_rat(c.get("mpLo").ok_or("missing comp mpLo")?)?,
                mp_hi: parse_rat(c.get("mpHi").ok_or("missing comp mpHi")?)?,
            });
        }
        let pieces_arr = m_val.get("pieces").and_then(|v| v.as_array()).ok_or("missing pieces")?;
        let mut pieces = Vec::new();
        for p in pieces_arr {
            let a = parse_rat(p.get("a").ok_or("missing piece a")?)?;
            let b = parse_rat(p.get("b").ok_or("missing piece b")?)?;
            let polys_arr = p.get("polys").and_then(|v| v.as_array()).ok_or("missing piece polys")?;
            let polys: Vec<Cubic> = polys_arr.iter().map(parse_cubic).collect::<Result<_, _>>()?;
            let cuts_arr = p.get("cuts").and_then(|v| v.as_array()).ok_or("missing piece cuts")?;
            let cuts: Vec<BigRational> = cuts_arr.iter().map(parse_rat).collect::<Result<_, _>>()?;
            pieces.push(Piece { a, b, polys, cuts });
        }
        members.push(Member { comps, pieces });
    }

    let st = j.get("static").ok_or("missing static object")?;
    let lam_lo = parse_rat(st.get("lambda").ok_or("missing static lambda")?)?;
    let x_arr = st.get("x").and_then(|v| v.as_array()).ok_or("missing static x")?;
    let x: Vec<BigRational> = x_arr.iter().map(parse_rat).collect::<Result<_, _>>()?;

    let kin = j.get("kinematic").ok_or("missing kinematic object")?;
    let lam_up = parse_rat(kin.get("lambda").ok_or("missing kinematic lambda")?)?;
    let mult_arr = kin.get("multipliers").and_then(|v| v.as_array()).ok_or("missing kinematic multipliers")?;
    let mult: Vec<BigRational> = mult_arr.iter().map(parse_rat).collect::<Result<_, _>>()?;

    let hinges_arr = kin.get("hinges").and_then(|v| v.as_array()).ok_or("missing kinematic hinges")?;
    let mut hinges = Vec::new();
    for h in hinges_arr {
        let member = h.get("member").and_then(|v| v.as_u64()).ok_or("missing hinge member")? as usize;
        let piece = h.get("piece").and_then(|v| v.as_u64()).ok_or("missing hinge piece")? as usize;
        let t = parse_rat(h.get("t").ok_or("missing hinge t")?)?;
        let theta_arr = h.get("theta").and_then(|v| v.as_array()).ok_or("missing hinge theta")?;
        let theta: Vec<BigRational> = theta_arr.iter().map(parse_rat).collect::<Result<_, _>>()?;
        let diss = parse_rat(h.get("diss").ok_or("missing hinge diss")?)?;
        hinges.push(Hinge {
            member,
            piece,
            t,
            theta,
            diss,
        });
    }

    Ok(PlasticBundleData {
        nv,
        rows,
        members,
        lam_lo,
        x,
        mult,
        hinges,
        lam_up,
    })
}

fn bernstein_bound(q0: &BigRational, q1: &BigRational, q2: &BigRational, q3: &BigRational, s: &BigRational, sp: &BigRational) -> BigRational {
    let h = sp - s;
    let p0 = q0 + q1 * s + q2 * s * s + q3 * s * s * s;
    let d1 = (q1 + BigRational::from_integer(BigInt::from(2)) * q2 * s + BigRational::from_integer(BigInt::from(3)) * q3 * s * s) * &h;
    let d2 = (q2 + BigRational::from_integer(BigInt::from(3)) * q3 * s) * &h * &h;
    let d3 = q3 * &h * &h * &h;

    let b0 = p0.abs();
    let b1 = (&p0 + &d1 / BigRational::from_integer(BigInt::from(3))).abs();
    let b2 = (&p0 + &d1 * BigRational::from_integer(BigInt::from(2)) / BigRational::from_integer(BigInt::from(3)) + &d2 / BigRational::from_integer(BigInt::from(3))).abs();
    let b3 = (&p0 + &d1 + &d2 + &d3).abs();

    b0.max(b1).max(b2).max(b3)
}

fn check_plastic_bundle(d: &PlasticBundleData) -> (u32, Verdict) {
    // Member checks
    for m in &d.members {
        for c in &m.comps {
            if c.n <= BigRational::zero() || c.mp_lo <= BigRational::zero() || c.mp_lo > c.mp_hi {
                return (
                    1,
                    Verdict::Reject("yield data (n > 0, 0 < mpLo <= mpHi)".to_string()),
                );
            }
        }
    }

    // Static certificate checks
    // 1. Equilibrium rows: sum a_i x_i == lam_lo * p
    for r in &d.rows {
        let mut lhs = BigRational::zero();
        for &(idx, ref coef) in &r.terms {
            if idx < d.x.len() {
                lhs += coef * &d.x[idx];
            }
        }
        let rhs = &d.lam_lo * &r.p;
        if lhs != rhs {
            return (
                1,
                Verdict::Reject(
                    "static certificate: equilibrium or yield test fails (λ_lo too large)".to_string(),
                ),
            );
        }
    }

    // 2. Yield condition on each piece
    let one = BigRational::one();
    for m in &d.members {
        for p in &m.pieces {
            let mut segs = Vec::new();
            let mut cur = &p.a;
            for cut in &p.cuts {
                if cur > cut {
                    return (1, Verdict::Reject("static: cuts not sorted".to_string()));
                }
                segs.push((cur, cut));
                cur = cut;
            }
            if cur > &p.b {
                return (1, Verdict::Reject("static: cuts exceed piece bound b".to_string()));
            }
            segs.push((cur, &p.b));

            for (s, sp) in segs {
                let mut sum_ratio = BigRational::zero();
                for (c_idx, comp) in m.comps.iter().enumerate() {
                    let cubic = &p.polys[c_idx];
                    let q0 = cubic.c0.eval(&d.x, &d.lam_lo);
                    let q1 = cubic.c1.eval(&d.x, &d.lam_lo);
                    let q2 = cubic.c2.eval(&d.x, &d.lam_lo);
                    let q3 = cubic.c3.eval(&d.x, &d.lam_lo);
                    let b_val = bernstein_bound(&q0, &q1, &q2, &q3, s, sp);
                    let b2 = &b_val * &b_val;
                    let denom = &comp.n * &comp.mp_lo * &comp.mp_lo;
                    sum_ratio += b2 / denom;
                }
                if sum_ratio > one {
                    return (
                        1,
                        Verdict::Reject(
                            "static certificate: equilibrium or yield test fails (λ_lo too large)".to_string(),
                        ),
                    );
                }
            }
        }
    }

    // Kinematic certificate checks
    let mut dense_x_coeffs = vec![BigRational::zero(); d.nv];

    // Multipliers on equilibrium rows
    for (r_idx, r) in d.rows.iter().enumerate() {
        let u_r = if r_idx < d.mult.len() { &d.mult[r_idx] } else { &BigRational::zero() };
        for &(idx, ref coef) in &r.terms {
            if idx < d.nv {
                dense_x_coeffs[idx] += u_r * coef;
            }
        }
    }

    // Hinges on x-coefficients and lambda-coefficient
    let mut hinge_work_lam = BigRational::zero();
    let mut total_diss = BigRational::zero();

    for h in &d.hinges {
        if h.member >= d.members.len() {
            return (1, Verdict::Reject("kinematic: hinge member out of range".to_string()));
        }
        let m = &d.members[h.member];
        if h.piece >= m.pieces.len() {
            return (1, Verdict::Reject("kinematic: hinge piece out of range".to_string()));
        }
        let piece = &m.pieces[h.piece];
        if &h.t < &piece.a || &h.t > &piece.b || h.diss < BigRational::zero() {
            return (1, Verdict::Reject("kinematic: hinge bounds violated".to_string()));
        }

        let mut sum_theta2_mp2 = BigRational::zero();
        let t = &h.t;
        let t2 = t * t;
        let t3 = &t2 * t;

        for (c_idx, comp) in m.comps.iter().enumerate() {
            let theta = if c_idx < h.theta.len() { &h.theta[c_idx] } else { &BigRational::zero() };
            sum_theta2_mp2 += theta * theta * &comp.n * &comp.mp_hi * &comp.mp_hi;

            let cubic = &piece.polys[c_idx];
            let terms_tuples = [
                (&cubic.c0, BigRational::one()),
                (&cubic.c1, t.clone()),
                (&cubic.c2, t2.clone()),
                (&cubic.c3, t3.clone()),
            ];

            for (lin, ref t_power) in terms_tuples {
                let factor = theta * t_power;
                for &(idx, ref c_val) in &lin.terms {
                    if idx < d.nv {
                        dense_x_coeffs[idx] -= &factor * c_val;
                    }
                }
                hinge_work_lam += &factor * &lin.lam;
            }
        }

        if sum_theta2_mp2 > &h.diss * &h.diss {
            return (
                1,
                Verdict::Reject("kinematic: dissipation bound violated for hinge".to_string()),
            );
        }
        total_diss += &h.diss;
    }

    // Check virtual-work equilibrium identity: all x coefficients must be EXACTLY zero
    for (idx, coeff) in dense_x_coeffs.iter().enumerate() {
        if !coeff.is_zero() {
            return (
                1,
                Verdict::Reject(format!(
                    "kinematic certificate: virtual-work identity fails on x_{}",
                    idx
                )),
            );
        }
    }

    // External work: W = sum u_r p_r + hinge_work_lam
    let mut work = hinge_work_lam;
    for (r_idx, r) in d.rows.iter().enumerate() {
        let u_r = if r_idx < d.mult.len() { &d.mult[r_idx] } else { &BigRational::zero() };
        work += u_r * &r.p;
    }

    if work <= BigRational::zero() {
        return (
            1,
            Verdict::Reject("kinematic certificate: external work <= 0".to_string()),
        );
    }

    if total_diss > &d.lam_up * &work {
        return (
            1,
            Verdict::Reject(
                "kinematic certificate: virtual-work identity, hinge data or Σ D ≤ λ_up W fails (λ_up too small)".to_string(),
            ),
        );
    }

    (
        0,
        Verdict::Accept(format!(
            "plastic collapse, {} equilibrium rows, {} members, {} hinges",
            d.rows.len(), d.members.len(), d.hinges.len()
        )),
    )
}

// -----------------------------------------------------------------------------
// Load Combinations Checker
// -----------------------------------------------------------------------------

struct CaseData {
    f_lo: Vec<Dy>,
    f_hi: Vec<Dy>,
    u: Vec<Dy>,
    bound: Dy,
}

struct ComboData {
    factors: Vec<(usize, Dy)>,
    u: Vec<Dy>,
    bound: Dy,
}

struct ComboBundleData {
    base: LinearCoreData,
    cases: Vec<CaseData>,
    combos: Vec<ComboData>,
    env_max_abs: Vec<Dy>,
    env_bound: Dy,
    nq: usize,
    case_vals: Vec<Vec<Dy>>,
    case_bounds: Vec<Vec<Dy>>,
    combo_vals: Vec<Vec<Dy>>,
    combo_bounds: Vec<Vec<Dy>>,
    env_q_bounds: Vec<Dy>,
}

fn parse_matrix_dy(v: &Value) -> Result<Vec<Vec<Dy>>, String> {
    let arr = v.as_array().ok_or("expected matrix array")?;
    arr.iter().map(parse_dy_array).collect()
}

fn parse_combo_bundle(j: &Value) -> Result<ComboBundleData, String> {
    let base = parse_linear_core(j)?;
    let cases_arr = j.get("cases").and_then(|v| v.as_array()).ok_or("missing cases")?;
    let mut cases = Vec::new();
    for c in cases_arr {
        cases.push(CaseData {
            f_lo: parse_dy_array(c.get("fLo").ok_or("case missing fLo")?)?,
            f_hi: parse_dy_array(c.get("fHi").ok_or("case missing fHi")?)?,
            u: parse_dy_array(c.get("u").ok_or("case missing u")?)?,
            bound: parse_dyadic(c.get("bound").ok_or("case missing bound")?)?,
        });
    }

    let combos_arr = j.get("combos").and_then(|v| v.as_array()).ok_or("missing combos")?;
    let mut combos = Vec::new();
    for c in combos_arr {
        let f_arr = c.get("factors").and_then(|v| v.as_array()).ok_or("combo missing factors")?;
        let mut factors = Vec::new();
        for item in f_arr {
            let pair = item.as_array().ok_or("factor not pair")?;
            let c_idx = pair[0].as_u64().ok_or("bad factor case index")? as usize;
            let factor = parse_dyadic(&pair[1])?;
            factors.push((c_idx, factor));
        }
        combos.push(ComboData {
            factors,
            u: parse_dy_array(c.get("u").ok_or("combo missing u")?)?,
            bound: parse_dyadic(c.get("bound").ok_or("combo missing bound")?)?,
        });
    }

    let env = j.get("envelope").ok_or("missing envelope")?;
    let env_max_abs = parse_dy_array(env.get("maxAbs").ok_or("missing maxAbs")?)?;
    let env_bound = parse_dyadic(env.get("bound").ok_or("missing env bound")?)?;

    let qn = j.get("quantities").ok_or("missing quantities")?;
    let env_q_bounds = parse_dy_array(qn.get("envelopeBounds").ok_or("missing envQBounds")?)?;
    let nq = env_q_bounds.len();

    let case_vals = parse_matrix_dy(qn.get("caseValues").ok_or("missing caseValues")?)?;
    let case_bounds = parse_matrix_dy(qn.get("caseBounds").ok_or("missing caseBounds")?)?;
    let combo_vals = parse_matrix_dy(qn.get("comboValues").ok_or("missing comboValues")?)?;
    let combo_bounds = parse_matrix_dy(qn.get("comboBounds").ok_or("missing comboBounds")?)?;

    Ok(ComboBundleData {
        base,
        cases,
        combos,
        env_max_abs,
        env_bound,
        nq,
        case_vals,
        case_bounds,
        combo_vals,
        combo_bounds,
        env_q_bounds,
    })
}

fn check_combo_bundle(d: &ComboBundleData) -> (u32, Verdict) {
    let n = d.base.n;

    // Check each case as a linear static bundle on the base K, L, mu
    for (i, c) in d.cases.iter().enumerate() {
        let case_bundle = LinearBundleData {
            n,
            k: CscMatrixDy {
                col_ptr: d.base.k.col_ptr.clone(),
                row_idx: d.base.k.row_idx.clone(),
                values: d.base.k.values.clone(),
            },
            f_lo: c.f_lo.clone(),
            f_hi: c.f_hi.clone(),
            u: c.u.clone(),
            perm: d.base.perm.clone(),
            l: CscMatrixDy {
                col_ptr: d.base.l.col_ptr.clone(),
                row_idx: d.base.l.row_idx.clone(),
                values: d.base.l.values.clone(),
            },
            mu: d.base.mu.clone(),
            bound: c.bound.clone(),
        };
        let (status, verdict) = check_linear_bundle(&case_bundle);
        if status != 0 {
            return (
                status,
                Verdict::Reject(format!("case {} failed linear verification: {:?}", i, verdict)),
            );
        }
    }

    // Check combinations
    for (j, co) in d.combos.iter().enumerate() {
        let mut lin_bound = Dy::zero();
        for &(c_idx, ref c_factor) in &co.factors {
            if c_idx >= d.cases.len() {
                return (1, Verdict::Reject("combo case index out of range".to_string()));
            }
            let term = c_factor.abs().mul(&d.cases[c_idx].bound);
            lin_bound = lin_bound.add(&term);
        }

        if !lin_bound.le(&co.bound) {
            return (
                1,
                Verdict::Reject(format!(
                    "a combination bound is below Σ|cᵢ|bᵢ for combo {}",
                    j
                )),
            );
        }

        let mut dev2 = Dy::zero();
        for k in 0..n {
            let mut lin_u_k = Dy::zero();
            for &(c_idx, ref c_factor) in &co.factors {
                let u_ik = if k < d.cases[c_idx].u.len() {
                    &d.cases[c_idx].u[k]
                } else {
                    &Dy::zero()
                };
                lin_u_k = lin_u_k.add(&c_factor.mul(u_ik));
            }
            let diff = lin_u_k.sub(&co.u[k]);
            dev2 = dev2.add(&diff.mul(&diff));
        }

        let margin = co.bound.sub(&lin_bound);
        let margin2 = margin.mul(&margin);
        if !dev2.le(&margin2) {
            return (
                1,
                Verdict::Reject(format!(
                    "combination bound is below Σ|cᵢ|bᵢ + ‖Σcᵢũᵢ − ũ‖ for combo {}",
                    j
                )),
            );
        }
    }

    // Envelope checks
    if d.combos.is_empty() {
        return (1, Verdict::Reject("no combinations in bundle".to_string()));
    }

    for co in &d.combos {
        if !co.bound.le(&d.env_bound) {
            return (
                1,
                Verdict::Reject("envelope bound is below a combination bound".to_string()),
            );
        }
    }

    for k in 0..n {
        let env_val = &d.env_max_abs[k];
        let mut found_max = false;
        for co in &d.combos {
            let u_abs = co.u[k].abs();
            if !u_abs.le(env_val) {
                return (
                    1,
                    Verdict::Reject("envelope: value is not max |u_j[k]|".to_string()),
                );
            }
            if env_val.le(&u_abs) {
                found_max = true;
            }
        }
        if !found_max {
            return (
                1,
                Verdict::Reject("envelope: maxAbs[k] not achieved by any combo".to_string()),
            );
        }
    }

    // Quantities checks
    for j in 0..d.combos.len() {
        let co = &d.combos[j];
        for q in 0..d.nq {
            let mut lin = Dy::zero();
            let mut lin_val = Dy::zero();
            for &(c_idx, ref factor) in &co.factors {
                let b_term = factor.abs().mul(&d.case_bounds[c_idx][q]);
                lin = lin.add(&b_term);
                let v_term = factor.mul(&d.case_vals[c_idx][q]);
                lin_val = lin_val.add(&v_term);
            }
            let dev = lin_val.sub(&d.combo_vals[j][q]).abs();
            let sum_bound = lin.add(&dev);
            let combo_b = &d.combo_bounds[j][q];
            if !sum_bound.le(combo_b) || !combo_b.le(&d.env_q_bounds[q]) {
                return (
                    1,
                    Verdict::Reject(format!(
                        "an end-force combination or envelope bound does not follow from case bounds (combo {}, q {})",
                        j, q
                    )),
                );
            }
        }
    }

    (
        0,
        Verdict::Accept(format!(
            "load combinations, n = {}, {} cases, {} combinations",
            n, d.cases.len(), d.combos.len()
        )),
    )
}

// -----------------------------------------------------------------------------
// Top-Level Bundle Entry Points
// -----------------------------------------------------------------------------

pub fn check_bundle_str(content: &str) -> (u32, Verdict) {
    let json: Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(e) => return (2, Verdict::BadFormat(format!("JSON parse error: {}", e))),
    };

    let fmt = match json.get("format").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return (2, Verdict::BadFormat("missing format field".to_string())),
    };

    match fmt {
        "stabileo.linear-certificate.v1" | "stabileo.linear-certificate.v2" => {
            if fmt == "stabileo.linear-certificate.v2" && json.get("provenance").is_none() {
                return (
                    2,
                    Verdict::BadFormat("format v2 requires provenance".to_string()),
                );
            }
            match parse_linear_bundle(&json) {
                Ok(data) => check_linear_bundle(&data),
                Err(e) => (2, Verdict::BadFormat(format!("linear bundle parse error: {}", e))),
            }
        }
        "stabileo.plastic-certificate.v1" => match parse_plastic_bundle(&json) {
            Ok(data) => check_plastic_bundle(&data),
            Err(e) => (2, Verdict::BadFormat(format!("plastic bundle parse error: {}", e))),
        },
        "stabileo.combination-certificate.v1" => match parse_combo_bundle(&json) {
            Ok(data) => check_combo_bundle(&data),
            Err(e) => (2, Verdict::BadFormat(format!("combination bundle parse error: {}", e))),
        },
        _ => (
            2,
            Verdict::BadFormat(format!("unsupported bundle format: '{}'", fmt)),
        ),
    }
}

pub fn check_bundle_file(path: &Path) -> (u32, Verdict) {
    let content = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return (2, Verdict::BadFormat(format!("cannot read file {:?}: {}", path, e))),
    };
    check_bundle_str(&content)
}

