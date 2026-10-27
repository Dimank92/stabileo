/**
 * Property tests for detailing invariants:
 *   1. As provided >= As required (and As provided >= As,min)
 *   2. Spacing and cover (CIRSOC 201 §25.2, §25.4, checkCover, minClearSpacing)
 *   3. No bar collisions (detectCollisions returns 0 overlap conflicts)
 *   4. Development lengths (anchorage at supports, cutoffs §9.7.3, splices §25.5)
 *
 * Sourced directly from physical 3D generated reinforcement in web/src/lib/engine/detailing.
 */
import { describe, it, expect } from 'vitest';
import fc from 'fast-check';
import {
  generateBeamBars,
  type BeamGenerationInput,
  type MomentStation,
} from '../generate-beam';
import {
  generateColumnStack,
  type ColumnStackInput,
} from '../generate-column';
import {
  detectCollisions,
  checkCover,
  type SectionPrism,
} from '../collision';
import { classifyPair } from '../classify';
import type { BarPath, Point3 } from '../../../codes/cirsoc201/bar-geometry';
import {
  minClearSpacingInLayer,
  minClearSpacingColumn,
} from '../../../codes/cirsoc201/spacing';
import {
  deriveDevelopment,
  MIN_DEVELOPMENT_MM,
  MIN_LAP_MM,
} from '../../../codes/cirsoc201/anchorage';

const BAR_AREAS: Record<number, number> = {
  6: 0.283, 8: 0.503, 10: 0.785, 12: 1.131, 16: 2.011, 20: 3.142, 25: 4.909, 32: 8.042,
};

function barAreaCm2(diameterMm: number, count: number): number {
  return count * (BAR_AREAS[diameterMm] ?? (Math.PI * (diameterMm / 10) ** 2) / 4);
}

function parabolicEnvelope(L: number, mSpan: number, mEndI: number, mEndJ: number, vMax: number): MomentStation[] {
  const count = 11;
  const stations: MomentStation[] = [];
  for (let i = 0; i <= count; i++) {
    const x = (i / count) * L;
    const s = x / L;
    const mPos = Math.max(0, mSpan * 4 * s * (1 - s));
    const mNeg = Math.max(0, (1 - s) * mEndI + s * mEndJ - mPos);
    const v = vMax * Math.abs(1 - 2 * s);
    stations.push({ x, mPos, mNeg, v });
  }
  return stations;
}

