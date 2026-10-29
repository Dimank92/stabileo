<script lang="ts">
  import { archStrings } from './biggins2026/strings';
  import BigginsPage from './biggins2026/BigginsPage.svelte';
  import { rs } from './i18n';
  /**
   * Stabileo Research: the gallery at /research and one page per runnable paper. Loaded as its own
   * chunk by Root.svelte; the experiment's WASM is fetched only when a simulation is started.
   */
  import { applyPageMeta, restorePageMeta } from '../../lib/page-meta';
  import { publicI18n } from '../../lib/i18n/store.svelte';
  import SiteHeader from '../site/SiteHeader.svelte';
  import LandingFooter from '../landing/LandingFooter.svelte';
  import PublicLink from '../landing/PublicLink.svelte';
  import Gallery from './Gallery.svelte';
  import TopoptPage from './andreassen2011/TopoptPage.svelte';
  import MattiassonPage from './mattiasson1981/MattiassonPage.svelte';
  import MacnealPage from './macneal1985/MacnealPage.svelte';
  import ShauerPage from './shauer2025/ShauerPage.svelte';
  import FerrariPage from './ferrari2025/FerrariPage.svelte';
  import FormPage from './pastrana2026/FormPage.svelte';
  import FaircloughPage from './fairclough2026/FaircloughPage.svelte';
  import { fs } from './fairclough2026/i18n';
  import NewmarkPage from './newmark1959/NewmarkPage.svelte';
  import CqcPage from './cqc1981/CqcPage.svelte';
  import SpringsPage from './grossman2026/SpringsPage.svelte';
  import '../landing/landing.css';
  import '../blog/blog.css';
  import './research.css';

  let { path }: { path: string } = $props();
  const route = $derived(path.replace(/\/+$/, '') || '/research');

  const PAGES: Record<string, { title: string; description: string }> = {
    '/research/benchmarks/newmark-1959': { title: 'Newmark (1959) — Stabileo Research', description: 'First version: production Newmark average acceleration against the exact SDOF step response.' },
    '/research/benchmarks/cqc-1981': { title: 'SRSS vs CQC (1981) — Stabileo Research', description: 'First version: closely spaced building modes compared with a closed-form reference.' },
    '/research': {
      title: 'Stabileo Research — computational mechanics you can run',
      description:
        'A curated catalogue of recent computational-mechanics papers, with the ones Stabileo has reproduced, re-implemented and validated against the authors’ code, runnable in the browser.',
    },
    '/research/optimization/andreassen-2011': { title: 'MBB topology optimization — Stabileo Research', description: 'SIMP topology optimization against the authors’ 88-line reference code.' },
    '/research/benchmarks/mattiasson-elastica': { title: 'Mattiasson elastica — Stabileo engine benchmarks', description: 'Production corotational cantilever with provisional references and explicit failures.' },
    '/research/benchmarks/macneal-harder-1985': { title: 'MacNeal–Harder (1985) — Stabileo engine benchmarks', description: 'Live production-engine accuracy benchmarks with explicit references, tolerances and failures.' },
    '/research/fracture/shauer-2025': {
      title: 'Phase-field fracture of a notched plate (Shauer 2025) — Stabileo Research',
      description:
        'AT2 phase-field fracture run in the browser by a Rust re-implementation validated step by step against the author’s Python reference code.',
    },
    '/research/metamaterials/grossman-2026': {
      title: 'Auxetic spring networks (Grossman & Boudaoud 2026) — Stabileo Research',
      description:
        'Spring networks solved in the browser as trusses: the auxetic transition of a disordered lattice, a re-entrant honeycomb and ordered lattices, compared with the paper’s published values.',
    },
  };

  $effect(() => {
    const meta = route === '/research/stability/ferrari-sigmund-2025' ? { title: rs().ferrariTitle, description: rs().ferrariLead } : route === '/research/form-finding/pastrana-2026' ? { title: rs().formTitle, description: rs().formLead } : route === '/research/mechanics/biggins-2026' ? { title: archStrings().title, description: archStrings().lead } : route === '/research/optimisation/fairclough-2026' ? {title:fs().title,description:fs().lead} : PAGES[route];
    if (meta) applyPageMeta({ title: meta.title, description: meta.description, locale: publicI18n.locale, path: route });
    return () => restorePageMeta();
  });
  const s = $derived(rs());
</script>

<div class="landing blog research">
  <SiteHeader current="research" />
  {#if route === '/research'}
    <Gallery />
  {:else if route === '/research/optimization/andreassen-2011'}
  <TopoptPage />
  {:else if route === '/research/benchmarks/mattiasson-elastica'}
    <MattiassonPage />
  {:else if route === '/research/benchmarks/macneal-harder-1985'}
    <MacnealPage />
  {:else if route === '/research/fracture/shauer-2025'}
    <ShauerPage />
  {:else if route === '/research/metamaterials/grossman-2026'}
    <SpringsPage />
  {:else if route === '/research/stability/ferrari-sigmund-2025'}
    <FerrariPage />
  {:else if route === '/research/form-finding/pastrana-2026'}
    <FormPage />
  {:else if route === '/research/mechanics/biggins-2026'}
    <BigginsPage />
  {:else if route === '/research/optimisation/fairclough-2026'}
    <FaircloughPage />
  {:else if route === '/research/benchmarks/newmark-1959'}
    <NewmarkPage />
  {:else if route === '/research/benchmarks/cqc-1981'}
    <CqcPage />
  {:else}
    <section class="sec sec--ink">
      <div class="wrap" style="padding-top: 7rem; padding-bottom: 4rem">
        <h1>{s.missingPage}</h1>
        <p><PublicLink to="/research">{s.backToResearch}</PublicLink></p>
      </div>
    </section>
  {/if}
  <LandingFooter />
</div>

<style>
  .research :global(.nav) {
    background: var(--lp-ink);
  }
  .research {
    font-synthesis: none;
    font-variant-numeric: tabular-nums lining-nums;
  }
  .research :global(sub),
  .research :global(sup) {
    font-size: 0.75em;
    line-height: 0;
  }
</style>
