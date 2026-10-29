import { describe, it, expect } from 'vitest';
import { readFileSync, existsSync } from 'node:fs';
import { resolve } from 'node:path';

describe('Issue #735: Finish rename from Dedaliano to Stabileo', () => {
  const rootDir = resolve(__dirname, '../../../../');

  it('FeedbackWidget does not contain legacy dedaliano contact URLs', () => {
    const feedbackPath = resolve(rootDir, 'web/src/components/FeedbackWidget.svelte');
    expect(existsSync(feedbackPath)).toBe(true);
    const content = readFileSync(feedbackPath, 'utf-8');
    expect(content).not.toContain('t.me/dedaliano');
    expect(content).not.toContain('cafecito.app/dedaliano');
    expect(content).toContain('t.me/stabileo');
    expect(content).toContain('cafecito.app/stabileo');
  });

  it('CHANGELOG.md and engine/README.md do not contain hardcoded laptop paths', () => {
    const changelogPath = resolve(rootDir, 'CHANGELOG.md');
    const engineReadmePath = resolve(rootDir, 'engine/README.md');

    const changelog = readFileSync(changelogPath, 'utf-8');
    expect(changelog).not.toContain('/Users/unbalancedparen/projects/dedaliano');

    const engineReadme = readFileSync(engineReadmePath, 'utf-8');
    expect(engineReadme).not.toContain('/Users/unbalancedparen/projects/dedaliano');
    expect(engineReadme).toContain('# Stabileo Engine');
  });

  it('Roadmaps have Stabileo titles', () => {
    const aiRoadmap = readFileSync(resolve(rootDir, 'docs/roadmap/AI_ROADMAP.md'), 'utf-8');
    const infraRoadmap = readFileSync(resolve(rootDir, 'docs/roadmap/INFRASTRUCTURE_ROADMAP.md'), 'utf-8');
    const solverRoadmap = readFileSync(resolve(rootDir, 'docs/roadmap/SOLVER_ROADMAP.md'), 'utf-8');
    const productRoadmap = readFileSync(resolve(rootDir, 'docs/roadmap/PRODUCT_ROADMAP.md'), 'utf-8');

    expect(aiRoadmap).toMatch(/^# Stabileo AI Roadmap/m);
    expect(infraRoadmap).toMatch(/^# Stabileo Infrastructure Roadmap/m);
    expect(solverRoadmap).toMatch(/^# Stabileo Solver Roadmap/m);
    expect(productRoadmap).toMatch(/^# Stabileo Product Roadmap/m);
  });

  it('Code comments and logs use Stabileo branding', () => {
    const fileStore = readFileSync(resolve(rootDir, 'web/src/lib/store/file.ts'), 'utf-8');
    expect(fileStore).toContain('[Stabileo] File loaded as 2D');
    expect(fileStore).not.toContain('[Dedaliano] File loaded as 2D');
  });

  it('Preserves backward-compatible localStorage migration keys', () => {
    const fileStore = readFileSync(resolve(rootDir, 'web/src/lib/store/file.ts'), 'utf-8');
    expect(fileStore).toContain("dedaliano-autosave");

    const uiStore = readFileSync(resolve(rootDir, 'web/src/lib/store/ui.svelte.ts'), 'utf-8');
    expect(uiStore).toContain("`dedaliano-${key}`");
  });
});
