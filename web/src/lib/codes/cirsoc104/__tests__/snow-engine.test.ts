/**
 * Differential test: CIRSOC 104 roof snow, Rust engine against verified TypeScript oracle.
 *
 * `roofSnow` in `../snow.ts` delegates to the Rust engine (`engine/src/loads/cirsoc104_snow.rs`,
 * op `cirsoc104Snow`). The verified TypeScript implementation is kept in `snow-oracle.ts`.
 * Both must agree on every number, clause reference, unbalanced load, and refused state.
 */
import { describe, it, expect } from 'vitest';
import fc from 'fast-check';
import { roofSnow, type SnowInputs, type SnowTerrain, type RoofExposure, type ThermalCondition, type SnowCategory, type RoofKind } from '../snow';
import { roofSnow as oracle } from './snow-oracle';

const TERRAINS: SnowTerrain[] = ['A', 'B', 'C', 'D', 'aboveTreeline'];
const EXPOSURES: RoofExposure[] = ['full', 'partial', 'sheltered'];
const THERMALS: ThermalCondition[] = ['normal', 'coldVentilated', 'unheated', 'greenhouse'];
const CATEGORIES: SnowCategory[] = ['I', 'II', 'III', 'IV'];
const ROOF_KINDS: RoofKind[] = ['mono', 'gable'];

describe('CIRSOC 104 roof snow: engine against verified TypeScript oracle', () => {
  it('base cases agree exactly', () => {
    const cases: SnowInputs[] = [
      {
        pg: 1.5, terrain: 'C', exposure: 'partial', thermal: 'normal', category: 'II',
        roof: { kind: 'gable', slopeDeg: 20, W: 8, slippery: false },
      },
      {
        pg: 0.8, terrain: 'C', exposure: 'partial', thermal: 'normal', category: 'II',
        roof: { kind: 'mono', slopeDeg: 3, W: 8, slippery: false },
      },
      {
        pg: 3.0, terrain: 'C', exposure: 'partial', thermal: 'normal', category: 'III',
        roof: { kind: 'mono', slopeDeg: 3, W: 8, slippery: false },
      },
      {
        pg: 0.6, terrain: 'C', exposure: 'partial', thermal: 'normal', category: 'II',
        roof: { kind: 'mono', slopeDeg: 1, W: 8, slippery: false },
      },
      {
        pg: 1.2, terrain: 'C', exposure: 'partial', thermal: 'normal', category: 'II',
        roof: { kind: 'gable', slopeDeg: 34, W: 9, slippery: false },
      },
      {
        pg: 1.5, terrain: 'A', exposure: 'full', thermal: 'normal', category: 'II',
        roof: { kind: 'gable', slopeDeg: 20, W: 8, slippery: false },
      },
      {
        pg: 1.5, terrain: 'aboveTreeline', exposure: 'sheltered', thermal: 'normal', category: 'II',
        roof: { kind: 'gable', slopeDeg: 20, W: 8, slippery: false },
      },
    ];

    for (const inp of cases) {
      const actual = roofSnow(inp);
      const expected = oracle(inp);

      expect(actual.ce).toBeCloseTo(expected.ce, 10);
      expect(actual.ct).toBeCloseTo(expected.ct, 10);
      expect(actual.importance).toBeCloseTo(expected.importance, 10);
      expect(actual.pfComputed).toBeCloseTo(expected.pfComputed, 10);
      if (expected.pfMinimum === null) {
        expect(actual.pfMinimum).toBeNull();
      } else {
        expect(actual.pfMinimum).toBeCloseTo(expected.pfMinimum, 10);
      }
      expect(actual.pf).toBeCloseTo(expected.pf, 10);
      expect(actual.cs).toBeCloseTo(expected.cs, 10);
      expect(actual.ps).toBeCloseTo(expected.ps, 10);
      expect(actual.rainOnSnow).toBeCloseTo(expected.rainOnSnow, 10);
      expect(actual.gamma).toBeCloseTo(expected.gamma, 10);
      expect(actual.refused).toEqual(expected.refused);

      if (expected.unbalanced === null) {
        expect(actual.unbalanced).toBeNull();
      } else {
        expect(actual.unbalanced!.leeward).toBeCloseTo(expected.unbalanced.leeward, 10);
        expect(actual.unbalanced!.windward).toBeCloseTo(expected.unbalanced.windward, 10);
      }

      expect(actual.refs.map((r) => r.clause)).toEqual(expected.refs.map((r) => r.clause));
    }
  });

  it('property-based random sweeps match oracle on thousands of inputs', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...TERRAINS),
        fc.constantFrom(...EXPOSURES),
        fc.constantFrom(...THERMALS),
        fc.constantFrom(...CATEGORIES),
        fc.constantFrom(...ROOF_KINDS),
        fc.double({ min: 0.1, max: 10.0, noNaN: true }),
        fc.double({ min: 0.0, max: 80.0, noNaN: true }),
        fc.double({ min: 1.0, max: 50.0, noNaN: true }),
        fc.boolean(),
        (terrain, exposure, thermal, category, kind, pg, slopeDeg, W, slippery) => {
          const inp: SnowInputs = {
            pg,
            terrain,
            exposure,
            thermal,
            category,
            roof: { kind, slopeDeg, W, slippery },
          };

          const actual = roofSnow(inp);
          const expected = oracle(inp);

          expect(actual.ce).toBeCloseTo(expected.ce, 8);
          expect(actual.ct).toBeCloseTo(expected.ct, 8);
          expect(actual.importance).toBeCloseTo(expected.importance, 8);
          expect(actual.pfComputed).toBeCloseTo(expected.pfComputed, 8);
          if (expected.pfMinimum === null) {
            expect(actual.pfMinimum).toBeNull();
          } else {
            expect(actual.pfMinimum).toBeCloseTo(expected.pfMinimum, 8);
          }
          expect(actual.pf).toBeCloseTo(expected.pf, 8);
          expect(actual.cs).toBeCloseTo(expected.cs, 8);
          expect(actual.ps).toBeCloseTo(expected.ps, 8);
          expect(actual.rainOnSnow).toBeCloseTo(expected.rainOnSnow, 8);
          expect(actual.gamma).toBeCloseTo(expected.gamma, 8);
          expect(actual.refused).toEqual(expected.refused);

          if (expected.unbalanced === null) {
            expect(actual.unbalanced).toBeNull();
          } else {
            expect(actual.unbalanced!.leeward).toBeCloseTo(expected.unbalanced.leeward, 8);
            expect(actual.unbalanced!.windward).toBeCloseTo(expected.unbalanced.windward, 8);
          }

          expect(actual.refs.map((r) => r.clause)).toEqual(expected.refs.map((r) => r.clause));
        },
      ),
      { numRuns: 1000, seed: 104 },
    );
  });
});
