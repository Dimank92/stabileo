<script lang="ts">
  /**
   * Slabs: the typology (drawn), the thickness, the levels, and the mesh the analysis reads them
   * with. Placed in every bay of the grid on each level chosen, in the building's concrete.
   */
  import { modelStore } from '../../../lib/store';
  import { t } from '../../../lib/i18n';
  import TypologyPicker from './TypologyPicker.svelte';
  import { placeSlabs, type PlaceResult } from '../../../lib/model/building/build-on-grid';
  import { buildingUi } from '../../../lib/store/building-ui.svelte';

  let { meshSize = $bindable(1), report }: { meshSize?: number; report: (r: PlaceResult) => void } = $props();

  const grid = $derived(modelStore.grid);
  const levels = $derived([...(grid?.levels ?? [])].sort((a, b) => a.z - b.z));
  let typology = $state('solid'), thickness = $state(0.15), chosen = $state<string[]>([]);
  const toggle = (id: string) => (chosen = chosen.includes(id) ? chosen.filter((x) => x !== id) : [...chosen, id]);
  /** Every level above the lowest, unless some are chosen. */
  const onLevels = () => (chosen.length > 0 ? chosen : levels.slice(1).map((l) => l.id));
  function place() {
    if (!grid) return;
    report(placeSlabs(grid, { levels: onLevels(), thickness, materialId: buildingUi.concreteId, meshSize, typology }));
  }
</script>

<section class="pk-card" data-testid="building-slabs">
  <TypologyPicker kind="slab" bind:value={typology} exclude={['ramp']} testid="building-slab-typology" />
  <div class="bs-grid">
    <label>{t('building.thickness')}<input type="number" min="0.05" step="0.01" bind:value={thickness} data-testid="building-slab-thickness" /></label>
    <label>{t('building.meshSize')}<input type="number" min="0.1" step="0.1" bind:value={meshSize} data-testid="building-mesh-size" /></label>
  </div>
  <div class="bs-levels" role="group" aria-label={t('building.atLevels')}>
    {#each levels as l (l.id)}
      <label><input type="checkbox" checked={chosen.includes(l.id)} onchange={() => toggle(l.id)} /> {l.name}</label>
    {/each}
  </div>
  <p class="pk-hint">{t('building.levelsDefault')}</p>
  <button class="pk-btn pk-btn-primary" onclick={place} data-testid="building-place-slabs">{t('building.place')}</button>
</section>

<style>
  .bs-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(8.5rem, 1fr)); gap: 6px; margin: 4px 0 6px; }
  .bs-grid label { display: flex; flex-direction: column; gap: 2px; font-size: 0.7rem; color: var(--st-text-2); }
  .bs-grid input { font-size: 0.72rem; }
  .bs-levels { display: flex; flex-wrap: wrap; gap: 4px 10px; font-size: 0.7rem; }
</style>
