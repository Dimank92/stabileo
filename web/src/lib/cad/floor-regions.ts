/**
 * Several floor plans drawn in one DXF, side by side or one above the other, read as the floors
 * of one building.
 *
 * ── Finding them ──────────────────────────────────────────────────
 *
 * The drawing's structural layers are split into clusters (`drawingRegions`); clusters much
 * smaller than the largest (a detail, a north arrow, a title block) are left out. They are given
 * in reading order for a sheet: left to right when they are laid in a row, bottom to top when
 * they are stacked in a column, so the first one is offered as the lowest floor.
 *
 * ── Laying them on each other ─────────────────────────────────────
 *
 * Each plan keeps its own drawing coordinates, so two plans drawn 30 m apart would be stacked
 * 30 m apart. Each plan gets a reference point, the point that goes to the model's origin:
 *
 *   the lowest plan  its lower-left column (its box's lower-left corner when it has none)
 *   every other one  the translation that lays most of its columns on the columns of the plan
 *                    below it; found by voting on the translations between every pair of
 *                    columns, then counting the matches of the best few. When fewer than two
 *                    columns (or under a third of the smaller plan's) match, its lower-left corner
 *                    is laid on the lower-left corner of the plan below.
 *
 * A reference given by hand is kept and the plans above it are laid on it.
 *
 * Pure: the drawing and its extracted plans in, windows and points out.
 */
import { CAD_UNIT_SCALE, type ArchPlan, type CadDocument, type CadEntity, type CadPt, type CadUnit, type LayerMapping } from './types';
import { drawingRegions } from '../model/underlay-regions';
import type { PlanWindow } from './infer';

/** The separate plans of a drawing, as crop windows in its raw units, in reading order. */
export function detectPlanWindows(doc: CadDocument, unit: CadUnit, mappings: readonly LayerMapping[]): PlanWindow[] {
  const k = CAD_UNIT_SCALE[unit];
  const structural = new Set(mappings.filter((m) => m.role !== 'ignore').map((m) => m.layer));
  const regions = drawingRegions(doc, unit, structural.size ? { layers: structural } : {});
  if (regions.length < 2) return [];
  const area = (b: { minX: number; minY: number; maxX: number; maxY: number }) => Math.max(b.maxX - b.minX, 1e-9) * Math.max(b.maxY - b.minY, 1e-9);
  const maxArea = Math.max(...regions.map((r) => area(r.box)));
  const maxCount = Math.max(...regions.map((r) => r.count));
  const kept = regions.filter((r) => area(r.box) >= 0.05 * maxArea && r.count >= Math.max(3, 0.05 * maxCount));
  if (kept.length < 2) return [];
  const cx = kept.map((r) => (r.box.minX + r.box.maxX) / 2), cy = kept.map((r) => (r.box.minY + r.box.maxY) / 2);
  const inRow = Math.max(...cx) - Math.min(...cx) >= Math.max(...cy) - Math.min(...cy);
  const order = kept.map((_, i) => i).sort((a, b) => (inRow ? cx[a]! - cx[b]! || cy[a]! - cy[b]! : cy[a]! - cy[b]! || cx[a]! - cx[b]!));
  return order.map((i) => {
    const b = kept[i]!.box;
    // A margin, so the crop (which keeps entities wholly inside) keeps the ones on the border.
    const pad = 0.01 * Math.max(b.maxX - b.minX, b.maxY - b.minY) + 0.05;
    return { x0: (b.minX - pad) / k, x1: (b.maxX + pad) / k, y0: (b.minY - pad) / k, y1: (b.maxY + pad) / k };
  });
}

/** The drawing moved by (dx, dy) in its raw units. */
export function shiftDoc(doc: CadDocument, dx: number, dy: number): CadDocument {
  if (dx === 0 && dy === 0) return doc;
  const p = (q: CadPt): CadPt => ({ x: q.x + dx, y: q.y + dy });
  const entities = doc.entities.map((e): CadEntity => {
    switch (e.kind) {
      case 'line': return { ...e, a: p(e.a), b: p(e.b) };
      case 'polyline': return { ...e, pts: e.pts.map(p) };
      case 'arc': case 'circle': return { ...e, center: p(e.center) };
      case 'insert': return { ...e, at: p(e.at), bbox: e.bbox && { minX: e.bbox.minX + dx, minY: e.bbox.minY + dy, maxX: e.bbox.maxX + dx, maxY: e.bbox.maxY + dy } };
      case 'text': return { ...e, at: p(e.at) };
    }
  });
  const bbox = doc.bbox && { minX: doc.bbox.minX + dx, minY: doc.bbox.minY + dy, maxX: doc.bbox.maxX + dx, maxY: doc.bbox.maxY + dy };
  return { ...doc, entities, bbox };
}

export interface PlanAlignment {
  /** The point that goes to the model's origin, metres, in the drawing's coordinates. */
  ref: CadPt;
  /** How it was found: the lowest plan's own point, matching columns, the corners, or by hand. */
  method: 'base' | 'columns' | 'corner' | 'manual';
  /** Columns laid on columns of the plan below (method 'columns'). */
  matched: number;
}

