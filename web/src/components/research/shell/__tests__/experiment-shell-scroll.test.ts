import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

describe('ExperimentShell panel overflow scrolling on short desktop windows (#802)', () => {
  const componentPath = resolve(__dirname, '../ExperimentShell.svelte');
  const source = readFileSync(componentPath, 'utf8');

  it('declares sticky desktop positioning on .rs-shell-app with full available height', () => {
    expect(source).toMatch(/\.rs-shell-app\s*\{[^}]*position:\s*sticky;/);
    expect(source).toMatch(/\.rs-shell-app\s*\{[^}]*height:\s*calc\(100dvh\s*-\s*var\(--lp-nav-h,\s*3\.6rem\)\s*-\s*1\.5rem\);/);
  });

  it('configures .rs-shell-app direct child to fill pane height as a flex column', () => {
    // Immediate child of .rs-shell-app must stretch to full height and enable child flexing
    expect(source).toMatch(/\.rs-shell-app\s*>\s*:global\(\*\)\s*\{[^}]*height:\s*100%;/);
    expect(source).toMatch(/\.rs-shell-app\s*>\s*:global\(\*\)\s*\{[^}]*display:\s*flex;/);
    expect(source).toMatch(/\.rs-shell-app\s*>\s*:global\(\*\)\s*\{[^}]*flex-direction:\s*column;/);
    expect(source).toMatch(/\.rs-shell-app\s*>\s*:global\(\*\)\s*\{[^}]*min-height:\s*0;/);
  });

  it('allows active tabpanels to scroll vertically within the sticky pane', () => {
    // Active panels must flex and scroll so parameters/network controls are not hidden on short windows
    expect(source).toMatch(
      /\.rs-shell-app\s*:global\(\[role="tabpanel"\]:not\(\[hidden\]\)\)[^{]*\{[^}]*overflow-y:\s*auto;/,
    );
    expect(source).toMatch(
      /\.rs-shell-app\s*:global\(\[role="tabpanel"\]:not\(\[hidden\]\)\)[^{]*\{[^}]*flex:\s*1\s*1\s*auto;/,
    );
    expect(source).toMatch(
      /\.rs-shell-app\s*:global\(\[role="tabpanel"\]:not\(\[hidden\]\)\)[^{]*\{[^}]*min-height:\s*0;/,
    );
  });

  it('applies standard research scrollbar styling to scrolling tabpanels', () => {
    expect(source).toMatch(
      /\.rs-shell-app\s*:global\(\[role="tabpanel"\]:not\(\[hidden\]\)\)[^{]*\{[^}]*scrollbar-color:\s*var\(--fg-2\)\s*var\(--hair\);/,
    );
    expect(source).toMatch(
      /\.rs-shell-app\s*:global\(\[role="tabpanel"\]:not\(\[hidden\]\)\)[^{]*\{[^}]*scrollbar-gutter:\s*stable;/,
    );
  });

  it('ensures hidden panels remain hidden', () => {
    expect(source).toMatch(
      /\.rs-shell-app\s*:global\(\[role="tabpanel"\]\[hidden\]\)[^{]*\{[^}]*display:\s*none;/,
    );
  });
});
