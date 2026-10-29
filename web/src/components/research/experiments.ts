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
  /** What the thumbnail shows (its alt text). */
  thumbnailAlt: string;
  /** The page's query that sets up the thumbnail's view (step, mesh, view, deformed). */
  thumbnailQuery: string;
  code: string;
  /** The page embeds the Stabileo editor with the experiment's model (opened by `#edit`). */
  editor?: boolean;
};

export const EXPERIMENTS: Experiment[] = [
  { paperId: 'andreassen-2011', path: '/research/optimization/andreassen-2011', name: 'Topology optimization of an MBB beam', hook: 'Redistribute material using SIMP and compare both filters against the authors’ code.', thumbnail: 'research-thumbs/andreassen-2011.svg', thumbnailAlt: 'Material density of the optimized MBB beam', thumbnailQuery: '', code: 'https://github.com/lambdaclass/stabileo/tree/main/research/optimization/andreassen-2011' },
  {
    paperId: 'pastrana-2026',
    path: '/research/form-finding/pastrana-2026',
    name: 'Four cables, one equilibrium',
    hook: 'Forward force density method on the paper’s four-bar topology.',
    thumbnail: 'research-thumbs/pastrana-2026.svg',
    thumbnailAlt: 'Projection of our four-cable equilibrium result',
    thumbnailQuery: '',
    editor: true,
    code: 'https://github.com/lambdaclass/stabileo/tree/main/research/form-finding/pastrana-2026',
  },
  {
    paperId: 'fairclough-2026',
    path: '/research/optimisation/fairclough-2026',
    name: 'Five-node gridshell', hook: 'Table 1: one fixed square under self-weight',
    thumbnail: 'research-thumbs/fairclough-2026.svg',
    thumbnailAlt: 'Four curved members from the Stabileo run', thumbnailQuery: '', editor: true,
    code: 'https://github.com/lambdaclass/stabileo/tree/main/research/optimisation/fairclough-2026',
  },
  {
    paperId: 'shauer-2025',
    path: '/research/fracture/shauer-2025',
    name: 'Phase-field fracture of a notched plate',
    hook: 'Pull a notched plate apart and watch an AT2 phase-field crack form and run, solved in your browser and checked step by step against the author’s code.',
    thumbnail: 'research-thumbs/shauer-2025.webp',
    thumbnailAlt: 'Damage field from our run of the experiment',
    thumbnailQuery: 'step=70&mesh=1&view=plate&deformed=1&deform=4&broken=show',
    code: 'https://github.com/lambdaclass/stabileo/tree/main/research/fracture/shauer-2025',
  },
  {
    paperId: 'grossman-2026',
    path: '/research/metamaterials/grossman-2026',
    name: 'Auxetic spring networks',
    hook: 'Shake the nodes of a triangular lattice of springs until it widens when pulled: a truss network solved in your browser, against the paper’s published numbers.',
    thumbnail: 'research-thumbs/grossman-2026.webp',
    thumbnailAlt: 'The auxetic spring network (eta = 0.5) from our run, bars coloured by axial force on the deformed strip',
    thumbnailQuery: 'eta=0.5&thumb',
    editor: true,
    code: 'https://github.com/lambdaclass/stabileo/tree/main/research/metamaterials/grossman-2026',
  },
  {
    paperId: 'zmyslony-2026',
    path: '/research/mechanics/biggins-2026',
    name: 'Snapping arches by curvature patterning',
    hook: 'Production buckling modes and a shallow-arch equilibrium jump, checked against published thresholds.',
    thumbnail: 'research-thumbs/biggins-2026.webp',
    thumbnailAlt: 'Arch equilibrium',
    thumbnailQuery: '',
    code: 'https://github.com/lambdaclass/stabileo/tree/main/research/mechanics/biggins-2026',
  },
];

/** The "Coming next" row; set false to hide it. */
export const SHOW_COMING_NEXT = true;

export type Upcoming = { name: string; field: string; note: string };

export const COMING_NEXT: Upcoming[] = [
];
