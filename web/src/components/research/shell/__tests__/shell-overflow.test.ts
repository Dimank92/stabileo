/**
 * Research shell desktop pane scrollability on short windows (Issue #802).
 * Verifies that ExperimentShell and PaneTabs configure internal overflow-y
 * scrolling on the sticky pane and tabpanels so panel contents (Network,
 * Parameters, etc.) are never hidden on short desktop windows.
 */

import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const readShell = () => readFileSync(resolve(HERE, '..', 'ExperimentShell.svelte'), 'utf8');
const readTabs = () => readFileSync(resolve(HERE, '..', 'PaneTabs.svelte'), 'utf8');

describe('Research shell sticky pane overflow (#802)', () => {
  it('ExperimentShell sticky desktop pane allows internal scrolling and bounds height', () => {
    const src = readShell();
    // The sticky pane must have overflow-y: auto (not bare overflow: hidden) so content is not cut off
    expect(src).toMatch(/\.rs-shell-app\s*\{[^}]*overflow-y:\s*auto/);
    expect(src).toMatch(/\.rs-shell-app\s*\{[^}]*max-height:\s*calc\(100dvh/);
  });

  it('ExperimentShell ensures tabpanels within the pane can scroll on short windows', () => {
    const src = readShell();
    expect(src).toMatch(/\[role=["']tabpanel["']\]\)?\s*\{[^}]*overflow-y:\s*auto/);
    expect(src).toMatch(/\[role=["']tabpanel["']\]\)?\s*\{[^}]*min-height:\s*0/);
  });

  it('PaneTabs remains docked at top when scrolling pane contents', () => {
    const src = readTabs();
    expect(src).toMatch(/\.rs-tabs\s*\{[^}]*position:\s*sticky/);
    expect(src).toMatch(/\.rs-tabs\s*\{[^}]*top:\s*0/);
  });
});
