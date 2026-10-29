<script lang="ts">
  import type { ToolId } from './calculators';
  let { id }: { id: ToolId } = $props();
</script>

<svg viewBox="0 0 600 300" fill="none" aria-hidden="true">
  <path d="M70 160H530" stroke="var(--fg-2)" stroke-width="3" />
  {#if id === 'simply-supported-beam' || id === 'column-buckling'}
    <path d="M70 160L55 185H85Z M530 160L515 185H545Z M510 193H550 M50 193H90" stroke="var(--fg-2)" stroke-width="2" />
  {:else}
    <path d="M70 125V195 M50 140L70 125 M50 155L70 140 M50 170L70 155 M50 185L70 170 M50 200L70 185" stroke="var(--fg-2)" stroke-width="2" />
  {/if}
  {#if id === 'column-buckling'}
    <path d="M70 160Q300 20 530 160 M575 160H540L550 150 M540 160L550 170" stroke="var(--lp-accent)" stroke-width="3" />
  {:else}
    {#each [110, 170, 230, 290, 350, 410, 470, 525] as x}
      <path d={`M${x} 75V145 m-6 -8 l6 8 l6 -8`} stroke="var(--fg-2)" stroke-width="2" />
    {/each}
    <path d={id === 'cantilever-beam' ? 'M70 160C230 160 390 220 530 245' : 'M70 160Q300 280 530 160'} stroke="var(--lp-accent)" stroke-width="3" />
  {/if}
</svg>
<style>svg { display: block; width: 100%; }</style>
