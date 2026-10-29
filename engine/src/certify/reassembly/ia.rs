//! Interval helpers for the independent re-assembly: input enclosures,
//! constants, 3-vectors, polynomials, and rigorous sin/cos.
//!
//! Everything here builds on `certify::interval::Interval` (outward-rounded
//! f64 intervals) and nothing else.

use crate::certify::interval::Interval;

pub type I = Interval;

/// Enclosure of the real number a model input stands for. Inputs reach the
/// engine as decimal JSON text parsed to the nearest f64 (serde_json and
/// JavaScript both round correctly), so the decimal lies within half an ulp of
/// the parsed value: `[pred(x), succ(x)]` contains it. An exact 0 stays 0 (a
/// decimal that rounds to 0 is below 2.5e-324 in magnitude).
#[inline]
pub fn inp(x: f64) -> I {
    if x == 0.0 {
        I::ZERO
    } else {
        I::new(x.next_down(), x.next_up())
    }
}

/// An f64 constant that is exactly the real number meant (integers, halves…).
#[inline]
pub fn c(x: f64) -> I {
    I::point(x)
}

/// Enclosure of a decimal constant that is not an f64 (e.g. `12e-6`).
#[inline]
pub fn dec(x: f64) -> I {
    inp(x)
}

/// π ∈ [PI, succ(PI)]: `std::f64::consts::PI` = 3.141592653589793115997… < π
/// = 3.14159265358979323846… < succ(PI).
pub fn pi() -> I {
    I::new(std::f64::consts::PI, std::f64::consts::PI.next_up())
}

#[inline]
pub fn mid(x: I) -> f64 {
    0.5 * x.lo + 0.5 * x.hi
}

#[inline]
pub fn scale(s: f64, x: I) -> I {
    c(s) * x
}

/// True if every real in `x` is > t (t exact).
#[inline]
pub fn certainly_gt(x: I, t: f64) -> bool {
    x.is_finite() && x.lo > t
}

/// True if every real in `x` is < t (t exact).
#[inline]
pub fn certainly_lt(x: I, t: f64) -> bool {
    x.is_finite() && x.hi < t
}

/// Decide `x > t` for every real in `x`, or `None` if the interval straddles t
/// (the model's convention is then ambiguous at rounding level).
#[inline]
pub fn decide_gt(x: I, t: f64) -> Option<bool> {
    if !x.is_finite() {
        None
    } else if x.lo > t {
        Some(true)
    } else if x.hi <= t {
        Some(false)
    } else {
        None
    }
}

// ---------------- 3-vectors ----------------

pub type V3 = [I; 3];

pub fn v3(x: [f64; 3]) -> V3 {
    [c(x[0]), c(x[1]), c(x[2])]
}

