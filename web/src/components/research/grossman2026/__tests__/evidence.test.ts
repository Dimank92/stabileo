import { expect, it } from 'vitest';
import { evidence } from '../evidence';

it('counts only published comparisons as paper checks', () => {
  expect(evidence.map(({ category, total, passed }) => ({ category, total, passed }))).toEqual([
    { category: 'paper', total: 8, passed: 8 },
    { category: 'independent', total: 21, passed: 21 },
  ]);
  expect(evidence[0].checks.every((check) => /^(Supplemental|Fig\.|Eq\.|foam:)/.test(check.name))).toBe(true);
});
