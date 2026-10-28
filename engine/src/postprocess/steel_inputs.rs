//! Rust implementation of steel design check inputs.
//!
//! Computes engineering parameters previously evaluated in TypeScript:
//! - Cb moment gradient factor (CIRSOC 301 §F.1.1)
//! - Section constants J, Cw, Zx, Zy (AISC DG 9, CIRSOC 301 §F.2)
//! - Unbraced lengths L and Lb for member chains and flange bracing
//! - Torsional buckling feasibility and Fez (CIRSOC 301 §E.4)
//! - Flange local buckling slenderness and classification (CIRSOC 301 §F.6.2, Tabla B.4.1b)
//! - Tension member effective net area Ae = An · U (CIRSOC 301 §D.3, §B.4.2.2)
//! - Steel material family classification

use serde::{Deserialize, Serialize};

// ==================== 1. Cb Moment Gradient ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationMoment {
    pub t: f64,
    pub m: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CbBasis {
    Computed,
    UnityRequiredCantilever,
    UnityNoDiagram,
    UnityOutOfScope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MomentGradientResult {
    pub cb: f64,
    pub basis: CbBasis,
    pub reason_key: String,
    pub m_max: f64,
    pub m_a: f64,
    pub m_b: f64,
    pub m_c: f64,
    pub curvature: String,
}

fn interpolate_moment(stations: &[StationMoment], t: f64) -> f64 {
    if stations.is_empty() {
        return 0.0;
    }
    if t <= stations[0].t {
        return stations[0].m;
    }
    let last = &stations[stations.len() - 1];
    if t >= last.t {
        return last.m;
    }
    for i in 1..stations.len() {
        let a = &stations[i - 1];
        let b = &stations[i];
        if t <= b.t {
            let span = b.t - a.t;
            if span <= 0.0 {
                return b.m;
            }
            return a.m + ((t - a.t) / span) * (b.m - a.m);
        }
    }
    last.m
}

pub fn compute_moment_gradient_cb(
    stations: &[StationMoment],
    t_start: f64,
    t_end: f64,
    shape: Option<&str>,
    is_cantilever_free_end: bool,
) -> MomentGradientResult {
    if is_cantilever_free_end {
        return MomentGradientResult {
            cb: 1.0,
            basis: CbBasis::UnityRequiredCantilever,
            reason_key: "steel.cb.reason.cantilever".into(),
            m_max: 0.0,
            m_a: 0.0,
            m_b: 0.0,
            m_c: 0.0,
            curvature: "none".into(),
        };
    }

    let mut sorted = stations.to_vec();
    sorted.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal));

    let in_segment: Vec<&StationMoment> = sorted
        .iter()
        .filter(|s| s.t >= t_start - 1e-9 && s.t <= t_end + 1e-9)
        .collect();

    let signs: Vec<i32> = in_segment
        .iter()
        .map(|s| if s.m > 1e-9 { 1 } else if s.m < -1e-9 { -1 } else { 0 })
        .filter(|&x| x != 0)
        .collect();

    let curvature = if signs.is_empty() {
        "none".to_string()
    } else if signs.iter().any(|&x| x != signs[0]) {
        "double".to_string()
    } else {
        "single".to_string()
    };

    let shape_upper = shape.map(|s| s.to_ascii_uppercase()).unwrap_or_default();
    let is_double_sym = matches!(shape_upper.as_str(), "I" | "H" | "RHS" | "CHS" | "RECT" | "BOX");
    let is_single_sym = matches!(shape_upper.as_str(), "U" | "C" | "T");

    let in_scope = if is_double_sym {
        true
    } else if is_single_sym {
        curvature != "double"
    } else {
        false
    };

    if !in_scope {
        return MomentGradientResult {
            cb: 1.0,
            basis: CbBasis::UnityOutOfScope,
            reason_key: if is_single_sym {
                "steel.cb.reason.singlySymmetricDoubleCurvature".into()
            } else {
                "steel.cb.reason.symmetryOutOfScope".into()
            },
            m_max: 0.0,
            m_a: 0.0,
            m_b: 0.0,
            m_c: 0.0,
            curvature,
        };
    }

    if in_segment.len() < 3 {
        return MomentGradientResult {
            cb: 1.0,
            basis: CbBasis::UnityNoDiagram,
            reason_key: "steel.cb.reason.noDiagram".into(),
            m_max: 0.0,
            m_a: 0.0,
            m_b: 0.0,
            m_c: 0.0,
            curvature,
        };
    }

    let span = t_end - t_start;
    let m_a = interpolate_moment(&sorted, t_start + 0.25 * span).abs();
    let m_b = interpolate_moment(&sorted, t_start + 0.50 * span).abs();
    let m_c = interpolate_moment(&sorted, t_start + 0.75 * span).abs();

    let mut m_max = m_a.max(m_b).max(m_c);
    for s in &in_segment {
        let abs_m = s.m.abs();
        if abs_m > m_max {
            m_max = abs_m;
        }
    }

    let denom = 2.5 * m_max + 3.0 * m_a + 4.0 * m_b + 3.0 * m_c;
    if denom <= 0.0 {
        return MomentGradientResult {
            cb: 1.0,
            basis: CbBasis::UnityNoDiagram,
            reason_key: "steel.cb.reason.zeroMoment".into(),
            m_max,
            m_a,
            m_b,
            m_c,
            curvature,
        };
    }

    let cb = (12.5 * m_max) / denom;
    MomentGradientResult {
        cb,
        basis: CbBasis::Computed,
        reason_key: "steel.cb.reason.computed".into(),
        m_max,
        m_a,
        m_b,
        m_c,
        curvature,
    }
}

