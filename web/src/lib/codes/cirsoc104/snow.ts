/**
 * CIRSOC 104-2005 — snow load on roofs: a thin adapter over the engine.
 *
 * The check is the engine's (`engine/src/loads/cirsoc104_snow.rs`, op `cirsoc104Snow`):
 *   Cap. 3  (1)  p_f = 0,7 C_e C_t I p_g,  with, on low-slope roofs (§3.4), p_f ≥ I·p_g for
 *                p_g ≤ 1 kN/m² and p_f ≥ I·1 kN/m² above that;
 *   Cap. 4  (2)  p_s = C_s p_f,  on the horizontal projection of the roof;
 *   Cap. 6       unbalanced load on gable roofs, §6.1 with β from (3);
 *   Cap. 7  (4)  γ = 0,426 p_g + 2,2 ≤ 4,70 kN/m³;
 *   Cap. 10      rain on snow, 0,25 kN/m² on roofs under 2,4° where 0 < p_g ≤ 1 kN/m².
 *
 * This module maps the app's `SnowInputs` in and the engine's numbers out.
 * The verified TypeScript implementation is kept as the differential oracle
 * (`__tests__/snow-oracle.ts`), compared in `snow-engine.test.ts`.
 */
import { clause, type ClauseRef } from '../regulation';
import { loadsRequest } from '../../engine/wasm-solver';

const R = (c: string, l?: string) => clause('cirsoc-104', '2005', c, l);
export const REF_PF = R('3', 'cubiertas planas');
export const REF_MIN = R('3.4', 'valores mínimos de pf');
export const REF_PS = R('4', 'cubiertas con pendiente');
export const REF_CS = R('Figura 2', 'factor de pendiente');
export const REF_UNBALANCED = R('6.1', 'cargas no balanceadas');
export const REF_RAIN = R('10', 'carga de lluvia sobre nieve');

export type SnowTerrain = 'A' | 'B' | 'C' | 'D' | 'aboveTreeline';
export type RoofExposure = 'full' | 'partial' | 'sheltered';

/** Tabla 2, C_e. Null where the table prints N/A. */
export const EXPOSURE_FACTOR: Readonly<Record<SnowTerrain, Record<RoofExposure, number | null>>> = Object.freeze({
  A: { full: null, partial: 1.1, sheltered: 1.3 },
  B: { full: 0.9, partial: 1.0, sheltered: 1.2 },
  C: { full: 0.9, partial: 1.0, sheltered: 1.1 },
  D: { full: 0.8, partial: 0.9, sheltered: 1.0 },
  aboveTreeline: { full: 0.7, partial: 0.8, sheltered: null },
});

export type ThermalCondition = 'normal' | 'coldVentilated' | 'unheated' | 'greenhouse';

/** Tabla 3, C_t. */
export const THERMAL_FACTOR: Readonly<Record<ThermalCondition, number>> = Object.freeze({
  normal: 1.0, coldVentilated: 1.1, unheated: 1.2, greenhouse: 0.85,
});

export type SnowCategory = 'I' | 'II' | 'III' | 'IV';

/** Tabla 4, I. */
export const IMPORTANCE_FACTOR: Readonly<Record<SnowCategory, number>> = Object.freeze({
  I: 0.8, II: 1.0, III: 1.1, IV: 1.2,
});

export type RoofKind = 'mono' | 'gable';

export interface SnowRoof {
  kind: RoofKind;
  slopeDeg: number;
  /** Horizontal distance from the ridge to the eave, m (§1.3: W). */
  W: number;
  /** A smooth surface with no obstructions and room under the eaves for what slides off (Cap. 4). */
  slippery: boolean;
}

/** §3.4: the roofs the minimum values apply to. */
export function isLowSlope(r: SnowRoof): boolean {
  return r.kind === 'mono' ? r.slopeDeg < 15 : r.slopeDeg <= 21 / Math.max(r.W, 1e-9) + 0.5;
}

/** Figura 2. */
export function slopeFactor(slopeDeg: number, ct: number, slippery: boolean): number {
  const start = ct <= 1 ? (slippery ? 5 : 30) : ct <= 1.1 ? (slippery ? 10 : 37.5) : (slippery ? 15 : 45);
  if (slopeDeg <= start) return 1;
  if (slopeDeg >= 70) return 0;
  return (70 - slopeDeg) / (70 - start);
}

/** (4), kN/m³. */
export const snowDensity = (pg: number) => Math.min(0.426 * pg + 2.2, 4.7);

/** (3). */
export function driftIndex(pg: number): number {
  if (pg <= 1) return 1;
  if (pg >= 2) return 0.5;
  return 1.5 - 0.5 * pg;
}

export interface SnowInputs {
  pg: number;
  terrain: SnowTerrain;
  exposure: RoofExposure;
  thermal: ThermalCondition;
  category: SnowCategory;
  roof: SnowRoof;
}

export interface SnowResult {
  ce: number;
  ct: number;
  importance: number;
  /** (1) as computed, and the §3.4 minimum when it applies. */
  pfComputed: number;
  pfMinimum: number | null;
  pf: number;
  cs: number;
  /** Balanced load on the horizontal projection, rain on snow included. */
  ps: number;
  rainOnSnow: number;
  /** §6.1 on a gable roof when required: the leeward side and the windward side. */
  unbalanced: { leeward: number; windward: number } | null;
  gamma: number;
  refs: ClauseRef[];
  /** i18n key when Tabla 2 prints N/A for the terrain and exposure chosen. */
  refused?: string;
}

function engineRoofSnow(i: SnowInputs): SnowResult {
  const res = loadsRequest({
    op: 'cirsoc104Snow',
    inputs: {
      pg: i.pg,
      terrain: i.terrain,
      exposure: i.exposure,
      thermal: i.thermal,
      category: i.category,
      roof: {
        kind: i.roof.kind,
        slopeDeg: i.roof.slopeDeg,
        w: i.roof.W,
        slippery: i.roof.slippery,
      },
    },
  }) as {
    ce: number;
    ct: number;
    importance: number;
    pfComputed: number;
    pfMinimum: number | null;
    pf: number;
    cs: number;
    ps: number;
    rainOnSnow: number;
    unbalanced: { leeward: number; windward: number } | null;
    gamma: number;
    refs: Array<{ regulation: string; edition: string; clause: string; label: string }>;
    refused?: string;
  };

  return {
    ce: res.ce,
    ct: res.ct,
    importance: res.importance,
    pfComputed: res.pfComputed,
    pfMinimum: res.pfMinimum,
    pf: res.pf,
    cs: res.cs,
    ps: res.ps,
    rainOnSnow: res.rainOnSnow,
    unbalanced: res.unbalanced,
    gamma: res.gamma,
    refs: res.refs.map((r) => clause(r.regulation, r.edition, r.clause, r.label)),
    ...(res.refused ? { refused: res.refused } : {}),
  };
}

export function roofSnow(i: SnowInputs): SnowResult {
  return engineRoofSnow(i);
}
