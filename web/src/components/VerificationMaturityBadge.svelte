<script lang="ts">
  /**
   * Verification maturity badge for results and design checks.
   *
   * Displays the verification maturity level derived strictly from
   * `docs/verification/dashboard.json` without hardcoding:
   *   Certified > Benchmarked > Differential > Tested > Unverified
   *
   * Implements Issue #440:
   * - Reads level from dashboard.json (maturity), built into bundle at release time.
   * - A per-result certificate that failed or was not computed shows the item's level
   *   without the word "certified" (never a pass styling for an uncertified number).
   * - When an open safety issue affects the item, the badge says so and links the issue.
   * - Steel results keep the "experimental" label until #389 is closed.
   */
  import {
    resolveMaturityBadge,
    type ResolveBadgeOptions,
    type MaturityBadgeResolution,
  } from '../lib/verification/maturity';

  interface Props {
    category?: 'analyses' | 'elements' | 'design';
    nameOrKey: string;
    perResultCertified?: boolean;
    compact?: boolean;
    showIssues?: boolean;
    customLabel?: string;
  }

  const {
    category,
    nameOrKey,
    perResultCertified = false,
    compact = false,
    showIssues = true,
    customLabel,
  }: Props = $props();

  const resolution: MaturityBadgeResolution = $derived(
    resolveMaturityBadge({
      category,
      nameOrKey,
      perResultCertified,
    })
  );

  const tooltip = $derived.by(() => {
    const parts = [
      `${resolution.name}: ${resolution.displayLevel} (matrix: ${resolution.matrixLevel})`,
    ];
    if (resolution.isExperimental) {
      parts.push('[Experimental steel result: open issue #389]');
    }
    if (resolution.hasOpenIssues) {
      parts.push(`Open issues: ${resolution.openIssues.map((n) => `#${n}`).join(', ')}`);
    }
    if (resolution.evidenceText) {
      parts.push(`Evidence: ${resolution.evidenceText}`);
    }
    return parts.join('\n');
  });
</script>

<span
  class="badge verification-maturity-badge tone-{resolution.tone}"
  class:pass={resolution.passStyling}
  class:experimental={resolution.isExperimental}
  data-testid="verification-maturity-badge"
  data-matrix-level={resolution.matrixLevel}
  data-display-level={resolution.displayLevel}
  data-category={category}
  data-name={nameOrKey}
  title={tooltip}
>
  <span class="badge-glyph" aria-hidden="true">{resolution.glyph}</span>

  {#if !compact}
    <span class="badge-text">{customLabel ?? resolution.displayLevel}</span>
  {/if}

  {#if resolution.isExperimental}
    <span class="badge-tag" data-testid="experimental-tag">exp</span>
  {/if}

  {#if showIssues && resolution.hasOpenIssues}
    <span class="badge-issues" data-testid="maturity-issues">
      {#each resolution.openIssues.slice(0, 2) as issueNum, idx}
        <a
          href={`https://github.com/lambdaclass/stabileo/issues/${issueNum}`}
          target="_blank"
          rel="noopener noreferrer"
          class="issue-link"
          title={`Safety issue #${issueNum}`}
          onclick={(e) => e.stopPropagation()}
        >#{issueNum}</a>
      {/each}
      {#if resolution.openIssues.length > 2}
        <span class="more-issues">+{resolution.openIssues.length - 2}</span>
      {/if}
    </span>
  {/if}

  <span class="sr-only">
    {resolution.name} verification maturity: {resolution.displayLevel}
    {#if resolution.isExperimental} (experimental){/if}
    {#if resolution.hasOpenIssues}. Affected by open issues: {resolution.openIssues.join(', ')}{/if}
  </span>
</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 1px 6px;
    border-radius: 3px;
    font-size: 0.67rem;
    font-weight: 600;
    line-height: 1.4;
    white-space: nowrap;
    border: 1px solid transparent;
    vertical-align: middle;
    font-family: inherit;
  }

  .badge-glyph {
    font-size: 0.72rem;
  }

  .badge-text {
    font-weight: 500;
  }

  /* Tones based on verification level */
  .tone-ok {
    background: rgba(34, 204, 102, 0.16);
    color: var(--st-ok, #22cc66);
    border-color: var(--st-ok, #22cc66);
  }

  .tone-benchmarked {
    background: rgba(30, 144, 255, 0.14);
    color: var(--st-info, #38bdf8);
    border-color: rgba(56, 189, 248, 0.4);
  }

  .tone-differential {
    background: rgba(147, 51, 234, 0.14);
    color: #c084fc;
    border-color: rgba(192, 132, 252, 0.4);
  }

  .tone-tested {
    background: rgba(100, 116, 139, 0.18);
    color: var(--st-text-2, #cbd5e1);
    border-color: var(--st-hair-strong, #475569);
  }

  .tone-unverified {
    background: rgba(100, 116, 139, 0.10);
    color: var(--st-text-3, #94a3b8);
    border-color: var(--st-hair, #334155);
  }

  .tone-warn {
    color: var(--st-warn, #eab308);
    border-color: var(--st-warn, #eab308);
    background: repeating-linear-gradient(
      45deg,
      rgba(221, 170, 0, 0.18) 0 4px,
      rgba(120, 92, 0, 0.18) 4px 8px
    );
  }

  .badge-tag {
    font-size: 0.6rem;
    text-transform: uppercase;
    padding: 0 3px;
    border-radius: 2px;
    background: rgba(234, 179, 8, 0.25);
    color: var(--st-warn, #eab308);
    border: 1px solid var(--st-warn, #eab308);
  }

  .badge-issues {
    display: inline-flex;
    gap: 2px;
    align-items: center;
    font-size: 0.62rem;
    font-family: var(--st-mono, monospace);
  }

  .issue-link {
    color: var(--st-warn, #f59e0b);
    text-decoration: underline;
    text-underline-offset: 1px;
    font-size: 0.62rem;
  }

  .issue-link:hover {
    color: var(--st-text, #ffffff);
  }

  .more-issues {
    color: var(--st-text-3, #94a3b8);
    font-size: 0.58rem;
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>

