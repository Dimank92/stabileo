import { readdirSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { join, relative } from 'node:path';
import { describe, expect, it } from 'vitest';
import { inspectE2E, SMOKE_PATTERN, SMOKE_TIMEOUT_MS } from '../e2e-policy';

const web = fileURLToPath(new URL('../../', import.meta.url));
function files(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap(entry => {
    const path = join(dir, entry.name);
    return entry.isDirectory() ? files(path) : entry.name.endsWith('.ts') ? [path] : [];
  });
}
const findings = files(join(web, 'e2e')).map(file => inspectE2E(readFileSync(file, 'utf8'), relative(web, file)));

it('every static E2E JSON import uses with { type: json }', () => {
  expect(findings.flatMap(f => f.jsonImports), 'JSON imports missing with { type: \'json\' } (file:line)').toEqual([]);
});
it('smoke specs respect the timeout cap or explain an exception on the same line', () => {
  expect(findings.flatMap(f => f.smokeTimeouts), `Timeouts above ${SMOKE_TIMEOUT_MS} ms: move to @slow or add // slow-ok: <reason>`).toEqual([]);
});

describe('policy regression cases', () => {
  it('parses multiline imports, side effects and attributes; ignores comments and dynamic imports', () => {
    expect(inspectE2E(`
      // import fake from './fake.json';
      import a from './a.json' with { type: 'json' };
      import b\n from './b.json';
      import './c.json';
      import d from './d.json' assert { type: 'json' };
      import e from './e.json' with { type: 'text' };
      const dynamic = import('./dynamic.json');
    `).jsonImports).toHaveLength(4);
  });
  it('catches multiline/numeric separator options and timeout overrides', () => {
    expect(inspectE2E(`test('@smoke x', () => {
      expect(x).toBeVisible({\n timeout: 60_000 });
      test.setTimeout(120000);
      test.describe.configure({ timeout: 90000 });
      test.slow();
      expect(x).toBeVisible({ 'timeout': 0 });
    });`).smokeTimeouts).toHaveLength(5);
  });
  it('requires a real same-line reason; a string or previous-line comment is insufficient', () => {
    expect(inspectE2E(`test('@smoke x', () => {
      f({ timeout: 45000 });
      f({ timeout: 60000 }); // slow-ok: live solve
      f({ timeout: 60000 }); // slow-ok:
      // slow-ok: not on the timeout line
      f({ timeout: 60000 });
      f({ timeout: 60000, message: '// slow-ok: fake' });
    });`).smokeTimeouts).toHaveLength(3);
  });
  it('checks shared helpers but leaves explicit slow and flaky scopes alone', () => {
    expect(inspectE2E(`
      function shared() { f({ timeout: 60000 }); }
      test('@smoke x', () => {});
      test('@slow y', () => { test.setTimeout(120000); });
      test.describe('z', { tag: ['@smoke', '@flaky'] }, () => { f({ timeout: 60000 }); });
    `).smokeTimeouts).toHaveLength(1);
    expect(inspectE2E(`// @smoke\nf({ timeout: 60000 });`).smokeTimeouts).toEqual([]);
  });
  it('routes smoke only once and preserves longer slow/flaky budgets', () => {
    expect(SMOKE_TIMEOUT_MS).toBe(45_000);
    expect(SMOKE_PATTERN.test('chromium file @smoke example')).toBe(true);
    for (const title of ['@slow example', '@smoke @flaky example', '@slow @smoke example', 'untagged']) expect(SMOKE_PATTERN.test(title)).toBe(false);
    const config = readFileSync(join(web, 'playwright.config.ts'), 'utf8');
    expect(config).toContain("name: 'smoke', grep: SMOKE_PATTERN, timeout: SMOKE_TIMEOUT_MS");
    expect(config).toContain("name: 'chromium', grepInvert: SMOKE_PATTERN");
  });
});