// ==================== 2. Section Constants ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteelSectionConstantsResult {
    pub j: f64,
    pub j_basis: String,
    pub cw: Option<f64>,
    pub zx: Option<f64>,
    pub zy: Option<f64>,
    pub c: Option<f64>,
    pub sy: Option<f64>,
}

pub fn compute_steel_section_constants(
    shape: Option<&str>,
    b: Option<f64>,
    h: Option<f64>,
    tw: Option<f64>,
    tf: Option<f64>,
    _iz: Option<f64>,
    iy: Option<f64>,
    j_declared: Option<f64>,
) -> SteelSectionConstantsResult {
    let s = shape.map(|s| s.to_ascii_uppercase()).unwrap_or_default();
    let is_i_shape = s == "I" || s == "H";
    let is_channel = s == "U" || s == "C";

    let has_plates = b.unwrap_or(0.0) > 0.0
        && h.unwrap_or(0.0) > 0.0
        && tw.unwrap_or(0.0) > 0.0
        && tf.unwrap_or(0.0) > 0.0
        && h.unwrap_or(0.0) > 2.0 * tf.unwrap_or(0.0);

    let (j, j_basis) = if let Some(jd) = j_declared {
        if jd > 0.0 {
            (jd, "declared".to_string())
        } else if is_i_shape && has_plates {
            let b_v = b.unwrap();
            let h_v = h.unwrap();
            let tw_v = tw.unwrap();
            let tf_v = tf.unwrap();
            let j_calc = (2.0 * b_v * tf_v.powi(3) + (h_v - 2.0 * tf_v) * tw_v.powi(3)) / 3.0;
            (j_calc, "thinWalled".to_string())
        } else {
            (0.0, "none".to_string())
        }
    } else if is_i_shape && has_plates {
        let b_v = b.unwrap();
        let h_v = h.unwrap();
        let tw_v = tw.unwrap();
        let tf_v = tf.unwrap();
        let j_calc = (2.0 * b_v * tf_v.powi(3) + (h_v - 2.0 * tf_v) * tw_v.powi(3)) / 3.0;
        (j_calc, "thinWalled".to_string())
    } else {
        (0.0, "none".to_string())
    };

    let mut cw = None;
    let mut c_factor = None;
    let mut sy = None;

    if is_i_shape && has_plates && iy.unwrap_or(0.0) > 0.0 {
        let h0 = h.unwrap() - tf.unwrap();
        cw = Some((iy.unwrap() * h0 * h0) / 4.0);
    } else if is_channel && has_plates && iy.unwrap_or(0.0) > 0.0 {
        let b_v = b.unwrap();
        let h_v = h.unwrap();
        let tw_v = tw.unwrap();
        let tf_v = tf.unwrap();
        let b_prime = b_v - tw_v / 2.0;
        let h0 = h_v - tf_v;
        let num = tf_v * b_prime.powi(3) * h0.powi(2) * (3.0 * b_prime * tf_v + 2.0 * h0 * tw_v);
        let den = 12.0 * (6.0 * b_prime * tf_v + h0 * tw_v);
        if den > 0.0 {
            let cw_val = num / den;
            cw = Some(cw_val);
            if cw_val > 0.0 {
                c_factor = Some((h0 / 2.0) * (iy.unwrap() / cw_val).sqrt());
            }
        }
        let x_bar = (b_v * b_v * tf_v + 0.5 * (h_v - 2.0 * tf_v) * tw_v * tw_v)
            / (2.0 * b_v * tf_v + (h_v - 2.0 * tf_v) * tw_v);
        if b_v > x_bar {
            sy = Some(iy.unwrap() / (b_v - x_bar));
        }
    }

    let mut zx = None;
    let mut zy = None;
    if is_i_shape && has_plates {
        let b_v = b.unwrap();
        let h_v = h.unwrap();
        let tw_v = tw.unwrap();
        let tf_v = tf.unwrap();
        zx = Some(tw_v * (h_v - 2.0 * tf_v).powi(2) / 4.0 + b_v * tf_v * (h_v - tf_v));
        zy = Some(tf_v * b_v.powi(2) / 2.0 + (h_v - 2.0 * tf_v) * tw_v.powi(2) / 4.0);
    }

    SteelSectionConstantsResult {
        j,
        j_basis,
        cw,
        zx,
        zy,
        c: c_factor,
        sy,
    }
}

