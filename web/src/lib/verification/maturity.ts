/**
 * Authoritative verification maturity matrix lookup, read directly from
 * `docs/verification/dashboard.json`.
 *
 * Implements Issue #440:
 *   Result badges use the verification maturity levels
 *   (Certified > Benchmarked > Differential > Tested > Unverified)
 *
 * Requirements:
 * 1. Read level from docs/verification/dashboard.json (`maturity`) so badge cannot
 *    drift from the matrix. No level is hard-coded in UI code.
 * 2. A per-result certificate that failed or was not computed shows the item's level
 *    without the word "certified" (never a pass styling for an uncertified number).
 * 3. When an open safety issue affects the item (the matrix's Open issues column),
 *    the badge says so and links the issue.
 * 4. Steel results keep the "experimental" label until #389 is closed.
 * 5. Replaces the three-level wording (certified / benchmarked / unverified) of multiplier #346.
 */

import dashboard from '../../../../docs/verification/dashboard.json';

export const MATURITY_LEVELS = [
  'Certified',
  'Benchmarked',
  'Differential',
  'Tested',
  'Unverified',
] as const;

export type VerificationMaturityLevel = (typeof MATURITY_LEVELS)[number];

export interface MaturityItem {
  readonly name: string;
  readonly level: VerificationMaturityLevel;
  readonly evidence: Record<string, string>;
  readonly issues: readonly number[];
  readonly category: 'analyses' | 'elements' | 'design';
}

export interface MaturityBadgeResolution {
  readonly name: string;
  readonly matrixLevel: VerificationMaturityLevel;
  readonly displayLevel: VerificationMaturityLevel;
  readonly openIssues: readonly number[];
  readonly hasOpenIssues: boolean;
  readonly issueLinks: readonly string[];
  readonly isExperimental: boolean;
  readonly passStyling: boolean;
  readonly tone: 'ok' | 'benchmarked' | 'differential' | 'tested' | 'unverified' | 'warn';
  readonly glyph: string;
  readonly evidenceText: string;
  readonly evidenceSources: Record<string, string>;
}

// Ingest from dashboard.json
const rawAnalyses = (dashboard.maturity?.analyses ?? []) as Array<{
  name: string;
  level: VerificationMaturityLevel;
  evidence: Record<string, string>;
  issues: number[];
}>;

const rawElements = (dashboard.maturity?.elements ?? []) as Array<{
  name: string;
  level: VerificationMaturityLevel;
  evidence: Record<string, string>;
  issues: number[];
}>;

const rawDesign = (dashboard.maturity?.design ?? []) as Array<{
  name: string;
  level: VerificationMaturityLevel;
  evidence: Record<string, string>;
  issues: number[];
}>;

export const analysesMaturity: readonly MaturityItem[] = rawAnalyses.map((a) => ({
  ...a,
  category: 'analyses' as const,
}));

export const elementsMaturity: readonly MaturityItem[] = rawElements.map((e) => ({
  ...e,
  category: 'elements' as const,
}));

export const designMaturity: readonly MaturityItem[] = rawDesign.map((d) => ({
  ...d,
  category: 'design' as const,
}));

export function allMaturityEntries(): readonly MaturityItem[] {
  return [...analysesMaturity, ...elementsMaturity, ...designMaturity];
}

function normalizeKey(str: string): string {
  return str.toLowerCase().replace(/[^a-z0-9]/g, '');
}

// Lookup maps
const analysisMap = new Map<string, MaturityItem>();
for (const item of analysesMaturity) {
  analysisMap.set(item.name.toLowerCase(), item);
  analysisMap.set(normalizeKey(item.name), item);
}

const elementMap = new Map<string, MaturityItem>();
for (const item of elementsMaturity) {
  elementMap.set(item.name.toLowerCase(), item);
  elementMap.set(normalizeKey(item.name), item);
}

const designMap = new Map<string, MaturityItem>();
for (const item of designMaturity) {
  designMap.set(item.name.toLowerCase(), item);
  designMap.set(normalizeKey(item.name), item);
}

