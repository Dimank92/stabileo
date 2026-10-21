import { expect, it } from 'vitest';
import { frameDesignQuality } from './design-quality-fixture';

// Final passes on 184184c32, before lap-aware seeds. Preserve those members, not just the
// total: a higher count could conceal a regression in an already verified column or beam.
const PREVIOUS_PASSES = [2, 4, 6, 7, 8, 9, 10, 13, 16, 17, 18, 19, 20, 38, 42, 43, 44, 48,
  52, 53, 54, 55, 58, 63, 68, 70, 72, 73, 74, 77, 78, 79, 82, 83, 84, 87, 88, 89, 92, 93,
  94, 97, 98, 99, 102, 103, 104, 107, 108, 109, 113, 114, 115, 118, 119, 121, 122, 123, 124,
  125, 136, 137, 138, 139, 140, 142, 143, 144, 147, 148, 149, 152, 153, 154, 157, 158, 159,
  185, 201, 202];

it('verifies four more columns through final detailing without losing previous passes', async () => {
  const { output, verifiedIds } = await frameDesignQuality();
  expect(output.aborted).toBe(false);
  expect(output.notReached).toBe(0);
  expect(Object.values(output.counts).reduce((a, b) => a + b, 0)).toBe(408);
  expect(verifiedIds).toEqual(expect.arrayContaining([...PREVIOUS_PASSES, 12, 14, 32, 34]));
  expect(verifiedIds.length).toBeGreaterThanOrEqual(84);
  expect(output.exhausted).toBeLessThanOrEqual(35);
  expect(output.counts.fail).toBeLessThanOrEqual(276);
  expect(output.blockingConflicts).toBeLessThanOrEqual(2981);
  const achievedOpen: Record<string, number> = {
    generation: 2, 'detailing.clash.unresolved': 1818, 'detailing.lapCheck.unarranged': 97,
    'detailing.lapCheck.integrity': 42, 'detailing.lapCheck.spacing': 9,
    'detailing.lapCheck.pitch': 4, 'detailing.lapCheck.noPartner': 2,
  };
  for (const [key, count] of Object.entries(output.open)) {
    expect(count, key).toBeLessThanOrEqual(achievedOpen[key] ?? 0);
  }
}, 600_000);
