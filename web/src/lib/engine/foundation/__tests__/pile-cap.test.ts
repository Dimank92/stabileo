/**
 * A four-pile cap by hand: piles at (±0,75, ±0,75), Ø0,40 m, cap 2,3 × 2,3 × 0,9 m, column 0,50 m.
 *   Service N = 1600 kN + cap 25·2,3²·0,9 = 119,0 kN: R = 429,8 kN each; My = 200 kN·m adds
 *   ±200·0,75/(4·0,75²) = ±66,7 → 496,4 / 363,1 kN.
 *   Factored N = 2240 kN, centred: 560 kN each. Face at 0,25 m: two piles 0,50 m past it,
 *   Mu = 2·560·0,50 = 560 kN·m.
 */
import { describe, it, expect } from 'vitest';
import { checkPileCap, pileReactions } from '../pile-cap';

const piles = [[-0.75, -0.75], [0.75, -0.75], [0.75, 0.75], [-0.75, 0.75]].map(([x, y]) => ({ x: x!, y: y! }));

describe('a four-pile cap', () => {
  it('shares the load and the moment between its piles', () => {
    const r = pileReactions(piles, 1600 + 119.025, 200, 0);
    expect(Math.max(...r)).toBeCloseTo(496.42, 1);
    expect(Math.min(...r)).toBeCloseTo(363.09, 1);
  });

  it('bends at the column face by the piles beyond it', () => {
    const c = checkPileCap({
      B: 2.3, L: 2.3, thickness: 0.9, cover: 0.075, fc: 25, fy: 420, piles, pileDiameter: 0.4, pileCapacity: 600,
      column: { x: 0, y: 0, cx: 0.5, cy: 0.5 }, service: { N: 1600, Mx: 0, My: 0 }, factored: { N: 2240, Mx: 0, My: 0 },
    });
    expect(c.reactions.factored.every((v) => Math.abs(v - 560) < 1e-9)).toBe(true);
    expect(c.x.mu).toBeCloseTo(560, 6);
    expect(c.piles.tension).toBe(false);
    expect(c.piles.utilization).toBeCloseTo((1600 + 119.025) / 4 / 600, 6);
  });
});