// Common aliases for analyses
const ANALYSIS_ALIASES: Record<string, string> = {
  linear: 'Linear static',
  static: 'Linear static',
  'linear static': 'Linear static',
  pdelta: 'P-Delta (second order)',
  'p-delta': 'P-Delta (second order)',
  buckling: 'Linear buckling',
  modal: 'Modal',
  spectral: 'Response spectrum (SRSS / CQC)',
  'response spectrum': 'Response spectrum (SRSS / CQC)',
  th: 'Time history',
  'time history': 'Time history',
  time_history: 'Time history',
  harmonic: 'Harmonic (steady state)',
  plastic: 'Plastic collapse (2D, 3D)',
  corotational: 'Co-rotational (geometric nonlinear)',
  'material nonlinear': 'Material nonlinear',
  'fiber nonlinear': 'Fiber nonlinear',
  'arc length': 'Arc length',
  arclength: 'Arc length',
  'displacement control': 'Displacement control',
  staged: 'Staged construction',
  winkler: 'Winkler foundation',
  contact: 'Contact / gaps',
  ssi: 'Soil-structure interaction (p-y, t-z, q-z)',
  cable: 'Cable solver',
  guyan: 'Guyan reduction',
  craigbampton: 'Craig–Bampton',
  kinematic: 'Kinematic / mechanism check',
  movingloads: 'Moving loads',
  influencelines: 'Influence lines',
};

// Common aliases for design checks
const DESIGN_ALIASES: Record<string, string> = {
  aisc360: 'AISC 360 steel (`steel_check`)',
  'steel_check': 'AISC 360 steel (`steel_check`)',
  ec3: 'EN 1993-1-1 steel (`ec3_check`)',
  'ec3_check': 'EN 1993-1-1 steel (`ec3_check`)',
  ec2: 'EN 1992-1-1 RC (`ec2_check`)',
  'ec2_check': 'EN 1992-1-1 RC (`ec2_check`)',
  aci318: 'ACI 318-19 RC (`rc_check`)',
  'rc_check': 'ACI 318-19 RC (`rc_check`)',
  cirsoc201: 'CIRSOC 201 RC, engine (`cirsoc201_check`)',
  'cirsoc201_check': 'CIRSOC 201 RC, engine (`cirsoc201_check`)',
  cirsoc301: 'CIRSOC 301 steel, engine (`cirsoc301_check`)',
  'cirsoc301_check': 'CIRSOC 301 steel, engine (`cirsoc301_check`)',
  timber: 'NDS timber (`timber_check`)',
  'timber_check': 'NDS timber (`timber_check`)',
  cfs: 'AISI S100 cold-formed (`cfs_check`)',
  'cfs_check': 'AISI S100 cold-formed (`cfs_check`)',
  masonry: 'TMS 402 masonry (`masonry_check`)',
  'masonry_check': 'TMS 402 masonry (`masonry_check`)',
  connection: 'Connections, engine (`connection_check`)',
  'connection_check': 'Connections, engine (`connection_check`)',
  foundation: 'Spread footings, engine (`foundation_check`)',
  'foundation_check': 'Spread footings, engine (`foundation_check`)',
  serviceability: 'Serviceability (`serviceability`)',
  'check_serviceability': 'Serviceability (`serviceability`)',
};

// Element aliases
const ELEMENT_ALIASES: Record<string, string> = {
  frame: 'Frame, Euler–Bernoulli (2D, 3D)',
  beam: 'Frame, Euler–Bernoulli (2D, 3D)',
  column: 'Frame, Euler–Bernoulli (2D, 3D)',
  truss: 'Truss',
  timoshenko: 'Frame, Timoshenko (shear deformable)',
  warping: 'Warping torsion (7th DOF)',
  curvedbeam: 'Curved beam',
  connector: 'Connector (spring element)',
  cable: 'Cable / catenary',
  fiberbeam: 'Fiber beam-column',
  plate: 'Plate, DKT triangle',
  quad: 'Shell, MITC4 quad',
  quad9: 'Shell, MITC9 quad',
  curvedshell: 'Curved shell',
  solidshell: 'Solid shell, SHB8-ANS',
};

export function getAnalysisMaturity(nameOrKey: string): MaturityItem | undefined {
  const direct = analysisMap.get(nameOrKey.toLowerCase()) ?? analysisMap.get(normalizeKey(nameOrKey));
  if (direct) return direct;
  const canonical = ANALYSIS_ALIASES[normalizeKey(nameOrKey)] ?? ANALYSIS_ALIASES[nameOrKey.toLowerCase()];
  if (canonical) return analysisMap.get(canonical.toLowerCase());
  return undefined;
}

