/**
 * Research component ceiling test: ShauerPage.svelte must stay under the
 * project's 600-line component ceiling (Issue #794).
 */

import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const loc = (rel: string) =>
  readFileSync(resolve(HERE, '..', rel), 'utf8').split('\n').length;

const COMPONENT_CEILING = 600;

describe('ShauerPage stays under the component ceiling (#794)', () => {
  it('ShauerPage.svelte and extracted subcomponents are under the 600-line ceiling', () => {
    expect(loc('ShauerPage.svelte')).toBeLessThan(COMPONENT_CEILING);
    expect(loc('ShauerArticle.svelte')).toBeLessThan(COMPONENT_CEILING);
    expect(loc('ShauerParameters.svelte')).toBeLessThan(COMPONENT_CEILING);
    expect(loc('ShauerResultsPanels.svelte')).toBeLessThan(COMPONENT_CEILING);
  });
});