// ==================== 3. Unbraced Lengths ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberLengthsResult {
    pub l: f64,
    pub lb: f64,
    pub source: String,
    pub free_end: bool,
}

pub fn compute_member_unbraced_lengths(
    element_length: f64,
    chain_elements: &[usize],
    user_declared_lb: Option<f64>,
    bracing_points: Option<&[f64]>,
) -> MemberLengthsResult {
    let chain_count = chain_elements.len().max(1);
    let l = element_length * (chain_count as f64);

    let (lb, source) = if let Some(dec) = user_declared_lb {
        if dec > 0.0 {
            (dec, "declared".to_string())
        } else {
            (l, "chain".to_string())
        }
    } else if let Some(points) = bracing_points {
        let mut pts = vec![0.0, l];
        for &p in points {
            if p > 1e-6 && p < l - 1e-6 {
                pts.push(p);
            }
        }
        pts.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut max_gap = 0.0;
        for i in 1..pts.len() {
            let gap = pts[i] - pts[i - 1];
            if gap > max_gap {
                max_gap = gap;
            }
        }
        (max_gap.max(0.01), "braced_points".to_string())
    } else if chain_count > 1 {
        (l, "chain".to_string())
    } else {
        (element_length, "element".to_string())
    };

    MemberLengthsResult {
        l,
        lb,
        source,
        free_end: false,
    }
}

// ==================== 4. Torsional Buckling E.4 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TorsionalBucklingResult {
    pub applicable: bool,
    pub scope: String,
    pub reason_key: String,
    pub fez: Option<f64>,
    pub gaps: Vec<String>,
}