export function getDesignCheckMaturity(nameOrKey: string): MaturityItem | undefined {
  const direct = designMap.get(nameOrKey.toLowerCase()) ?? designMap.get(normalizeKey(nameOrKey));
  if (direct) return direct;
  const canonical = DESIGN_ALIASES[normalizeKey(nameOrKey)] ?? DESIGN_ALIASES[nameOrKey.toLowerCase()];
  if (canonical) return designMap.get(canonical.toLowerCase());
  // Substring match for clause or code check names
  for (const item of designMaturity) {
    if (item.name.toLowerCase().includes(nameOrKey.toLowerCase())) return item;
  }
  return undefined;
}

export function getElementMaturity(nameOrKey: string): MaturityItem | undefined {
  const direct = elementMap.get(nameOrKey.toLowerCase()) ?? elementMap.get(normalizeKey(nameOrKey));
  if (direct) return direct;
  const canonical = ELEMENT_ALIASES[normalizeKey(nameOrKey)] ?? ELEMENT_ALIASES[nameOrKey.toLowerCase()];
  if (canonical) return elementMap.get(canonical.toLowerCase());
  return undefined;
}

export interface ResolveBadgeOptions {
  category?: 'analyses' | 'elements' | 'design';
  nameOrKey: string;
  perResultCertified?: boolean;
}

/**
 * Resolves the display properties of a verification maturity badge according
 * to Issue #440 requirements.
 */
export function resolveMaturityBadge(options: ResolveBadgeOptions): MaturityBadgeResolution {
  const { category, nameOrKey, perResultCertified = false } = options;

  let item: MaturityItem | undefined;
  if (category === 'analyses') item = getAnalysisMaturity(nameOrKey);
  else if (category === 'design') item = getDesignCheckMaturity(nameOrKey);
  else if (category === 'elements') item = getElementMaturity(nameOrKey);
  else {
    item = getAnalysisMaturity(nameOrKey) ?? getDesignCheckMaturity(nameOrKey) ?? getElementMaturity(nameOrKey);
  }

  const name = item?.name ?? nameOrKey;
  const matrixLevel = item?.level ?? 'Unverified';
  const openIssues = item?.issues ?? [];
  const hasOpenIssues = openIssues.length > 0;
  const issueLinks = openIssues.map((num) => `https://github.com/lambdaclass/stabileo/issues/${num}`);

  // Steel experimental check: steel results keep "experimental" label until #389 is closed
  const isSteel =
    name.toLowerCase().includes('steel') ||
    name.toLowerCase().includes('301') ||
    name.toLowerCase().includes('aisc');
  const isExperimental = isSteel && openIssues.includes(389);

  // Per-result certification rule:
  // A per-result certificate that failed or was not computed shows the item's
  // level without the word "certified" (never a pass styling for an uncertified number).
  let displayLevel: VerificationMaturityLevel;
  let passStyling = false;

  if (matrixLevel === 'Certified') {
    if (perResultCertified) {
      displayLevel = 'Certified';
      passStyling = true;
    } else {
      // Find the next level in evidence
      const ev = item?.evidence ?? {};
      if (ev.Benchmarked) displayLevel = 'Benchmarked';
      else if (ev.Differential) displayLevel = 'Differential';
      else if (ev.Tested) displayLevel = 'Tested';
      else displayLevel = 'Unverified';
      passStyling = false;
    }
  } else {
    displayLevel = matrixLevel;
    passStyling = false;
  }

  let tone: 'ok' | 'benchmarked' | 'differential' | 'tested' | 'unverified' | 'warn';
  if (passStyling) {
    tone = 'ok';
  } else if (isExperimental) {
    tone = 'warn';
  } else {
    switch (displayLevel) {
      case 'Benchmarked':
        tone = 'benchmarked';
        break;
      case 'Differential':
        tone = 'differential';
        break;
      case 'Tested':
        tone = 'tested';
        break;
      case 'Unverified':
      default:
        tone = 'unverified';
        break;
    }
  }

  const glyphMap: Record<VerificationMaturityLevel, string> = {
    Certified: '✓',
    Benchmarked: '★',
    Differential: '⇋',
    Tested: '🧪',
    Unverified: '○',
  };
  const glyph = isExperimental ? '⚗' : glyphMap[displayLevel];

  const evidenceSources = item?.evidence ?? {};
  const evidenceText = Object.entries(evidenceSources)
    .map(([lvl, ev]) => `${lvl}: ${ev}`)
    .join('; ');

  return {
    name,
    matrixLevel,
    displayLevel,
    openIssues,
    hasOpenIssues,
    issueLinks,
    isExperimental,
    passStyling,
    tone,
    glyph,
    evidenceText,
    evidenceSources,
  };
}

