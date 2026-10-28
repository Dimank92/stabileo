/**
 * Analysis algorithms that moved into the engine are kept only as test oracles.
 *
 * `web/src/lib/engine/__tests__/differential/oracle/` holds the legacy TypeScript analysis
 * implementations before the Rust engine took them over (plastic collapse, rigid diaphragm penalty
 * beams, CIRSOC 201/301, section stress, etc.). They exist strictly to be compared with the engine
 * in differential parity tests.
 *
 * Production code importing one or re-defining its algorithms brings back an unverified,
 * uncertified duplicate, so no production file may import from differential oracles or re-implement
 * the banned duplicate functions.
 */
import { describe, it, expect } from 'vitest';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const SRC = join(fileURLToPath(import.meta.url), '..', '..', '..', '..');

function sources(dir: string, out: string[] = []): string[] {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) {
      if (name === '__tests__' || name === 'wasm' || name === 'node_modules') continue;
      sources(p, out);
    } else if (/\.(ts|svelte)$/.test(name) && !/\.test\.ts$/.test(name)) {
      out.push(p);
    }
  }
  return out;
}

describe('analysis oracles stay in tests and out of production', () => {
  it('no production module imports differential oracles', () => {
    const imports = /(?:from|import)\s*\(?\s*['"][^'"]*differential\/oracle[^'"]*['"]/;
    const offenders = sources(SRC).filter((f) => imports.test(readFileSync(f, 'utf8')));
    expect(offenders).toEqual([]);
  });

  it('no production module imports legacy plastic-collapse or rigid-diaphragm penalty solvers', () => {
    const bannedImports = [
      /(?:from|import)\s*\(?\s*['"][^'"]*plastic-collapse['"]/,
      /(?:from|import)\s*\{[^}]*applyRigidDiaphragm[^}]*\}\s*from/,
    ];
    const offenders: string[] = [];
    for (const f of sources(SRC)) {
      const src = readFileSync(f, 'utf8');
      for (const re of bannedImports) {
        if (re.test(src)) offenders.push(`${f}: ${re}`);
      }
    }
    expect(offenders).toEqual([]);
  });

  it('the moved analysis algorithms have no TypeScript definition left in production', () => {
    const banned = [
      /function\s+applyRigidDiaphragm\b/,
      /function\s+plasticCollapse2D\b/,
    ];
    const offenders: string[] = [];
    for (const f of sources(SRC)) {
      const src = readFileSync(f, 'utf8').replace(/\/\*[\s\S]*?\*\//g, '').replace(/\/\/.*$/gm, '');
      for (const re of banned) {
        if (re.test(src)) offenders.push(`${f}: ${re}`);
      }
    }
    expect(offenders).toEqual([]);
  });
});