const MAX_POINTS = 500;

/** The points a plan is laid by: its columns, or its members' ends when it has under three. */
function anchors(plan: ArchPlan): CadPt[] {
  const pts = plan.columns.map((c) => c.at);
  if (pts.length >= 3) return pts.slice(0, MAX_POINTS);
  const seen = new Set<string>();
  const out = [...pts];
  for (const m of [...plan.beams, ...plan.walls]) for (const q of [m.a, m.b]) {
    const key = `${q.x.toFixed(4)},${q.y.toFixed(4)}`;
    if (!seen.has(key)) { seen.add(key); out.push(q); }
  }
  return out.slice(0, MAX_POINTS);
}

function lowerLeft(plan: ArchPlan): CadPt | null {
  let x = Infinity, y = Infinity;
  const take = (q: CadPt) => { x = Math.min(x, q.x); y = Math.min(y, q.y); };
  for (const c of plan.columns) take(c.at);
  for (const m of [...plan.beams, ...plan.walls]) { take(m.a); take(m.b); }
  for (const s of plan.slabs) s.outline.forEach(take);
  return Number.isFinite(x) ? { x, y } : null;
}

/** The translation laying most of `Q` on `P` (within `tol`), and how many points it lays. */
function bestTranslation(P: CadPt[], Q: CadPt[], tol: number, prefer: CadPt): { t: CadPt; score: number } | null {
  if (!P.length || !Q.length) return null;
  const votes = new Map<string, { n: number; t: CadPt }>();
  for (const p of P) for (const q of Q) {
    const t = { x: p.x - q.x, y: p.y - q.y };
    const key = `${Math.round(t.x / tol)},${Math.round(t.y / tol)}`;
    const v = votes.get(key);
    if (v) v.n++; else votes.set(key, { n: 1, t });
  }
  const grid = new Map<string, CadPt[]>();
  const cell = (x: number) => Math.floor(x / tol);
  for (const p of P) { const key = `${cell(p.x)},${cell(p.y)}`; (grid.get(key) ?? grid.set(key, []).get(key)!).push(p); }
  const score = (t: CadPt) => {
    let n = 0;
    for (const q of Q) {
      const x = q.x + t.x, y = q.y + t.y;
      let hit = false;
      for (let i = cell(x) - 1; i <= cell(x) + 1 && !hit; i++) for (let j = cell(y) - 1; j <= cell(y) + 1 && !hit; j++) {
        for (const p of grid.get(`${i},${j}`) ?? []) if (Math.hypot(p.x - x, p.y - y) <= tol) { hit = true; break; }
      }
      if (hit) n++;
    }
    return n;
  };
  const top = [...votes.values()].sort((a, b) => b.n - a.n).slice(0, 8);
  let best: { t: CadPt; score: number; d: number } | null = null;
  for (const v of top) {
    const s = score(v.t), d = Math.hypot(v.t.x - prefer.x, v.t.y - prefer.y);
    if (!best || s > best.score || (s === best.score && d < best.d)) best = { t: v.t, score: s, d };
  }
  return best && { t: best.t, score: best.score };
}

/**
 * The reference point of each plan, given lowest floor first; `fixed[i]` (metres) is a point
 * chosen by hand for plan i, kept as is. `tol` is how close two columns must land, metres.
 */
export function alignPlans(plans: readonly ArchPlan[], fixed: ReadonlyArray<CadPt | null | undefined> = [], tol = 0.05): PlanAlignment[] {
  const out: PlanAlignment[] = [];
  plans.forEach((plan, i) => {
    const hand = fixed[i];
    if (hand) { out.push({ ref: hand, method: 'manual', matched: 0 }); return; }
    const own = anchors(plan);
    const corner = lowerLeft(plan) ?? { x: 0, y: 0 };
    if (i === 0) {
      const cols = plan.columns.map((c) => c.at);
      const ll = cols.length ? cols.reduce((a, b) => (b.x + b.y < a.x + a.y - 1e-9 ? b : a)) : corner;
      out.push({ ref: ll, method: 'base', matched: 0 });
      return;
    }
    const below = plans[i - 1]!, prev = out[i - 1]!;
    const belowCorner = lowerLeft(below) ?? { x: 0, y: 0 };
    // The corners laid on each other: the fallback, and the tie-break between equal matches.
    const cornerT = { x: belowCorner.x - corner.x, y: belowCorner.y - corner.y };
    const found = bestTranslation(anchors(below), own, tol, cornerT);
    const enough = found && found.score >= 2 && found.score >= Math.ceil(0.3 * Math.min(anchors(below).length, own.length));
    const t = enough ? found!.t : cornerT;
    out.push({ ref: { x: prev.ref.x - t.x, y: prev.ref.y - t.y }, method: enough ? 'columns' : 'corner', matched: enough ? found!.score : 0 });
  });
  return out;
}