pub fn dot(a: &V3, b: &V3) -> I {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn cross(a: &V3, b: &V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

pub fn sub3(a: &V3, b: &V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn smul3(s: I, a: &V3) -> V3 {
    [s * a[0], s * a[1], s * a[2]]
}

pub fn neg3(a: &V3) -> V3 {
    [-a[0], -a[1], -a[2]]
}

pub fn add3(a: &V3, b: &V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// Euclidean norm (rigorous enclosure).
pub fn norm3(a: &V3) -> I {
    let s = a[0].square() + a[1].square() + a[2].square();
    I::new(s.lo.max(0.0), s.hi.max(0.0)).sqrt()
}

/// a / |a|. NaN components when |a| may be 0.
pub fn unit3(a: &V3) -> V3 {
    let n = norm3(a);
    [a[0].div(n), a[1].div(n), a[2].div(n)]
}

pub fn finite3(a: &V3) -> bool {
    a.iter().all(|x| x.is_finite())
}

// ---------------- polynomials (ascending coefficients) ----------------

pub type Poly = Vec<I>;

pub fn padd(a: &Poly, b: &Poly) -> Poly {
    let n = a.len().max(b.len());
    (0..n)
        .map(|k| match (a.get(k), b.get(k)) {
            (Some(&x), Some(&y)) => x + y,
            (Some(&x), None) | (None, Some(&x)) => x,
            (None, None) => I::ZERO,
        })
        .collect()
}

pub fn pmul(a: &Poly, b: &Poly) -> Poly {
    if a.is_empty() || b.is_empty() {
        return Vec::new();
    }
    let mut out = vec![I::ZERO; a.len() + b.len() - 1];
    let mut set = vec![false; out.len()];
    for (i, &x) in a.iter().enumerate() {
        for (j, &y) in b.iter().enumerate() {
            let p = x * y;
            if set[i + j] {
                out[i + j] = out[i + j] + p;
            } else {
                out[i + j] = p;
                set[i + j] = true;
            }
        }
    }
    out
}

pub fn pscale(s: I, a: &Poly) -> Poly {
    a.iter().map(|&x| s * x).collect()
}

/// p(c0 + c1·t) as a polynomial in t (Horner over polynomials).
pub fn compose_affine(p: &Poly, c0: I, c1: I) -> Poly {
    let lin = vec![c0, c1];
    let mut acc: Poly = Vec::new();
    for &coef in p.iter().rev() {
        acc = padd(&pmul(&acc, &lin), &vec![coef]);
    }
    acc
}

/// p(x) by Horner.
pub fn peval(p: &Poly, x: I) -> I {
    let mut acc = I::ZERO;
    let mut first = true;
    for &coef in p.iter().rev() {
        acc = if first { coef } else { acc * x + coef };
        first = false;
    }
    acc
}

/// ∫₀¹ p(t) dt = Σ p_k / (k+1).
pub fn pint01(p: &Poly) -> I {
    let mut acc = I::ZERO;
    for (k, &coef) in p.iter().enumerate() {
        let term = if k == 0 { coef } else { coef.div(c((k + 1) as f64)) };
        acc = if k == 0 { term } else { acc + term };
    }
    acc
}

// ---------------- sin / cos ----------------

/// Rigorous enclosures of (sin x, cos x) for every real x in `x`, |x| ≤ 1e6.
/// Range reduction y = x − k·π/2 with π enclosed, then Taylor series of
/// degree 2N+1 / 2N with the Lagrange remainder |y|^(2N+2)/(2N+2)! added.
pub fn sin_cos(x: I) -> Option<(I, I)> {
    if !x.is_finite() || x.lo.abs() > 1e6 || x.hi.abs() > 1e6 {
        return None;
    }
    let half_pi = pi().div(c(2.0));
    let k = (mid(x) / std::f64::consts::FRAC_PI_2).round();
    let y = x - c(k) * half_pi;
    let ymag = y.mag();
    if !(ymag <= 1.0) {
        return None;
    }
    // Taylor series to degree 25 (sin) / 24 (cos); remainder ≤ 1/26! < 2.5e-27.
    let n_terms = 13;
    let y2 = y.square();
    let mut s = I::ZERO; // Σ (−1)^n y^(2n+1)/(2n+1)!
    let mut co = I::ZERO; // Σ (−1)^n y^(2n)/(2n)!
    let mut term_c = c(1.0); // y^(2n)/(2n)!
    let mut term_s = y; // y^(2n+1)/(2n+1)!
    for n in 0..n_terms {
        let sign = if n % 2 == 0 { 1.0 } else { -1.0 };
        co = co + scale(sign, term_c);
        s = s + scale(sign, term_s);
        let a = (2 * n + 1) as f64;
        let b = (2 * n + 2) as f64;
        let b3 = (2 * n + 3) as f64;
        term_c = (term_c * y2).div(c(a * b));
        term_s = (term_s * y2).div(c(b * b3));
    }
    // Remainders: |R_sin| ≤ |y|^(2N+1)/(2N+1)!, |R_cos| ≤ |y|^(2N)/(2N)! with
    // 2N = 26 (next even/odd terms), bounded by the next term magnitudes.
    let rc = term_c.mag();
    let rs = term_s.mag();
    let rc = I::new(-rc.next_up(), rc.next_up());
    let rs = I::new(-rs.next_up(), rs.next_up());
    let s = s + rs;
    let co = co + rc;
    let q = ((k as i64) % 4 + 4) % 4;
    let (sx, cx) = match q {
        0 => (s, co),
        1 => (co, -s),
        2 => (-s, -co),
        _ => (-co, s),
    };
    // |sin|, |cos| ≤ 1
    let clamp = |v: I| I::new(v.lo.max(-1.0), v.hi.min(1.0));
    let (sx, cx) = (clamp(sx), clamp(cx));
    if sx.is_finite() && cx.is_finite() {
        Some((sx, cx))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sin_cos_encloses_libm_and_identities() {
        let mut x = -20.0;
        while x < 20.0 {
            let (s, co) = sin_cos(inp(x)).unwrap();
            assert!(s.lo <= x.sin() + 1e-15 && x.sin() - 1e-15 <= s.hi, "sin {x}: {s:?}");
            assert!(co.lo <= x.cos() + 1e-15 && x.cos() - 1e-15 <= co.hi, "cos {x}: {co:?}");
            assert!(s.hi - s.lo < 1e-13 && co.hi - co.lo < 1e-13, "{x}: {s:?} {co:?}");
            let one = s.square() + co.square();
            assert!(one.lo <= 1.0 && 1.0 <= one.hi);
            x += 0.137;
        }
        // Exact points: sin(π) = 0 must be enclosed for the enclosure of π.
        let (s, co) = sin_cos(pi()).unwrap();
        assert!(s.contains_zero() && co.lo <= -1.0 + 1e-15);
        let (s, _) = sin_cos(I::ZERO).unwrap();
        assert!(s.contains_zero() && s.mag() < 1e-300);
    }

    #[test]
    fn polynomial_helpers() {
        // p(t) = 1 + 2t + 3t² ; p(2 + t) = 17 + 14t + 3t²
        let p = vec![c(1.0), c(2.0), c(3.0)];
        let q = compose_affine(&p, c(2.0), c(1.0));
        let want = [17.0, 14.0, 3.0];
        for (k, w) in want.iter().enumerate() {
            assert!(q[k].lo <= *w && *w <= q[k].hi);
        }
        let i = pint01(&p); // 1 + 1 + 1 = 3
        assert!(i.lo <= 3.0 && 3.0 <= i.hi);
        let v = peval(&p, c(2.0)); // 17
        assert!(v.lo <= 17.0 && 17.0 <= v.hi);
    }

    #[test]
    fn test_certainly_gt_and_lt_boundaries() {
        // certainly_gt: x.is_finite() && x.lo > t
        assert!(!certainly_gt(I::new(f64::NAN, 1.0), 0.0));
        assert!(!certainly_gt(I::new(1.0, 3.0), 1.0));
        assert!(certainly_gt(I::new(1.0000001, 3.0), 1.0));
        assert!(!certainly_gt(I::new(0.5, 3.0), 1.0));

        // certainly_lt: x.is_finite() && x.hi < t
        assert!(!certainly_lt(I::new(f64::NAN, 0.5), 1.0));
        assert!(!certainly_lt(I::new(0.0, 1.0), 1.0));
        assert!(certainly_lt(I::new(0.0, 0.9999999), 1.0));
        assert!(!certainly_lt(I::new(0.0, 2.0), 1.0));

        // decide_gt: !finite => None, lo > t => Some(true), hi <= t => Some(false), straddles => None
        assert_eq!(decide_gt(I::new(f64::NAN, 1.0), 0.0), None);
        assert_eq!(decide_gt(I::new(1.0, 2.0), 1.0), None);
        assert_eq!(decide_gt(I::new(1.0000001, 2.0), 1.0), Some(true));
        assert_eq!(decide_gt(I::new(0.0, 1.0), 1.0), Some(false));
        assert_eq!(decide_gt(I::new(0.0, 0.5), 1.0), Some(false));
    }

    #[test]
    fn test_neg3() {
        let v = [c(1.0), c(2.0), c(3.0)];
        let n = neg3(&v);
        assert_eq!(n[0].lo, -1.0);
        assert_eq!(n[0].hi, -1.0);
        assert_eq!(n[1].lo, -2.0);
        assert_eq!(n[1].hi, -2.0);
        assert_eq!(n[2].lo, -3.0);
        assert_eq!(n[2].hi, -3.0);
    }

    #[test]
    fn test_pmul_edge_cases() {
        let empty: Poly = Vec::new();
        let one: Poly = vec![c(1.0)];
        assert!(pmul(&empty, &one).is_empty());
        assert!(pmul(&one, &empty).is_empty());
        assert!(pmul(&empty, &empty).is_empty());

        let a = vec![c(1.0), c(2.0)];
        let b = vec![c(3.0), c(4.0)];
        let res = pmul(&a, &b);
        assert_eq!(res.len(), 3);
        assert!(res[0].lo <= 3.0 && 3.0 <= res[0].hi);
        assert!(res[1].lo <= 10.0 && 10.0 <= res[1].hi);
        assert!(res[2].lo <= 8.0 && 8.0 <= res[2].hi);
    }

    #[test]
    fn test_sin_cos_boundaries_and_domain() {
        assert!(sin_cos(I::new(-1.000001e6, 0.0)).is_none());
        assert!(sin_cos(I::new(0.0, 1.000001e6)).is_none());
        assert!(sin_cos(I::new(f64::NAN, 0.0)).is_none());
        assert!(sin_cos(c(1e6)).is_some());
        assert!(sin_cos(c(-1e6)).is_some());

        let non_finite = I::new(f64::NEG_INFINITY, f64::INFINITY);
        assert!(sin_cos(non_finite).is_none());
    }
}
