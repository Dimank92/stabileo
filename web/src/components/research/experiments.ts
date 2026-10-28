/**
 * The experiments that run on this site, and the ones being built. The gallery and the
 * experiment pages read this; papers.json (the catalogue) is joined to it by `paperId`.
 */
export type Experiment = {
  paperId: string;
  path: string;
  name: string;
  hook: string;
  /** Rendered at build time from our own run (prerender), absent until it has been generated. */
  thumbnail: string;
  code: string;
};

export const EXPERIMENTS: Experiment[] = [
  {
    paperId: 'shauer-2025',
    path: '/research/fracture/shauer-2025',
    name: 'Phase-field fracture of a notched plate',
    hook: 'Pull a notched plate apart and watch an AT2 phase-field crack form and run, solved in your browser and checked step by step against the author’s code.',
    thumbnail: 'research-thumbs/shauer-2025.webp',
    code: 'https://github.com/lambdaclass/stabileo/tree/main/research/fracture/shauer-2025',
  },
  {
    paperId: 'grossman-2026',
    path: '/research/metamaterials/grossman-2026',
    name: 'Auxetic spring networks',
    hook: 'Shake the nodes of a triangular lattice of springs until it widens when pulled: a truss network solved in your browser, against the paper’s published numbers.',
    thumbnail: 'research-thumbs/grossman-2026.webp',
    code: 'https://github.com/lambdaclass/stabileo/tree/main/research/metamaterials/grossman-2026',
  },
];

export type Upcoming = { name: string; field: string; note: string };

export const COMING_NEXT: Upcoming[] = [
  { name: 'Topology optimization of an MBB beam', field: 'optimization', note: 'SIMP with density filter and optimality criteria, against the 88-line reference results' },
];
