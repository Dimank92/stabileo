/**
 * CIRSOC 104-2005 — verified TypeScript oracle for roof snow.
 *
 * Kept as an independent test oracle for differential testing against the
 * Rust engine (`engine/src/loads/cirsoc104_snow.rs`).
 */
import { clause, type ClauseRef } from '../../regulation';

const R = (c: string, l?: string) => clause('cirsoc-104', '2005', c, l);
export const REF_PF = R('3', 'cubiertas planas');
export const REF_MIN = R('3.4', 'valores mínimos de pf');
export const REF_PS = R('4', 'cubiertas con pendiente');
export const REF_CS = R('Figura 2', 'factor de pendiente');
export const REF_UNBALANCED = R('6.1', 'cargas no balanceadas');
export const REF_RAIN = R('10', 'carga de lluvia sobre nieve');

export type SnowTerrain = 'A' | 'B' | 'C' | 'D' | 'aboveTreeline';
export type RoofExposure = 'full' | 'partial' | 'sheltered';

export const EXPOSURE_FACTOR: Readonly<Record<SnowTerrain, Record<RoofExposure, number | null>>> = Object.freeze({
  A: { full: null, partial: 1.1, sheltered: 1.3 },
  B: { full: 0.9, partial: 1.0, sheltered: 1.2 },
  C: { full: 0.9, partial: 1.0, sheltered: 1.1 },
  D: { full: 0.8, partial: 0.9, sheltered: 1.0 },
  aboveTreeline: { full: 0.7, partial: 0.8, sheltered: null },
});

export type ThermalCondition = 'normal' | 'coldVentilated' | 'unheated' | 'greenhouse';

export const THERMAL_FACTOR: Readonly<Record<ThermalCondition, number>> = Object.freeze({
  normal: 1.0, coldVentilated: 1.1, unheated: 1.2, greenhouse: 0.85,
});

export type SnowCategory = 'I' | 'II' | 'III' | 'IV';

export const IMPORTANCE_FACTOR: Readonly<Record<SnowCategory, number>> = Object.freeze({
  I: 0.8, II: 1.0, III: 1.1, IV: 1.2,
});

export type RoofKind = 'mono' | 'gable';

export interface SnowRoof {
  kind: RoofKind;
  slopeDeg: number;
  W: number;
  slippery: boolean;
}

export function isLowSlope(r: SnowRoof): boolean {
  return r.kind === 'mono' ? r.slopeDeg < 15 : r.slopeDeg <= 21 / Math.max(r.W, 1e-9) + 0.5;
}

export function slopeFactor(slopeDeg: number, ct: number, slippery: boolean): number {
  const start = ct <= 1 ? (slippery ? 5 : 30) : ct <= 1.1 ? (slippery ? 10 : 37.5) : (slippery ? 15 : 45);
  if (slopeDeg <= start) return 1;
  if (slopeDeg >= 70) return 0;
  return (70 - slopeDeg) / (70 - start);
}

export const snowDensity = (pg: number) => Math.min(0.426 * pg + 2.2, 4.7);

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
  ce: number; ct: number; importance: number;
  pfComputed: number;
  pfMinimum: number | null;
  pf: number;
  cs: number;
  ps: number;
  rainOnSnow: number;
  unbalanced: { leeward: number; windward: number } | null;
  gamma: number;
  refs: ClauseRef[];
  refused?: string;
}

export function roofSnow(i: SnowInputs): SnowResult {
  const ce = EXPOSURE_FACTOR[i.terrain][i.exposure];
  const ct = THERMAL_FACTOR[i.thermal];
  const importance = IMPORTANCE_FACTOR[i.category];
  const gamma = snowDensity(i.pg);
  const refs = [REF_PF, REF_PS, REF_CS];
  if (ce === null) {
    return {
      ce: 0, ct, importance, pfComputed: 0, pfMinimum: null, pf: 0, cs: 0, ps: 0, rainOnSnow: 0,
      unbalanced: null, gamma, refs, refused: 'snow.refused.exposureNA',
    };
  }
  const pfComputed = 0.7 * ce * ct * importance * i.pg;
  const pfMinimum = isLowSlope(i.roof) && i.pg > 0 ? importance * Math.min(i.pg, 1) : null;
  if (pfMinimum !== null) refs.push(REF_MIN);
  const pf = Math.max(pfComputed, pfMinimum ?? 0);
  const cs = slopeFactor(i.roof.slopeDeg, ct, i.roof.slippery);

  let rainOnSnow = 0;
  if (i.pg > 0 && i.pg <= 1 && i.roof.slopeDeg < 2.4) {
    const excess = pfMinimum !== null ? Math.max(pfMinimum - pfComputed, 0) : 0;
    rainOnSnow = Math.max(0.25 - Math.min(excess, 0.25), 0);
    refs.push(REF_RAIN);
  }
  const ps = cs * pf + rainOnSnow;

  let unbalanced: SnowResult['unbalanced'] = null;
  if (i.roof.kind === 'gable' && i.roof.slopeDeg <= 70 && i.roof.slopeDeg >= 21 / Math.max(i.roof.W, 1e-9) + 0.5) {
    const psBal = cs * pf;
    unbalanced = i.roof.W <= 6
      ? { leeward: (1.5 * psBal) / ce, windward: 0 }
      : { leeward: (1.2 * (1 + driftIndex(i.pg) / 2) * psBal) / ce, windward: 0.3 * psBal };
    refs.push(REF_UNBALANCED);
  }
  return { ce, ct, importance, pfComputed, pfMinimum, pf, cs, ps, rainOnSnow, unbalanced, gamma, refs };
}