pub fn evaluate_torsional_buckling_e4(
    shape: Option<&str>,
    j: Option<f64>,
    cw: Option<f64>,
    e: f64,
    g: f64,
    kz: f64,
    lz: f64,
    ag: f64,
    ro_sq: f64,
) -> TorsionalBucklingResult {
    let s = shape.map(|x| x.to_ascii_uppercase()).unwrap_or_default();
    if s == "RHS" || s == "CHS" || s == "BOX" {
        return TorsionalBucklingResult {
            applicable: false,
            scope: "outOfScope".into(),
            reason_key: "steel.e4.reason.closedSection".into(),
            fez: None,
            gaps: Vec::new(),
        };
    }

    let mut gaps = Vec::new();
    if j.unwrap_or(0.0) <= 0.0 {
        gaps.push("torsionalConstant".into());
    }
    if cw.unwrap_or(0.0) <= 0.0 {
        gaps.push("warpingConstant".into());
    }

    if !gaps.is_empty() {
        return TorsionalBucklingResult {
            applicable: false,
            scope: "inScope".into(),
            reason_key: "steel.e4.reason.missingInputs".into(),
            fez: None,
            gaps,
        };
    }

    let j_val = j.unwrap();
    let cw_val = cw.unwrap();
    let klz = (kz * lz).max(1e-6);

    let pi = std::f64::consts::PI;
    let num = (pi * pi * e * cw_val) / (klz * klz) + g * j_val;
    let den = ag * ro_sq;
    let fez = if den > 0.0 { Some(num / den) } else { None };

    TorsionalBucklingResult {
        applicable: true,
        scope: "inScope".into(),
        reason_key: "steel.e4.reason.applicable".into(),
        fez,
        gaps: Vec::new(),
    }
}

// ==================== 5. Flange Local Buckling F.6.2 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlangeLocalBucklingResult {
    pub state: String,
    pub branch: String,
    pub reason_key: String,
    pub flange_slenderness: Option<f64>,
    pub bf: Option<f64>,
    pub fcr_mpa: Option<f64>,
    pub sy: Option<f64>,
}

pub fn evaluate_flange_local_buckling_f62(
    shape: Option<&str>,
    b: f64,
    tf: f64,
    fy: f64,
    e: f64,
    iy: f64,
) -> FlangeLocalBucklingResult {
    let s = shape.map(|x| x.to_ascii_uppercase()).unwrap_or_default();
    let is_i = s == "I" || s == "H";
    let is_channel = s == "U" || s == "C";

    if !is_i && !is_channel {
        return FlangeLocalBucklingResult {
            state: "outOfScope".into(),
            branch: "notApplicable".into(),
            reason_key: "steel.f62.reason.outOfScope".into(),
            flange_slenderness: None,
            bf: None,
            fcr_mpa: None,
            sy: None,
        };
    }

    if b <= 0.0 || tf <= 0.0 {
        return FlangeLocalBucklingResult {
            state: "geometryUnavailable".into(),
            branch: "undetermined".into(),
            reason_key: "steel.f62.reason.missingDimensions".into(),
            flange_slenderness: None,
            bf: None,
            fcr_mpa: None,
            sy: None,
        };
    }

    let bf = if is_i { b / 2.0 } else { b };
    let lambda_f = bf / tf;
    let fcr = 138000.0 / (lambda_f * lambda_f);

    let sy = if iy > 0.0 {
        Some(iy / (b / 2.0))
    } else {
        None
    };

    if fy <= 0.0 {
        return FlangeLocalBucklingResult {
            state: "classificationUnavailable".into(),
            branch: "undetermined".into(),
            reason_key: "steel.f62.reason.missingFy".into(),
            flange_slenderness: Some(lambda_f),
            bf: Some(bf),
            fcr_mpa: Some(fcr),
            sy,
        };
    }

    let lambda_pf = 0.38 * (e / fy).sqrt();
    let lambda_rf = 1.00 * (e / fy).sqrt();

    let branch = if lambda_f <= lambda_pf {
        "notApplicableCompact"
    } else if lambda_f <= lambda_rf {
        "nonCompact"
    } else {
        "slender"
    };

    FlangeLocalBucklingResult {
        state: "classified".into(),
        branch: branch.into(),
        reason_key: "steel.f62.reason.classified".into(),
        flange_slenderness: Some(lambda_f),
        bf: Some(bf),
        fcr_mpa: Some(fcr),
        sy,
    }
}

