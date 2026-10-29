/**
 * The experiments that run on this site, and the ones being built. The gallery and the
 * experiment pages read this; papers.json (the catalogue) is joined to it by `paperId`.
 */
export type Experiment = {
  paperId: string;
  path: string;
  name: string;
  hook: string;
  /** web/public/<thumbnail>, rendered from our own run by scripts/research-thumbs.mjs and committed. */
  thumbnail: string;
  /** The page's query that sets up the thumbnail's view (step, mesh, view, deformed). */
  thumbnailQuery: string;
  code: string;
};

export const EXPERIMENTS: Experiment[] = [
  {
    paperId: 'shauer-2025',
    path: '/research/fracture/shauer-2025',
    name: 'Phase-field fracture of a notched plate',
    hook: 'Pull a notched plate apart and watch an AT2 phase-field crack form and run, solved in your browser and checked step by step against the author’s code.',
    thumbnail: 'research-thumbs/shauer-2025.webp',
    thumbnailQuery: 'step=70&mesh=1&view=plate&deformed=1&deform=4&broken=show',
    code: 'https://github.com/lambdaclass/stabileo/tree/main/research/fracture/shauer-2025',
  },
  {
    paperId: 'zmyslony-2026',
    path: '/research/mechanics/biggins-2026',
    name: 'Snapping arches by curvature patterning',
    hook: 'Production buckling modes and a shallow-arch equilibrium jump, checked against published thresholds.',
    thumbnail: 'research-thumbs/biggins-2026.webp',
    thumbnailQuery: '',
    code: 'https://github.com/lambdaclass/stabileo/tree/main/research/mechanics/biggins-2026',
  },
];

/** The "Coming next" row; set false to hide it. */
export const SHOW_COMING_NEXT = true;

export type Upcoming = { name: string; field: string; note: string };

export const COMING_NEXT: Upcoming[] = [
  { name: 'Topology optimization of an MBB beam', field: 'optimization', note: 'SIMP with density filter and optimality criteria, against the 88-line reference results' },
  { name: 'Mechanics of spring networks', field: 'metamaterials', note: 'Truss networks against the paper’s published numbers (no reference code)' },
];
