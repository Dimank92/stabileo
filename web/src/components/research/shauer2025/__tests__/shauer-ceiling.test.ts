/**
 * Issue #794: split ShauerPage.svelte under the 600-line component ceiling.
 */

import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const shauerDir = resolve(HERE, '..');
const loc = (rel: string) =>
  readFileSync(resolve(shauerDir, rel), 'utf8').split('\n').length;

const COMPONENT_MAX = 600;

describe('ShauerPage component ceiling (#794)', () => {
  it('ShauerPage.svelte is strictly under the 600-line component ceiling', () => {
    const lines = loc('ShauerPage.svelte');
    expect(lines).toBeLessThan(COMPONENT_MAX);
  });

  it('all Svelte components in shauer2025 are strictly under the 600-line ceiling', () => {
    const svelteFiles = readdirSync(shauerDir).filter((f) => f.endsWith('.svelte'));
    expect(svelteFiles.length).toBeGreaterThan(0);
    for (const file of svelteFiles) {
      const lines = loc(file);
      expect(lines, `${file} exceeds ${COMPONENT_MAX} lines (${lines})`).toBeLessThan(COMPONENT_MAX);
    }
  });
});
