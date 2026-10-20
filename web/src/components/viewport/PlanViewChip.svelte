<script lang="ts">
  /**
   * While the plan view is on: which level is under the camera, and the way out. A state the
   * reader lives in for a while is said on the view itself, not only in the menu that set it.
   */
  import { modelStore, uiStore } from '../../lib/store';
  import { tp, t } from '../../lib/i18n';

  const level = $derived.by(() => {
    const z = uiStore.nodeCreateZ;
    const named = (modelStore.grid?.levels ?? []).find((l) => Math.abs(l.z - z) < 1e-6);
    return named ? named.name : `+${z.toFixed(2)} m`;
  });
</script>

<div class="pv" role="status" data-testid="plan-view-chip">
  <span>{tp('viewport.planChip', { level })}</span>
  <button onclick={() => (uiStore.planView3D = false)} aria-label={t('viewport.planExit')} title={t('viewport.planExit')} data-testid="plan-view-exit">×</button>
</div>

<style>
  .pv {
    position: absolute; top: 8px; left: 50%; transform: translateX(-50%); z-index: 6;
    display: flex; align-items: center; gap: 6px; padding: 2px 4px 2px 10px;
    font-size: 0.72rem; color: var(--st-text);
    background: color-mix(in srgb, var(--st-surface) 92%, transparent);
    border: 1px solid color-mix(in srgb, var(--st-value) 45%, transparent); border-radius: var(--st-radius);
  }
  button { border: none; background: none; color: var(--st-text-2); cursor: pointer; font-size: 0.9rem; line-height: 1; padding: 2px 4px; }
  button:hover { color: var(--st-text); }
</style>
