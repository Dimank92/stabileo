//! CIRSOC 104-2005 — snow load on roofs in Rust.
//!
//! Evaluates Cap. 3 (pf = 0.7 Ce Ct I pg, with §3.4 low-slope minimum),
//! Cap. 4 (ps = Cs pf), Cap. 6 (§6.1 unbalanced loads on gable roofs),
//! Cap. 7 (γ = min(0.426 pg + 2.2, 4.70)), and Cap. 10 (rain on snow).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnowTerrain {
    A,
    B,
    C,
    D,
    #[serde(rename = "aboveTreeline")]
    AboveTreeline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RoofExposure {
    Full,
    Partial,
    Sheltered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThermalCondition {
    Normal,
    ColdVentilated,
    Unheated,
    Greenhouse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnowCategory {
    I,
    II,
    III,
    IV,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RoofKind {
    Mono,
    Gable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnowRoof {
    pub kind: RoofKind,
    pub slope_deg: f64,
    pub w: f64,
    pub slippery: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnowInputs {
    pub pg: f64,
    pub terrain: SnowTerrain,
    pub exposure: RoofExposure,
    pub thermal: ThermalCondition,
    pub category: SnowCategory,
    pub roof: SnowRoof,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClauseRef {
    pub regulation: String,
    pub edition: String,
    pub clause: String,
    pub label: String,
}

fn cref(clause: &str, label: &str) -> ClauseRef {
    ClauseRef {
        regulation: "cirsoc-104".into(),
        edition: "2005".into(),
        clause: clause.into(),
        label: label.into(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnbalancedSnow {
    pub leeward: f64,
    pub windward: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnowResult {
    pub ce: f64,
    pub ct: f64,
    pub importance: f64,
    pub pf_computed: f64,
    pub pf_minimum: Option<f64>,
    pub pf: f64,
    pub cs: f64,
    pub ps: f64,
    pub rain_on_snow: f64,
    pub unbalanced: Option<UnbalancedSnow>,
    pub gamma: f64,
    pub refs: Vec<ClauseRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refused: Option<String>,
}

/// Exposure factor Ce (Tabla 2). Returns None for N/A.
pub fn exposure_factor(terrain: SnowTerrain, exposure: RoofExposure) -> Option<f64> {
    match (terrain, exposure) {
        (SnowTerrain::A, RoofExposure::Full) => None,
        (SnowTerrain::A, RoofExposure::Partial) => Some(1.1),
        (SnowTerrain::A, RoofExposure::Sheltered) => Some(1.3),
        (SnowTerrain::B, RoofExposure::Full) => Some(0.9),
        (SnowTerrain::B, RoofExposure::Partial) => Some(1.0),
        (SnowTerrain::B, RoofExposure::Sheltered) => Some(1.2),
        (SnowTerrain::C, RoofExposure::Full) => Some(0.9),
        (SnowTerrain::C, RoofExposure::Partial) => Some(1.0),
        (SnowTerrain::C, RoofExposure::Sheltered) => Some(1.1),
        (SnowTerrain::D, RoofExposure::Full) => Some(0.8),
        (SnowTerrain::D, RoofExposure::Partial) => Some(0.9),
        (SnowTerrain::D, RoofExposure::Sheltered) => Some(1.0),
        (SnowTerrain::AboveTreeline, RoofExposure::Full) => Some(0.7),
        (SnowTerrain::AboveTreeline, RoofExposure::Partial) => Some(0.8),
        (SnowTerrain::AboveTreeline, RoofExposure::Sheltered) => None,
    }
}

/// Thermal factor Ct (Tabla 3).
pub fn thermal_factor(thermal: ThermalCondition) -> f64 {
    match thermal {
        ThermalCondition::Normal => 1.0,
        ThermalCondition::ColdVentilated => 1.1,
        ThermalCondition::Unheated => 1.2,
        ThermalCondition::Greenhouse => 0.85,
    }
}

/// Importance factor I (Tabla 4).
pub fn importance_factor(category: SnowCategory) -> f64 {
    match category {
        SnowCategory::I => 0.8,
        SnowCategory::II => 1.0,
        SnowCategory::III => 1.1,
        SnowCategory::IV => 1.2,
    }
}

/// §3.4: the roofs the minimum values apply to.
pub fn is_low_slope(roof: &SnowRoof) -> bool {
    match roof.kind {
        RoofKind::Mono => roof.slope_deg < 15.0,
        RoofKind::Gable => roof.slope_deg <= 21.0 / roof.w.max(1e-9) + 0.5,
    }
}

/// Figura 2 slope factor Cs.
pub fn slope_factor(slope_deg: f64, ct: f64, slippery: bool) -> f64 {
    let start = if ct <= 1.0 {
        if slippery { 5.0 } else { 30.0 }
    } else if ct <= 1.1 {
        if slippery { 10.0 } else { 37.5 }
    } else {
        if slippery { 15.0 } else { 45.0 }
    };

    if slope_deg <= start {
        1.0
    } else if slope_deg >= 70.0 {
        0.0
    } else {
        (70.0 - slope_deg) / (70.0 - start)
    }
}

/// Snow density γ = min(0.426 pg + 2.2, 4.7) kN/m³ (Cap. 7 eq 4).
pub fn snow_density(pg: f64) -> f64 {
    (0.426 * pg + 2.2).min(4.7)
}

/// Drift index β (Cap. 6 eq 3).
pub fn drift_index(pg: f64) -> f64 {
    if pg <= 1.0 {
        1.0
    } else if pg >= 2.0 {
        0.5
    } else {
        1.5 - 0.5 * pg
    }
}

/// Primary roof snow calculation per CIRSOC 104-2005.
pub fn roof_snow(inputs: &SnowInputs) -> SnowResult {
    let ce_opt = exposure_factor(inputs.terrain, inputs.exposure);
    let ct = thermal_factor(inputs.thermal);
    let importance = importance_factor(inputs.category);
    let gamma = snow_density(inputs.pg);

    let mut refs = vec![
        cref("3", "cubiertas planas"),
        cref("4", "cubiertas con pendiente"),
        cref("Figura 2", "factor de pendiente"),
    ];

    let ce = match ce_opt {
        Some(val) => val,
        None => {
            return SnowResult {
                ce: 0.0,
                ct,
                importance,
                pf_computed: 0.0,
                pf_minimum: None,
                pf: 0.0,
                cs: 0.0,
                ps: 0.0,
                rain_on_snow: 0.0,
                unbalanced: None,
                gamma,
                refs,
                refused: Some("snow.refused.exposureNA".into()),
            };
        }
    };

    let pf_computed = 0.7 * ce * ct * importance * inputs.pg;
    let pf_minimum = if is_low_slope(&inputs.roof) && inputs.pg > 0.0 {
        Some(importance * inputs.pg.min(1.0))
    } else {
        None
    };

    if pf_minimum.is_some() {
        refs.push(cref("3.4", "valores mínimos de pf"));
    }

    let pf = pf_computed.max(pf_minimum.unwrap_or(0.0));
    let cs = slope_factor(inputs.roof.slope_deg, ct, inputs.roof.slippery);

    let mut rain_on_snow = 0.0;
    if inputs.pg > 0.0 && inputs.pg <= 1.0 && inputs.roof.slope_deg < 2.4 {
        let excess = match pf_minimum {
            Some(min_val) => (min_val - pf_computed).max(0.0),
            None => 0.0,
        };
        rain_on_snow = (0.25 - excess.min(0.25)).max(0.0);
        refs.push(cref("10", "carga de lluvia sobre nieve"));
    }

    let ps = cs * pf + rain_on_snow;

    let mut unbalanced = None;
    if inputs.roof.kind == RoofKind::Gable
        && inputs.roof.slope_deg <= 70.0
        && inputs.roof.slope_deg >= 21.0 / inputs.roof.w.max(1e-9) + 0.5
    {
        let ps_bal = cs * pf;
        let (leeward, windward) = if inputs.roof.w <= 6.0 {
            ((1.5 * ps_bal) / ce, 0.0)
        } else {
            (
                (1.2 * (1.0 + drift_index(inputs.pg) / 2.0) * ps_bal) / ce,
                0.3 * ps_bal,
            )
        };
        unbalanced = Some(UnbalancedSnow { leeward, windward });
        refs.push(cref("6.1", "cargas no balanceadas"));
    }

    SnowResult {
        ce,
        ct,
        importance,
        pf_computed,
        pf_minimum,
        pf,
        cs,
        ps,
        rain_on_snow,
        unbalanced,
        gamma,
        refs,
        refused: None,
    }
}