// ==================== 6. Tension Net Area D.3 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoleData {
    pub diameter_mm: f64,
    pub count_per_line: usize,
    pub lines: usize,
    pub thickness_m: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaggerData {
    pub s_mm: f64,
    pub g_mm: f64,
    pub thickness_m: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectiveNetAreaResult {
    pub an: f64,
    pub u: f64,
    pub ae: f64,
    pub reason_key: String,
}

pub fn compute_effective_net_area_d3(
    ag: f64,
    holes: Option<&[HoleData]>,
    staggers: Option<&[StaggerData]>,
    x_bar: f64,
    connection_length_l: f64,
) -> EffectiveNetAreaResult {
    if ag <= 0.0 {
        return EffectiveNetAreaResult {
            an: 0.0,
            u: 1.0,
            ae: 0.0,
            reason_key: "steel.d3.reason.zeroGrossArea".into(),
        };
    }

    let mut hole_deduction = 0.0;
    if let Some(h_list) = holes {
        for h in h_list {
            // Nominal hole dimension + 2 mm (CIRSOC 301 §B.4.2.2)
            let d_effective = (h.diameter_mm + 2.0) * 1e-3;
            hole_deduction += (h.lines as f64) * d_effective * h.thickness_m;
        }
    }

    let mut stagger_addition = 0.0;
    if let Some(s_list) = staggers {
        for s in s_list {
            if s.g_mm > 0.0 {
                stagger_addition += (s.s_mm * s.s_mm / (4.0 * s.g_mm)) * 1e-3 * s.thickness_m;
            }
        }
    }

    let an = (ag - hole_deduction + stagger_addition).max(0.0).min(ag);

    // Shear lag factor U = 1 - x_bar / L <= 0.9 (CIRSOC 301 §D.3.2)
    let u = if connection_length_l > 0.0 {
        (1.0 - x_bar / connection_length_l).min(0.90).max(0.20)
    } else {
        1.0
    };

    let ae = an * u;

    EffectiveNetAreaResult {
        an,
        u,
        ae,
        reason_key: "steel.d3.reason.computed".into(),
    }
}

// ==================== 7. Material Family ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialFamilyResult {
    pub family: String,
    pub fy_mpa: f64,
    pub fu_mpa: f64,
}

pub fn resolve_material_family(name: &str, fy: f64) -> MaterialFamilyResult {
    let lower = name.to_ascii_lowercase();
    if lower.contains("f24") || lower.contains("a36") || lower.contains("s235") || (fy >= 220.0 && fy < 255.0) {
        MaterialFamilyResult {
            family: "F-24".into(),
            fy_mpa: 235.0,
            fu_mpa: 370.0,
        }
    } else if lower.contains("f26") || lower.contains("s275") || (fy >= 255.0 && fy < 315.0) {
        MaterialFamilyResult {
            family: "F-26".into(),
            fy_mpa: 275.0,
            fu_mpa: 410.0,
        }
    } else if lower.contains("f36") || lower.contains("a572") || lower.contains("s355") || (fy >= 315.0 && fy <= 400.0) {
        MaterialFamilyResult {
            family: "F-36".into(),
            fy_mpa: 355.0,
            fu_mpa: 490.0,
        }
    } else {
        MaterialFamilyResult {
            family: "customSteel".into(),
            fy_mpa: fy,
            fu_mpa: fy * 1.35,
        }
    }
}