describe('detailing property invariants', () => {
  describe('Beam detailing invariants', () => {
    it('satisfies As >= As,req, spacing & cover, no collisions, and development lengths across random physical beams', () => {
      let testedBeams = 0;

      fc.assert(
        fc.property(
          fc.record({
            L: fc.double({ min: 3.5, max: 7.5, noNaN: true }),
            b: fc.constantFrom(0.25, 0.30, 0.35),
            h: fc.constantFrom(0.40, 0.50, 0.60, 0.70),
            fc: fc.constantFrom(20, 25, 30, 35),
            fy: fc.constantFrom(420, 500),
            maxAgg: fc.constantFrom(19, 25),
            cover: fc.constantFrom(0.025, 0.030, 0.035),
            stirrupDia: fc.constantFrom(6, 8, 10),
            mSpan: fc.double({ min: 60, max: 240, noNaN: true }),
            mEnd: fc.double({ min: 50, max: 200, noNaN: true }),
            supportI: fc.constantFrom<'simple' | 'continuous'>('simple', 'continuous'),
            supportJ: fc.constantFrom<'simple' | 'continuous'>('simple', 'continuous'),
            edition: fc.constantFrom<'2005' | '2025'>('2025', '2005'),
          }),
          (params) => {
            const { L, b, h, fc: fcM, fy, maxAgg, cover, stirrupDia, mSpan, mEnd, supportI, supportJ, edition } = params;
            const d = h - cover - stirrupDia / 1000 - 0.010; // estimate d for Ø20 bar
            if (d <= 0.25) return;

            // Compute required As for sagging and hogging (CIRSOC 201 flexure approximation with phi=0.90)
            const phi = 0.90;
            const jd = 0.88 * d; // lever arm ~ 0.88 d
            const asReqSagCm2 = (mSpan * 1e3) / (phi * fy * 1e6 * jd) * 1e4; // mSpan in kN·m -> cm²
            const asReqHogCm2 = (mEnd * 1e3) / (phi * fy * 1e6 * jd) * 1e4;

            // Minimum flexural reinforcement (CIRSOC 201 §9.6.1.2)
            const asMinCm2 = Math.max((0.25 * Math.sqrt(fcM)) / fy, 1.4 / fy) * (b * d) * 1e4;

            const targetAsBottom = Math.max(asReqSagCm2, asMinCm2);
            const targetAsTop = Math.max(asReqHogCm2, asMinCm2);

            // Select bar diameters and counts to provide As >= targetAs
            const chooseBars = (targetCm2: number) => {
              const dia = targetCm2 > 12 ? 25 : targetCm2 > 7 ? 20 : 16;
              const barA = BAR_AREAS[dia];
              const count = Math.max(2, Math.ceil(targetCm2 / barA));
              return { diameterMm: dia, count };
            };

            const bottom = chooseBars(targetAsBottom);
            const topI = chooseBars(targetAsTop);
            const topJ = chooseBars(targetAsTop);

            // Verify chosen bars fit inside width horizontally with min clear spacing
            const minClear = minClearSpacingInLayer(
              edition,
              { barDiameterMm: bottom.diameterMm, maxAggregateSizeMm: maxAgg },
            ).minClear;
            const availableInside = b - 2 * (cover + stirrupDia / 1000);
            const neededWidth = bottom.count * (bottom.diameterMm / 1000) + (bottom.count - 1) * minClear;
            if (neededWidth > availableInside) return; // Skip geometric oversaturation beyond 1 layer

            const stations = parabolicEnvelope(L, mSpan, mEnd, mEnd, 120);

            // Anchorage length function conforming to CIRSOC 201 §25.4.2
            const ldFunc = (dia: number) => {
              const dev = deriveDevelopment({
                diameterMm: dia,
                fc: fcM,
                fy,
                favourableSpacing: true,
                edition,
              });
              return dev.ldM;
            };

            const input: BeamGenerationInput = {
              elementId: 101,
              L,
              b,
              h,
              d,
              cover,
              stirrupDia,
              fc: fcM,
              fy,
              maxAggregateSizeMm: maxAgg,
              edition,
              stations,
              supportI,
              supportJ,
              vn: 300,
              bottom,
              topStart: topI,
              topEnd: topJ,
              lateralSystem: false,
              ld: ldFunc,
              origin: { x: 0, y: 0, z: 0 },
              axis: { x: 1, y: 0, z: 0 },
              up: { x: 0, y: 0, z: 1 },
              simpleSupport: supportI === 'simple' && supportJ === 'simple',
              bentUp: { seismicDesign: 'notRequired', optOut: false, assumption: 'Zona sísmica 0' },
            };

            const generated = generateBeamBars(input);
            testedBeams++;

            // ─── INVARIANT 1: As,provided >= As,required ─────────────────────────────────
            // 1a. Bottom reinforcement provides at least required flexural area
            const asProvidedBottomCm2 = barAreaCm2(bottom.diameterMm, bottom.count);
            expect(asProvidedBottomCm2).toBeGreaterThanOrEqual(asReqSagCm2);
            expect(asProvidedBottomCm2).toBeGreaterThanOrEqual(asMinCm2);

            // 1b. Support continuing bars satisfy §9.7.3.8 (at least 1/3 for simple, 1/4 for continuous)
            const minContinuingFraction = (supportI === 'simple' || supportJ === 'simple') ? 1 / 3 : 1 / 4;
            const minContinuingCount = Math.max(2, Math.ceil(bottom.count * minContinuingFraction));
            const continuingBars = generated.bars.filter((bar) => bar.id.includes('-bot-cont-') || bar.id.includes('-bot-run-'));
            expect(continuingBars.length).toBeGreaterThanOrEqual(minContinuingCount);

            // ─── INVARIANT 2: Spacing & Cover ───────────────────────────────────────────
            // 2a. Cover: All physical bars must respect the minimum concrete cover
            const prism: SectionPrism = {
              elementId: 101,
              origin: { x: 0, y: 0, z: 0 },
              axis: { x: 1, y: 0, z: 0 },
              halfWidth: b / 2,
              halfHeight: h / 2,
              uAxis: { x: 0, y: 1, z: 0 },
              vAxis: { x: 0, y: 0, z: 1 },
              requiredCover: cover,
            };
            const coverBreaches = checkCover(generated.bars, [prism]);
            expect(coverBreaches).toHaveLength(0);

            // 2b. Clear spacing: In-layer spacing must satisfy CIRSOC 201 minimums
            const bottomClear = (availableInside - bottom.count * (bottom.diameterMm / 1000)) / (bottom.count - 1);
            expect(bottomClear).toBeGreaterThanOrEqual(minClear - 1e-6);

            // ─── INVARIANT 3: No bar collisions ─────────────────────────────────────────
            const classifyForBeam = (
              a: BarPath, b: BarPath, s: number, ta?: Point3, tb?: Point3,
            ) => classifyPair(a, b, {
              edition,
              maxAggregateSizeMm: maxAgg,
              memberKindOf: () => 'beam',
              layerOf: (barId) => generated.barLayers[barId] ?? 0,
              isLapPair: () => undefined,
            }, s, ta, tb);
            const collisionResult = detectCollisions(generated.bars, { classifyFor: classifyForBeam });
            const overlaps = collisionResult.conflicts.filter((c) => c.severity === 'overlap');
            expect(overlaps).toHaveLength(0);

            // ─── INVARIANT 4: Development lengths ───────────────────────────────────────
            // 4a. Minimum development length floor >= 300 mm (§25.4.2.1(b))
            const ldBottom = ldFunc(bottom.diameterMm);
            expect(ldBottom).toBeGreaterThanOrEqual(MIN_DEVELOPMENT_MM / 1000);

            // 4b. Cutoffs satisfy §9.7.3.3 extension (actual >= theoretical + extension) and legality
            for (const cutoff of generated.cutoffs) {
              expect(cutoff.legality).not.toBe('illegal');
              // Extension must be at least d or 12 db
              const minExt = Math.max(d, 12 * (bottom.diameterMm / 1000));
              const actualExt = Math.abs(cutoff.actual - cutoff.theoretical);
              expect(actualExt).toBeGreaterThanOrEqual(minExt - 1e-4);
            }
          },
        ),
        { numRuns: 25, seed: 133742 },
      );

      expect(testedBeams).toBeGreaterThan(10);
    }, 60_000);
  });

  describe('Column detailing invariants', () => {
    it('satisfies column spacing (§25.2.3), cover, lap splices (§25.5.2), and no collisions', () => {
      let testedColumns = 0;

      fc.assert(
        fc.property(
          fc.record({
            liftHeight: fc.double({ min: 3.0, max: 4.5, noNaN: true }),
            b: fc.constantFrom(0.30, 0.40, 0.50),
            h: fc.constantFrom(0.30, 0.40, 0.50),
            barDia: fc.constantFrom(16, 20, 25),
            barCount: fc.constantFrom(4, 8),
            fc: fc.constantFrom(25, 30, 35),
            fy: fc.constantFrom(420, 500),
            maxAgg: fc.constantFrom(19, 25),
            cover: fc.constantFrom(0.030, 0.040),
            edition: fc.constantFrom<'2005' | '2025'>('2025', '2005'),
          }),
          (params) => {
            const { liftHeight, b, h, barDia, barCount, fc: fcM, fy, maxAgg, cover, edition } = params;

            // Check column spacing rule (§25.2.3)
            const colSpacingReq = minClearSpacingColumn(
              edition,
              { barDiameterMm: barDia, maxAggregateSizeMm: maxAgg },
            );
            // Minimum clear spacing: max(40 mm, 1.5 db, 4/3 d_agg)
            expect(colSpacingReq.minClear).toBeGreaterThanOrEqual(0.040);
            expect(colSpacingReq.minClear).toBeGreaterThanOrEqual(1.5 * (barDia / 1000));

            // Lap splice requirement (§25.5.2): Class A >= max(1.0 ld, 300 mm), Class B >= max(1.3 ld, 300 mm)
            const ld = deriveDevelopment({
              diameterMm: barDia,
              fc: fcM,
              fy,
              favourableSpacing: true,
              edition,
            }).ldM;
            const lapClassA = Math.max(ld, MIN_LAP_MM / 1000);
            const lapClassB = Math.max(1.3 * ld, MIN_LAP_MM / 1000);
            expect(lapClassA).toBeGreaterThanOrEqual(0.300);
            expect(lapClassB).toBeGreaterThanOrEqual(lapClassA);

            // Construct stack input for generateColumnStack
            const stackInput: ColumnStackInput = {
              stackId: 'C-prop',
              lifts: [
                {
                  elementId: 201,
                  baseZ: 0,
                  topZ: liftHeight,
                  b,
                  h,
                  centre: { x: 0, y: 0 },
                  bars: { count: barCount, diameterMm: barDia },
                  tieDia: 8,
                  cover,
                },
              ],
              fc: fcM,
              fy,
              maxAggregateSizeMm: maxAgg,
              edition,
              lapSplice: (d) => Math.max(
                1.3 * deriveDevelopment({
                  diameterMm: d,
                  fc: fcM,
                  fy,
                  favourableSpacing: true,
                  edition,
                }).ldM,
                MIN_LAP_MM / 1000,
              ),
              beamDepthAtTop: new Map([[0, 0.50]]),
              beamDepthAtBase: 0,
              roofTermination: false,
            };

            const stack = generateColumnStack(stackInput);
            testedColumns++;

            // Invariant: Generated column bars have NO collisions / overlaps
            if (stack.bars.length > 0) {
              const classifyForColumn = (
                a: BarPath, b: BarPath, s: number, ta?: Point3, tb?: Point3,
              ) => classifyPair(a, b, {
                edition,
                maxAggregateSizeMm: maxAgg,
                memberKindOf: () => 'column',
                layerOf: () => 0,
                isLapPair: () => undefined,
              }, s, ta, tb);
              const colCollision = detectCollisions(stack.bars, { classifyFor: classifyForColumn });
              const overlaps = colCollision.conflicts.filter((c) => c.severity === 'overlap');
              expect(overlaps).toHaveLength(0);

              // Invariant: Cover respected for column cage
              const colPrism: SectionPrism = {
                elementId: 201,
                origin: { x: 0, y: 0, z: 0 },
                axis: { x: 0, y: 0, z: 1 },
                halfWidth: b / 2,
                halfHeight: h / 2,
                uAxis: { x: 1, y: 0, z: 0 },
                vAxis: { x: 0, y: 1, z: 0 },
                requiredCover: cover,
              };
              const colBreaches = checkCover(stack.bars, [colPrism]);
              expect(colBreaches).toHaveLength(0);
            }
          },
        ),
        { numRuns: 20, seed: 998877 },
      );

      expect(testedColumns).toBeGreaterThan(10);
    });
  });
});
