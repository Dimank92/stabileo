import { publicI18n } from '../../lib/i18n/store.svelte';

/** Research interface and article strings; unsupported locales fall back to English. */
export type ResearchStrings = {
  forceComparisonTolerance: string;
  referenceDots: string;

  coarseMesh: string;
  mediumMesh: string;
  referenceMesh: string;
  viewer: string;
  force: string;
  convergence: string;
  parameters: string;
  precomputedSeries: string;
  browserSeries: string;
  referenceSeries: string;
  referenceDifference: string;
  elasticLabel: string;
  fractureLabel: string;
  elasticEnergy: string;
  fractureEnergy: string;
  thisReplay: string;
  thisRun: string;
  nonConverged: string;
  referenceMismatch: string;
  damageField: string;
  stressAvailable: string;
  hideBroken: string;
  hideBrokenHint: string;
  plate: string;
  crackBand: string;
  notchTip: string;
  degradedStress: string;
  clampedColours: string;
  elementCentres: string;
  pause: string;
  play: string;
  equal: string;
  differs: string;
  pass: string;
  fail: string;
  notConvergedSuffix: string;
  referenceLower: string;
  noLicence: string;
  deformed: string;
  undeformed: string;
  inTime: string;
  peakReferenceLabel: string;
  peakLabel: string;
  viewerMeshLabel: string;
  replaySource: string;
  liveSource: string;
  cachedSource: string;

  researchTitle: string;
  galleryLead: string;
  galleryDisclaimer: string;
  damageAlt: string;
  missingPage: string;
  backToResearch: string;
  iterationsChartLabel: string;
  iterations: string;
  loadStepLower: string;
  step: string;
  residualSuffix: string;
  shauerTitle: string;
  shauerLead: string;
  paperShows: string;
  paperOverview: string;
  replayGrowth: string;
  theForce: string;
  risesTo: string;
  peakStepLink: string;
  nextIncrement: string;
  crackStepLink: string;
  afterComma: string;
  alternating308: string;
  forceDrops: string;
  finalState: string;
  openCrack: string;
  modelHeading: string;
  displacement: string;
  bilinearModel: string;
  elasticity: string;
  fixedDamage: string;
  wholeStress: string;
  damageAT2: string;
  atFixed: string;
  drivingEnergy: string;
  positiveStrains: string;
  hybridModel: string;
  alternatingSolution: string;
  solvePerStep: string;
  currentDamage: string;
  residualThreshold: string;
  irreversibility: string;
  noIrreversibility: string;
  enableHistory: string;
  keepOnUnloading: string;
  unitsAndPeak: string;
  lengthStrength: string;
  tryLength: string;
  meshMatters: string;
  tryCoarse: string;
  reproductionHeading: string;
  referenceProcedure: string;
  rustProcedure: string;
  checksProcedure: string;
  referenceComparison: string;
  validationChecks: string;
  checksCaption: string;
  check: string;
  observed: string;
  tolerance: string;
  result: string;
  symmetryFinding: string;
  toleranceMeaning: string;
  stepTable: string;
  stepsParen: string;
  residual: string;
  referenceForce: string;
  paperComparison: string;
  paperNotCompared: string;
  limitations: string;
  noCertification: string;
  meshLimitation: string;
  damageLimitation: string;
  iterationLimitation: string;
  browserLimitation: string;
  codeReferences: string;
  implementationOutputs: string;
  adaptationCredit: string;
  referenceCode: string;
  comparisonMatches: string;
  atStep: string;
  identicalIterationsAt: string;
  stepsPeriod: string;
  details: string;
  replayMismatch: string;
  show: string;
  display: string;
  view: string;
  reactionCaption: string;
  topDisplacementAxis: string;
  referenceCurveNote: string;
  againstReferenceCode: string;
  peakForce: string;
  knAtStep: string;
  referenceSemicolon: string;
  relativeForceDifference: string;
  relativeTestTolerance: string;
  absoluteForceDifference: string;
  absoluteTestTolerance: string;
  identicalIterations: string;
  iterationsCaption: string;
  energies: string;
  energyAxis: string;
  youngModulus: string;
  youngModulusLabel: string;
  poissonRatio: string;
  poissonRatioLabel: string;
  fractureToughness: string;
  fractureToughnessLabel: string;
  lengthScale: string;
  lengthScaleLabel: string;
  topDisplacement: string;
  topDisplacementLabel: string;
  loadStep: string;
  mesh: string;
  historyField: string;
  planeStrainLoaded: string;
  refinedRatio: string;
  underResolved: string;
  stop: string;
  restart: string;
  run: string;
  solveAgain: string;
  backToReplay: string;
  solverUnavailable: string;
  solvingStep: string;
  done: string;
  steps: string;
  completedRunsKept: string;
  stoppedAfter: string;
  solverRefused: string;
  runReuseNote: string;
  runBrowserNote: string;
  loadStepShown: string;
  afterAuthors: string;
  citeThis: string;
  citationTitle: string;
  reproductionOf: string;
  implementationLabel: string;
  atCommit: string;
  engineCitation: string;
  theExperiment: string;
  experiment: string;
  viewerUnavailable: string;
  sourceLabel: Record<'crossref-abstract' | 'publisher-abstract' | 'repository-readme' | 'full-text' | 'reference-code', string>;
  resetParameters: string;
  summaryIntro: string;
  noAbstract: string;
  runIt: string;
  comingNext: string;
  inProgress: string;
  catalog: string;
  field: string;
  status: string;
  all: string;
  readRun: string;
  paper: string;
  code: string;
  openInStabileo: string;
  soon: string;
  notDesign: string;
  checksPass: string;
  notCompared: string;
  research: string;
  suggest: string;
  onlyEnglish: string;
  againstCode: string;
  againstPaper: string;
  of: string;
  seeChecks: string;
  disclaimer: string;
  notComparedWhy: string;
};

const STRINGS: Record<'en' | 'es' | 'pt', ResearchStrings> = {
  en: {
    forceComparisonTolerance: "force comparison tolerance",
    referenceDots: " (dots where the reference differs; none expected)",
    coarseMesh: "Coarse: h = 0.0025 mm",
    mediumMesh: "Medium: h = 0.00167 mm",
    referenceMesh: "Reference: 5 600 elements, h = 0.00125 mm",
    viewer: "Viewer",
    force: "Force",
    convergence: "Convergence",
    parameters: "Parameters",
    precomputedSeries: "Stabileo (Rust, precomputed replay)",
    browserSeries: "Stabileo (Rust, in this browser)",
    referenceSeries: "Reference code (Python, committed outputs)",
    referenceDifference: "difference from the reference",
    elasticLabel: "elastic",
    fractureLabel: "fracture Gc·Γ",
    elasticEnergy: "Elastic energy",
    fractureEnergy: "Fracture energy Gc·Γ",
    thisReplay: "This replay",
    thisRun: "This run",
    nonConverged: "Non-converged run",
    referenceMismatch: "Reference mismatch",
    damageField: "Damage d",
    stressAvailable: "Stress is available after a run in this browser",
    hideBroken: "Hide broken",
    hideBrokenHint: "Hide elements with two or more corners at d > 0.95 (the open crack)",
    plate: "Plate",
    crackBand: "Crack band",
    notchTip: "Notch tip",
    degradedStress: "σyy (degraded)",
    clampedColours: "colours clamped to [0, 1]",
    elementCentres: "kN/mm², element centres",
    pause: "Pause",
    play: "Play",
    equal: "equal",
    differs: "differs",
    pass: "✓ pass",
    fail: "✗ fail",
    notConvergedSuffix: " (not converged)",
    referenceLower: "reference",
    noLicence: "no licence",
    deformed: "Deformed",
    undeformed: "undeformed",
    inTime: "in",
    peakReferenceLabel: "Stabileo (blue), reference (orange halo): peak",
    peakLabel: "Stabileo: peak",
    viewerMeshLabel: "Stabileo viewer: {field} on the {shape} mesh",
    replaySource: "Replay of a run precomputed with this solver",
    liveSource: "Live computation in your browser",
    cachedSource: "Cached result (computed in this browser)",
    researchTitle: "Stabileo Research",
    galleryLead: "Computational-mechanics papers you can run. We reproduce the authors’ code, re-implement the method on Stabileo’s engine, validate it against that reference, and let you run it here with your own parameters, in Stabileo’s viewer.",
    galleryDisclaimer: "Nothing here is an engineering result; agreement with a reference code is not a certification. Contributions:",
    damageAlt: "Damage field from our run of the experiment",
    missingPage: "No such research page",
    backToResearch: "Back to Stabileo Research",
    iterationsChartLabel: "Staggered iterations per load step, log scale",
    iterations: "iterations",
    loadStepLower: "load step",
    step: "step",
    residualSuffix: "· residual",
    shauerTitle: "Phase-field fracture of a notched plate",
    shauerLead: "A square plate with a notch from its left edge to the centre is pulled apart at the top. The AT2 phase-field model represents the crack as a damage field d between 0 (intact) and 1 (broken), smeared over a length ℓ. The experiment pane runs the Stabileo Rust implementation in Stabileo’s viewer; the orange curve is what the author’s Python code computed.",
    paperShows: "What the paper shows",
    paperOverview: "The article presents a self-contained Python finite element code, under 500 lines, for brittle fracture with the phase-field method, and uses it on three examples: a bar in tension compared with the one-dimensional AT2 solution, a single-edge notched plate in tension (this page), and a three-point bending test. We read the code in full (the article’s text was not accessible to us) and everything below about the method comes from it.",
    replayGrowth: "In the replay the damage grows smoothly ahead of the notch while",
    theForce: "the force",
    risesTo: "rises linearly to",
    peakStepLink: "the peak load (step 56)",
    nextIncrement: "; at the next increment",
    crackStepLink: "the crack runs through the plate in a single step (step 57)",
    afterComma: ", after",
    alternating308: "308 alternating iterations",
    forceDrops: ", and the force drops to almost zero.",
    finalState: "The final state",
    openCrack: "shows the open crack on the deformed plate.",
    modelHeading: "The model",
    displacement: "Displacement",
    bilinearModel: "and damage d are both bilinear on quadrilaterals, integrated with 2×2 Gauss points; plane strain, small strain, quasi-static loading.",
    elasticity: "Elasticity",
    fixedDamage: "at fixed d: div(g(d)",
    wholeStress: ")) = 0 with g(d) = (1 − d)², applied to the whole stress.",
    damageAT2: "Damage (AT2)",
    atFixed: "at fixed",
    drivingEnergy: "Driving energy",
    positiveStrains: "over the positive principal strains, with σ",
    hybridModel: "the undamaged principal stresses: elasticity is degraded fully while damage is driven by tension only, a “hybrid” formulation.",
    alternatingSolution: "Alternating solution",
    solvePerStep: "per load step: solve",
    currentDamage: "with the current d, then d with the new",
    residualThreshold: ", until the elasticity residual ‖K(d)u − f‖ falls below 10",
    irreversibility: "Irreversibility",
    noIrreversibility: ": none in the reference code; damage is recomputed from the current energy and is not bounded by 1.",
    enableHistory: "Turn on the history field",
    keepOnUnloading: "to keep it on unloading (not in the reference).",
    unitsAndPeak: "Units are kN and mm. The homogeneous AT2 peak stress σ",
    lengthStrength: "/ 256 ℓ) ≈ 2.82 kN/mm² here, so ℓ behaves as a strength as much as a regularisation length:",
    tryLength: "try ℓ = 0.015 mm",
    meshMatters: "and the peak load falls. The mesh matters too: the crack band needs elements a few times smaller than ℓ;",
    tryCoarse: "try the coarse mesh",
    reproductionHeading: "How we reproduced it",
    referenceProcedure: "The author’s code (commit 61037c6, MIT licence) was run unmodified for its three examples in Python 3.12.15 with numpy 2.1.3 and scipy 1.14.1; every step’s force, iteration count, residual and damage range, and nodal fields at chosen steps, were saved with a manifest of versions and checksums.",
    rustProcedure: "The Rust implementation regenerates the same meshes (bit-identical), applies the same boundary conditions, loop and stopping rule, and solves with the Stabileo engine’s sparse Cholesky factorisation.",
    checksProcedure: "named checks compare the two, each with a stated tolerance, in continuous integration. The browser bundle and the replay on this page are checked separately against the native build.",
    referenceComparison: "Comparison with the reference",
    validationChecks: "Validation checks",
    checksCaption: "validation checks run in CI, each with its tolerance",
    check: "Check",
    observed: "Observed",
    tolerance: "Tolerance",
    result: "Result",
    symmetryFinding: "One finding: the plate is symmetric about the notch plane, and the crack runs along the row of elements just above it in one code and just below it in the other. Which row is chosen is decided by floating-point round-off; against the mirror image the final damage fields differ by 3.085970368132962 × 10",
    toleranceMeaning: ". The thresholds in the table are comparison tolerances, not mathematical error bounds. Displacement fields are compared at example 1 step 50, example 2 step 56, and example 3 step 56.",
    stepTable: "Step table (",
    stepsParen: "steps)",
    residual: "residual",
    referenceForce: "reference F, kN",
    paperComparison: "Comparison with the paper",
    paperNotCompared: "Not done: the article’s full text was not accessible to us, so no number printed in it has been compared. Figures here come from our own runs.",
    limitations: "Limitations",
    noCertification: "Agreement with a reference code is not validation against experiments, and none of this is an engineering certification.",
    meshLimitation: "The reference mesh refines only a 0.04 × 0.04 mm box around the notch tip; beyond it the crack crosses elements wider than 3ℓ.",
    damageLimitation: "No bound constraints: damage can exceed 1 slightly (the reference reaches 1.0096 here and 1.0021897551918642 in example 1) and undershoot 0. The recorded Python reference outputs have minimum nodal damage 3.076336950017797e-08 (example 1), -0.0031224948220514573 (example 2), and -0.06024191772700118 (example 3). This also occurs in the reference code’s unconstrained AT2 discretization. Negative damage is unphysical: (1-d)² at the example 3 minimum is about 1.124, above intact stiffness. Assembly uses interpolated damage at quadrature points, so this is not a 12.4% assembled-stiffness error. The solver does not clamp damage; the replay’s quantized nodal field and colour scale clamp it for display.",
    iterationLimitation: "The alternating scheme is not accelerated; at unstable crack growth it can take hundreds of iterations in one step.",
    browserLimitation: "Browser runs are limited in mesh size and parameter range.",
    codeReferences: "Code and references",
    implementationOutputs: "Implementation, tests and reference outputs:",
    adaptationCredit: "(mesh generators and loop adapted from the MIT reference, credited in the source).",
    referenceCode: "Reference code:",
    comparisonMatches: "agrees with reference forces, iterations, pre-peak maximum damage, damage at steps 56/57/78 and displacement at step 56 (replay fields quantized; stress has no reference snapshot): peak",
    atStep: "at step",
    identicalIterationsAt: ", identical iterations at",
    stepsPeriod: "steps.",
    details: "Details",
    replayMismatch: "Replay build or configuration mismatch: reference comparison disabled.",
    show: "Show",
    display: "Display",
    view: "View",
    reactionCaption: "Reaction force against imposed top displacement",
    topDisplacementAxis: "top displacement u (mm)",
    referenceCurveNote: "The reference curve is shown only for the reference parameters.",
    againstReferenceCode: "against the reference code",
    peakForce: "Peak force",
    knAtStep: "kN at step",
    referenceSemicolon: "; reference",
    relativeForceDifference: "Largest relative force difference up to the peak:",
    relativeTestTolerance: "(test tolerance 1e-9).",
    absoluteForceDifference: "Largest absolute force difference, fracture included:",
    absoluteTestTolerance: "kN (test tolerance 1e-5).",
    identicalIterations: "Identical number of staggered iterations at",
    iterationsCaption: "Staggered iterations per load step",
    energies: "Energies",
    energyAxis: "energy (kN·mm)",
    youngModulus: "Young’s modulus",
    youngModulusLabel: "Young’s modulus E, kN/mm²",
    poissonRatio: "Poisson’s ratio",
    poissonRatioLabel: "Poisson’s ratio ν",
    fractureToughness: "Fracture toughness",
    fractureToughnessLabel: "Fracture toughness Gc, kN/mm",
    lengthScale: "Length scale",
    lengthScaleLabel: "Length scale ℓ, mm",
    topDisplacement: "Top displacement at t = 1",
    topDisplacementLabel: "Top displacement u at t = 1, mm",
    loadStep: "Load step",
    mesh: "Mesh",
    historyField: "History field (irreversibility; not in the reference)",
    planeStrainLoaded: "Plane strain, loaded to t = 0.8 (",
    refinedRatio: "kN/mm². h/ℓ in the refined box =",
    underResolved: "— under-resolved (h > ℓ/2)",
    stop: "Stop",
    restart: "Restart",
    run: "Run",
    solveAgain: "Solve again in this browser",
    backToReplay: "Back to the replay",
    solverUnavailable: "The in-browser solver couldn’t be loaded in this build; the replay above is still our solver’s run.",
    solvingStep: "Solving step",
    done: "Done:",
    steps: "steps",
    completedRunsKept: ". Completed runs are kept, so earlier parameters replay instantly.",
    stoppedAfter: "Stopped after",
    solverRefused: "The solver refused the input:",
    runReuseNote: "Run reuses a result already computed for these parameters, if there is one; Solve again recomputes it here (about 15–30 s).",
    runBrowserNote: "Run computes these parameters in your browser.",
    loadStepShown: "Load step shown",
    afterAuthors: "After",
    citeThis: "Cite this",
    citationTitle: "Stabileo Research, “",
    reproductionOf: "”, reproduction of",
    implementationLabel: ". Implementation:",
    atCommit: "at commit",
    engineCitation: ". Engine: the Stabileo engine at the same commit. Permanent links to this exact build are planned.",
    theExperiment: "The experiment",
    experiment: "Experiment",
    viewerUnavailable: "Stabileo’s viewer couldn’t be loaded in this build.",
    resetParameters: 'Reset to the reference-code parameters',
    sourceLabel: { 'crossref-abstract': 'from the abstract', 'publisher-abstract': 'from the abstract', 'repository-readme': 'from the code’s README', 'full-text': 'from the paper', 'reference-code': 'from the reference code' },
    summaryIntro: 'Summary in our words,',
    noAbstract: 'Abstract not available.',
    runIt: 'Run it',
    comingNext: 'Coming next',
    inProgress: 'in progress',
    catalog: 'On our list: papers we plan to reproduce',
    field: 'Field',
    status: 'Status',
    all: 'All',
    readRun: 'Read & run',
    paper: 'Paper',
    code: 'Code',
    openInStabileo: 'Open in Stabileo',
    soon: 'coming with research mode',
    notDesign: 'Research, not a design tool',
    checksPass: 'checks pass',
    notCompared: 'Published figures not compared',
    research: 'Research',
    suggest: 'Suggest a paper to reproduce →',
    onlyEnglish: '',
    againstCode: 'Against the author’s code',
    againstPaper: 'Against the paper’s figures',
    of: 'of',
    seeChecks: 'see the checks',
    disclaimer: 'Research, not a design tool. Nothing here is an engineering result.',
    notComparedWhy: 'Not compared: the full text was not accessible to us',
  },
  es: {
    forceComparisonTolerance: "tolerancia de comparación de fuerzas",
    referenceDots: " (puntos donde la referencia difiere; no se espera ninguno)",
    coarseMesh: "Gruesa: h = 0.0025 mm",
    mediumMesh: "Media: h = 0.00167 mm",
    referenceMesh: "Referencia: 5 600 elementos, h = 0.00125 mm",
    viewer: "Visor",
    force: "Fuerza",
    convergence: "Convergencia",
    parameters: "Parámetros",
    precomputedSeries: "Stabileo (Rust, reproducción precalculada)",
    browserSeries: "Stabileo (Rust, en este navegador)",
    referenceSeries: "Código de referencia (Python, resultados versionados)",
    referenceDifference: "diferencia respecto a la referencia",
    elasticLabel: "elástica",
    fractureLabel: "fractura Gc·Γ",
    elasticEnergy: "Energía elástica",
    fractureEnergy: "Energía de fractura Gc·Γ",
    thisReplay: "Esta reproducción",
    thisRun: "Esta ejecución",
    nonConverged: "Ejecución sin convergencia",
    referenceMismatch: "Discrepancia con la referencia",
    damageField: "Daño d",
    stressAvailable: "La tensión está disponible después de una ejecución en este navegador",
    hideBroken: "Ocultar rotos",
    hideBrokenHint: "Ocultar elementos con dos o más esquinas con d > 0.95 (la grieta abierta)",
    plate: "Placa",
    crackBand: "Banda de la grieta",
    notchTip: "Punta de la entalla",
    degradedStress: "σyy (degradada)",
    clampedColours: "colores limitados a [0, 1]",
    elementCentres: "kN/mm², centros de elementos",
    pause: "Pausar",
    play: "Reproducir",
    equal: "igual",
    differs: "difiere",
    pass: "✓ aprobado",
    fail: "✗ fallido",
    notConvergedSuffix: " (sin convergencia)",
    referenceLower: "referencia",
    noLicence: "sin licencia",
    deformed: "Deformada",
    undeformed: "sin deformar",
    inTime: "en",
    peakReferenceLabel: "Stabileo (azul), referencia (halo naranja): pico",
    peakLabel: "Stabileo: pico",
    viewerMeshLabel: "Visor de Stabileo: {field} en la malla {shape}",
    replaySource: "Reproducción de una ejecución precalculada con este solver",
    liveSource: "Cálculo en vivo en tu navegador",
    cachedSource: "Resultado en caché (calculado en este navegador)",
    researchTitle: "Stabileo Research",
    galleryLead: "Artículos de mecánica computacional que podés ejecutar. Reproducimos el código de los autores, reimplementamos el método en el motor de Stabileo, lo validamos frente a esa referencia y te permitimos ejecutarlo aquí con tus propios parámetros, en el visor de Stabileo.",
    galleryDisclaimer: "Nada de esto es un resultado de ingeniería; la concordancia con un código de referencia no es una certificación. Contribuciones:",
    damageAlt: "Campo de daño de nuestra ejecución del experimento",
    missingPage: "No existe esa página de investigación",
    backToResearch: "Volver a Stabileo Research",
    iterationsChartLabel: "Iteraciones alternadas por paso de carga, escala logarítmica",
    iterations: "iteraciones",
    loadStepLower: "paso de carga",
    step: "paso",
    residualSuffix: "· residuo",
    shauerTitle: "Fractura por campo de fase de una placa entallada",
    shauerLead: "Una placa cuadrada con una entalla desde su borde izquierdo hasta el centro se tracciona desde arriba. El modelo de campo de fase AT2 representa la grieta como un campo de daño d entre 0 (intacto) y 1 (roto), distribuido sobre una longitud ℓ. El panel del experimento ejecuta la implementación Rust de Stabileo en su visor; la curva naranja es lo que calculó el código Python del autor.",
    paperShows: "Qué muestra el artículo",
    paperOverview: "El artículo presenta un código autónomo de elementos finitos en Python, de menos de 500 líneas, para la fractura frágil con el método de campo de fase, y lo aplica a tres ejemplos: una barra a tracción comparada con la solución unidimensional AT2, una placa con entalla en un borde sometida a tracción (esta página) y un ensayo de flexión en tres puntos. Leímos todo el código (no tuvimos acceso al texto del artículo) y todo lo que sigue sobre el método proviene de él.",
    replayGrowth: "En la reproducción el daño crece suavemente por delante de la entalla mientras",
    theForce: "la fuerza",
    risesTo: "crece linealmente hasta",
    peakStepLink: "la carga máxima (paso 56)",
    nextIncrement: "; en el siguiente incremento",
    crackStepLink: "la grieta atraviesa la placa en un solo paso (paso 57)",
    afterComma: ", después de",
    alternating308: "308 iteraciones alternadas",
    forceDrops: ", y la fuerza cae casi a cero.",
    finalState: "El estado final",
    openCrack: "muestra la grieta abierta en la placa deformada.",
    modelHeading: "El modelo",
    displacement: "Desplazamiento",
    bilinearModel: "y el daño d son ambos bilineales en cuadriláteros, integrados con 2×2 puntos de Gauss; deformación plana, pequeñas deformaciones, carga cuasiestática.",
    elasticity: "Elasticidad",
    fixedDamage: "con d fijo: div(g(d)",
    wholeStress: ")) = 0 con g(d) = (1 − d)², aplicado a toda la tensión.",
    damageAT2: "Daño (AT2)",
    atFixed: "con valor fijo de",
    drivingEnergy: "Energía impulsora",
    positiveStrains: "sobre las deformaciones principales positivas, con σ",
    hybridModel: "las tensiones principales sin daño: la elasticidad se degrada por completo mientras que el daño es impulsado solo por la tracción, una formulación «híbrida».",
    alternatingSolution: "Solución alternada",
    solvePerStep: "por paso de carga: resolver",
    currentDamage: "con el d actual, luego d con el nuevo",
    residualThreshold: ", hasta que el residuo de elasticidad ‖K(d)u − f‖ sea menor que 10",
    irreversibility: "Irreversibilidad",
    noIrreversibility: ": ninguna en el código de referencia; el daño se recalcula a partir de la energía actual y no está acotado por 1.",
    enableHistory: "Activá el campo de historia",
    keepOnUnloading: "para conservarlo durante la descarga (no está en la referencia).",
    unitsAndPeak: "Las unidades son kN y mm. La tensión máxima homogénea AT2 σ",
    lengthStrength: "/ 256 ℓ) ≈ 2.82 kN/mm² aquí, por lo que ℓ actúa tanto como una resistencia como una longitud de regularización:",
    tryLength: "probá ℓ = 0.015 mm",
    meshMatters: "y la carga máxima baja. La malla también importa: la banda de la grieta necesita elementos varias veces más pequeños que ℓ;",
    tryCoarse: "probá la malla gruesa",
    reproductionHeading: "Cómo lo reprodujimos",
    referenceProcedure: "El código del autor (commit 61037c6, licencia MIT) se ejecutó sin modificaciones para sus tres ejemplos en Python 3.12.15 con numpy 2.1.3 y scipy 1.14.1; la fuerza, el número de iteraciones, el residuo y el rango de daño de cada paso, y los campos nodales en pasos seleccionados, se guardaron con un manifiesto de versiones y sumas de comprobación.",
    rustProcedure: "La implementación Rust regenera las mismas mallas (idénticas bit a bit), aplica las mismas condiciones de contorno, bucle y criterio de parada, y resuelve con la factorización de Cholesky dispersa del motor de Stabileo.",
    checksProcedure: "controles con nombre comparan ambas implementaciones, cada uno con una tolerancia declarada, en integración continua. El paquete del navegador y la reproducción de esta página se verifican por separado frente a la compilación nativa.",
    referenceComparison: "Comparación con la referencia",
    validationChecks: "Controles de validación",
    checksCaption: "controles de validación ejecutados en CI, cada uno con su tolerancia",
    check: "Control",
    observed: "Observado",
    tolerance: "Tolerancia",
    result: "Resultado",
    symmetryFinding: "Un hallazgo: la placa es simétrica respecto al plano de la entalla, y la grieta avanza por la fila de elementos justo encima en un código y justo debajo en el otro. La fila elegida depende del redondeo en coma flotante; frente a la imagen especular los campos de daño finales difieren en 3.085970368132962 × 10",
    toleranceMeaning: ". Los umbrales de la tabla son tolerancias de comparación, no cotas matemáticas del error. Los campos de desplazamiento se comparan en el ejemplo 1 paso 50, el ejemplo 2 paso 56 y el ejemplo 3 paso 56.",
    stepTable: "Tabla de pasos (",
    stepsParen: "pasos)",
    residual: "residuo",
    referenceForce: "F de referencia, kN",
    paperComparison: "Comparación con el artículo",
    paperNotCompared: "No realizada: no tuvimos acceso al texto completo del artículo, por lo que no se comparó ningún número publicado en él. Las figuras aquí provienen de nuestras propias ejecuciones.",
    limitations: "Limitaciones",
    noCertification: "La concordancia con un código de referencia no es una validación frente a experimentos, y nada de esto es una certificación de ingeniería.",
    meshLimitation: "La malla de referencia refina solo una caja de 0.04 × 0.04 mm alrededor de la punta de la entalla; fuera de ella la grieta cruza elementos más anchos que 3ℓ.",
    damageLimitation: "Sin restricciones de cota: el daño puede superar ligeramente 1 (la referencia alcanza 1.0096 aquí y 1.0021897551918642 en el ejemplo 1) y caer por debajo de 0. Los resultados de referencia Python registrados tienen daños nodales mínimos de 3.076336950017797e-08 (ejemplo 1), -0.0031224948220514573 (ejemplo 2) y -0.06024191772700118 (ejemplo 3). Esto también ocurre en la discretización AT2 sin restricciones del código de referencia. El daño negativo no es físico: (1-d)² en el mínimo del ejemplo 3 es aproximadamente 1.124, por encima de la rigidez intacta. El ensamblaje usa daño interpolado en puntos de cuadratura, así que esto no es un error de 12.4% en la rigidez ensamblada. El solver no recorta el daño; el campo nodal cuantizado de la reproducción y la escala de colores lo recortan para su visualización.",
    iterationLimitation: "El esquema alternado no está acelerado; durante el crecimiento inestable de la grieta puede requerir cientos de iteraciones en un paso.",
    browserLimitation: "Las ejecuciones en el navegador están limitadas en tamaño de malla y rango de parámetros.",
    codeReferences: "Código y referencias",
    implementationOutputs: "Implementación, pruebas y resultados de referencia:",
    adaptationCredit: "(generadores de malla y bucle adaptados de la referencia MIT, con atribución en el código fuente).",
    referenceCode: "Código de referencia:",
    comparisonMatches: "concuerda con las fuerzas, iteraciones, daño máximo anterior al pico, daño en los pasos 56/57/78 y desplazamiento en el paso 56 de la referencia (campos de reproducción cuantizados; la tensión no tiene una instantánea de referencia): pico",
    atStep: "en el paso",
    identicalIterationsAt: ", iteraciones idénticas en",
    stepsPeriod: "pasos.",
    details: "Detalles",
    replayMismatch: "La compilación o configuración de la reproducción no coincide: comparación con la referencia desactivada.",
    show: "Mostrar",
    display: "Visualización",
    view: "Vista",
    reactionCaption: "Fuerza de reacción frente al desplazamiento superior impuesto",
    topDisplacementAxis: "desplazamiento superior u (mm)",
    referenceCurveNote: "La curva de referencia se muestra solo para los parámetros de referencia.",
    againstReferenceCode: "frente al código de referencia",
    peakForce: "Fuerza máxima",
    knAtStep: "kN en el paso",
    referenceSemicolon: "; referencia",
    relativeForceDifference: "Mayor diferencia relativa de fuerza hasta el pico:",
    relativeTestTolerance: "(tolerancia de prueba 1e-9).",
    absoluteForceDifference: "Mayor diferencia absoluta de fuerza, incluida la fractura:",
    absoluteTestTolerance: "kN (tolerancia de prueba 1e-5).",
    identicalIterations: "Número idéntico de iteraciones alternadas en",
    iterationsCaption: "Iteraciones alternadas por paso de carga",
    energies: "Energías",
    energyAxis: "energía (kN·mm)",
    youngModulus: "Módulo de Young",
    youngModulusLabel: "Módulo de Young E, kN/mm²",
    poissonRatio: "Coeficiente de Poisson",
    poissonRatioLabel: "Coeficiente de Poisson ν",
    fractureToughness: "Tenacidad a la fractura",
    fractureToughnessLabel: "Tenacidad a la fractura Gc, kN/mm",
    lengthScale: "Longitud característica",
    lengthScaleLabel: "Longitud característica ℓ, mm",
    topDisplacement: "Desplazamiento superior en t = 1",
    topDisplacementLabel: "Desplazamiento superior u en t = 1, mm",
    loadStep: "Paso de carga",
    mesh: "Malla",
    historyField: "Campo de historia (irreversibilidad; no está en la referencia)",
    planeStrainLoaded: "Deformación plana, cargada hasta t = 0.8 (",
    refinedRatio: "kN/mm². h/ℓ en la caja refinada =",
    underResolved: "— resolución insuficiente (h > ℓ/2)",
    stop: "Detener",
    restart: "Reiniciar",
    run: "Ejecutar",
    solveAgain: "Resolver de nuevo en este navegador",
    backToReplay: "Volver a la reproducción",
    solverUnavailable: "No se pudo cargar el solver del navegador en esta compilación; la reproducción de arriba sigue siendo una ejecución de nuestro solver.",
    solvingStep: "Resolviendo el paso",
    done: "Completado:",
    steps: "pasos",
    completedRunsKept: ". Las ejecuciones completas se guardan, por lo que los parámetros anteriores se reproducen al instante.",
    stoppedAfter: "Detenido después de",
    solverRefused: "El solver rechazó la entrada:",
    runReuseNote: "Ejecutar reutiliza un resultado ya calculado para estos parámetros, si existe; Resolver de nuevo lo recalcula aquí (unos 15–30 s).",
    runBrowserNote: "Ejecutar calcula estos parámetros en tu navegador.",
    loadStepShown: "Paso de carga mostrado",
    afterAuthors: "A partir de",
    citeThis: "Citar esto",
    citationTitle: "Stabileo Research, “",
    reproductionOf: "”, reproducción de",
    implementationLabel: ". Implementación:",
    atCommit: "en el commit",
    engineCitation: ". Motor: el motor de Stabileo en el mismo commit. Se prevén enlaces permanentes a esta compilación exacta.",
    theExperiment: "El experimento",
    experiment: "Experimento",
    viewerUnavailable: "No se pudo cargar el visor de Stabileo en esta compilación.",
    resetParameters: 'Restablecer los parámetros del código de referencia',
    sourceLabel: { 'crossref-abstract': 'a partir del resumen', 'publisher-abstract': 'a partir del resumen', 'repository-readme': 'a partir del README del código', 'full-text': 'a partir del artículo', 'reference-code': 'a partir del código de referencia' },
    summaryIntro: 'Resumen en nuestras palabras,',
    noAbstract: 'Resumen no disponible.',
    runIt: 'Ejecutalo',
    comingNext: 'Próximamente',
    inProgress: 'en curso',
    catalog: 'En nuestra lista: artículos que planeamos reproducir',
    field: 'Área',
    status: 'Estado',
    all: 'Todos',
    readRun: 'Leer y ejecutar',
    paper: 'Artículo',
    code: 'Código',
    openInStabileo: 'Abrir en Stabileo',
    soon: 'llega con el modo investigación',
    notDesign: 'Investigación, no una herramienta de diseño',
    checksPass: 'controles aprobados',
    notCompared: 'Sin comparar con las figuras publicadas',
    research: 'Investigación',
    suggest: 'Sugerí un artículo para reproducir →',
    onlyEnglish: 'Este artículo está disponible solo en inglés.',
    againstCode: 'Frente al código del autor',
    againstPaper: 'Frente a las figuras del artículo',
    of: 'de',
    seeChecks: 'ver los controles',
    disclaimer: 'Investigación, no una herramienta de diseño. Nada de esto es un resultado de ingeniería.',
    notComparedWhy: 'No comparado: no tuvimos acceso al texto completo',
  },
  pt: {
    forceComparisonTolerance: "tolerância de comparação de forças",
    referenceDots: " (pontos onde a referência difere; nenhum esperado)",
    coarseMesh: "Grossa: h = 0.0025 mm",
    mediumMesh: "Média: h = 0.00167 mm",
    referenceMesh: "Referência: 5 600 elementos, h = 0.00125 mm",
    viewer: "Visualizador",
    force: "Força",
    convergence: "Convergência",
    parameters: "Parâmetros",
    precomputedSeries: "Stabileo (Rust, reprodução pré-calculada)",
    browserSeries: "Stabileo (Rust, neste navegador)",
    referenceSeries: "Código de referência (Python, resultados versionados)",
    referenceDifference: "diferença em relação à referência",
    elasticLabel: "elástica",
    fractureLabel: "fratura Gc·Γ",
    elasticEnergy: "Energia elástica",
    fractureEnergy: "Energia de fratura Gc·Γ",
    thisReplay: "Esta reprodução",
    thisRun: "Esta execução",
    nonConverged: "Execução sem convergência",
    referenceMismatch: "Discrepância com a referência",
    damageField: "Dano d",
    stressAvailable: "A tensão fica disponível após uma execução neste navegador",
    hideBroken: "Ocultar rompidos",
    hideBrokenHint: "Ocultar elementos com dois ou mais cantos com d > 0.95 (a fissura aberta)",
    plate: "Placa",
    crackBand: "Faixa da fissura",
    notchTip: "Ponta do entalhe",
    degradedStress: "σyy (degradada)",
    clampedColours: "cores limitadas a [0, 1]",
    elementCentres: "kN/mm², centros dos elementos",
    pause: "Pausar",
    play: "Reproduzir",
    equal: "igual",
    differs: "difere",
    pass: "✓ aprovado",
    fail: "✗ falhou",
    notConvergedSuffix: " (sem convergência)",
    referenceLower: "referência",
    noLicence: "sem licença",
    deformed: "Deformada",
    undeformed: "sem deformação",
    inTime: "em",
    peakReferenceLabel: "Stabileo (azul), referência (halo laranja): pico",
    peakLabel: "Stabileo: pico",
    viewerMeshLabel: "Visualizador do Stabileo: {field} na malha {shape}",
    replaySource: "Reprodução de uma execução pré-calculada com este solver",
    liveSource: "Cálculo ao vivo no seu navegador",
    cachedSource: "Resultado em cache (calculado neste navegador)",
    researchTitle: "Stabileo Research",
    galleryLead: "Artigos de mecânica computacional que você pode executar. Reproduzimos o código dos autores, reimplementamos o método no motor do Stabileo, validamos em relação a essa referência e permitimos que você o execute aqui com seus próprios parâmetros, no visualizador do Stabileo.",
    galleryDisclaimer: "Nada aqui é um resultado de engenharia; a concordância com um código de referência não é uma certificação. Contribuições:",
    damageAlt: "Campo de dano da nossa execução do experimento",
    missingPage: "Essa página de pesquisa não existe",
    backToResearch: "Voltar ao Stabileo Research",
    iterationsChartLabel: "Iterações alternadas por passo de carga, escala logarítmica",
    iterations: "iterações",
    loadStepLower: "passo de carga",
    step: "passo",
    residualSuffix: "· resíduo",
    shauerTitle: "Fratura por campo de fase de uma placa entalhada",
    shauerLead: "Uma placa quadrada com um entalhe da borda esquerda até o centro é tracionada pelo topo. O modelo de campo de fase AT2 representa a fissura como um campo de dano d entre 0 (íntegro) e 1 (rompido), distribuído ao longo de um comprimento ℓ. O painel do experimento executa a implementação Rust do Stabileo em seu visualizador; a curva laranja é o que o código Python do autor calculou.",
    paperShows: "O que o artigo mostra",
    paperOverview: "O artigo apresenta um código autônomo de elementos finitos em Python, com menos de 500 linhas, para fratura frágil pelo método de campo de fase, e o aplica a três exemplos: uma barra sob tração comparada com a solução unidimensional AT2, uma placa com entalhe em uma borda sob tração (esta página) e um ensaio de flexão em três pontos. Lemos todo o código (não tivemos acesso ao texto do artigo) e tudo abaixo sobre o método vem dele.",
    replayGrowth: "Na reprodução o dano cresce suavemente à frente do entalhe enquanto",
    theForce: "a força",
    risesTo: "cresce linearmente até",
    peakStepLink: "a carga máxima (passo 56)",
    nextIncrement: "; no incremento seguinte",
    crackStepLink: "a fissura atravessa a placa em um único passo (passo 57)",
    afterComma: ", após",
    alternating308: "308 iterações alternadas",
    forceDrops: ", e a força cai para quase zero.",
    finalState: "O estado final",
    openCrack: "mostra a fissura aberta na placa deformada.",
    modelHeading: "O modelo",
    displacement: "Deslocamento",
    bilinearModel: "e o dano d são ambos bilineares em quadriláteros, integrados com 2×2 pontos de Gauss; estado plano de deformação, pequenas deformações, carregamento quase estático.",
    elasticity: "Elasticidade",
    fixedDamage: "com d fixo: div(g(d)",
    wholeStress: ")) = 0 com g(d) = (1 − d)², aplicado a toda a tensão.",
    damageAT2: "Dano (AT2)",
    atFixed: "com valor fixo de",
    drivingEnergy: "Energia motriz",
    positiveStrains: "sobre as deformações principais positivas, com σ",
    hybridModel: "as tensões principais sem dano: a elasticidade é degradada integralmente enquanto o dano é impulsionado apenas pela tração, uma formulação “híbrida”.",
    alternatingSolution: "Solução alternada",
    solvePerStep: "por passo de carga: resolver",
    currentDamage: "com o d atual, depois d com o novo",
    residualThreshold: ", até que o resíduo de elasticidade ‖K(d)u − f‖ fique abaixo de 10",
    irreversibility: "Irreversibilidade",
    noIrreversibility: ": nenhuma no código de referência; o dano é recalculado a partir da energia atual e não é limitado por 1.",
    enableHistory: "Ative o campo de histórico",
    keepOnUnloading: "para mantê-lo durante o descarregamento (não está na referência).",
    unitsAndPeak: "As unidades são kN e mm. A tensão máxima homogênea AT2 σ",
    lengthStrength: "/ 256 ℓ) ≈ 2.82 kN/mm² aqui, portanto ℓ atua tanto como uma resistência quanto como um comprimento de regularização:",
    tryLength: "experimente ℓ = 0.015 mm",
    meshMatters: "e a carga máxima diminui. A malha também importa: a faixa da fissura precisa de elementos algumas vezes menores que ℓ;",
    tryCoarse: "experimente a malha grossa",
    reproductionHeading: "Como reproduzimos",
    referenceProcedure: "O código do autor (commit 61037c6, licença MIT) foi executado sem modificações para seus três exemplos em Python 3.12.15 com numpy 2.1.3 e scipy 1.14.1; a força, a contagem de iterações, o resíduo e a faixa de dano de cada passo, e os campos nodais em passos selecionados, foram salvos com um manifesto de versões e somas de verificação.",
    rustProcedure: "A implementação Rust regenera as mesmas malhas (idênticas bit a bit), aplica as mesmas condições de contorno, laço e critério de parada, e resolve com a fatoração de Cholesky esparsa do motor do Stabileo.",
    checksProcedure: "verificações nomeadas comparam as duas implementações, cada uma com uma tolerância declarada, na integração contínua. O pacote do navegador e a reprodução desta página são verificados separadamente em relação à compilação nativa.",
    referenceComparison: "Comparação com a referência",
    validationChecks: "Verificações de validação",
    checksCaption: "verificações de validação executadas em CI, cada uma com sua tolerância",
    check: "Verificação",
    observed: "Observado",
    tolerance: "Tolerância",
    result: "Resultado",
    symmetryFinding: "Uma constatação: a placa é simétrica em relação ao plano do entalhe, e a fissura percorre a fileira de elementos logo acima em um código e logo abaixo no outro. A fileira escolhida depende do arredondamento em ponto flutuante; em relação à imagem espelhada os campos de dano finais diferem em 3.085970368132962 × 10",
    toleranceMeaning: ". Os limiares da tabela são tolerâncias de comparação, não limites matemáticos do erro. Os campos de deslocamento são comparados no exemplo 1 passo 50, exemplo 2 passo 56 e exemplo 3 passo 56.",
    stepTable: "Tabela de passos (",
    stepsParen: "passos)",
    residual: "resíduo",
    referenceForce: "F de referência, kN",
    paperComparison: "Comparação com o artigo",
    paperNotCompared: "Não realizada: não tivemos acesso ao texto completo do artigo, portanto nenhum número publicado nele foi comparado. As figuras aqui vêm das nossas próprias execuções.",
    limitations: "Limitações",
    noCertification: "A concordância com um código de referência não é uma validação em relação a experimentos, e nada disso é uma certificação de engenharia.",
    meshLimitation: "A malha de referência refina apenas uma caixa de 0.04 × 0.04 mm ao redor da ponta do entalhe; fora dela a fissura atravessa elementos mais largos que 3ℓ.",
    damageLimitation: "Sem restrições de limite: o dano pode exceder ligeiramente 1 (a referência atinge 1.0096 aqui e 1.0021897551918642 no exemplo 1) e ficar abaixo de 0. Os resultados de referência Python registrados têm danos nodais mínimos de 3.076336950017797e-08 (exemplo 1), -0.0031224948220514573 (exemplo 2) e -0.06024191772700118 (exemplo 3). Isso também ocorre na discretização AT2 sem restrições do código de referência. Dano negativo não é físico: (1-d)² no mínimo do exemplo 3 é aproximadamente 1.124, acima da rigidez íntegra. A montagem usa dano interpolado nos pontos de quadratura, portanto isso não é um erro de 12.4% na rigidez montada. O solver não limita o dano; o campo nodal quantizado da reprodução e a escala de cores o limitam para exibição.",
    iterationLimitation: "O esquema alternado não é acelerado; durante o crescimento instável da fissura pode exigir centenas de iterações em um passo.",
    browserLimitation: "As execuções no navegador são limitadas em tamanho de malha e faixa de parâmetros.",
    codeReferences: "Código e referências",
    implementationOutputs: "Implementação, testes e resultados de referência:",
    adaptationCredit: "(geradores de malha e laço adaptados da referência MIT, com atribuição no código-fonte).",
    referenceCode: "Código de referência:",
    comparisonMatches: "concorda com as forças, iterações, dano máximo anterior ao pico, dano nos passos 56/57/78 e deslocamento no passo 56 da referência (campos da reprodução quantizados; a tensão não tem instantâneo de referência): pico",
    atStep: "no passo",
    identicalIterationsAt: ", iterações idênticas em",
    stepsPeriod: "passos.",
    details: "Detalhes",
    replayMismatch: "A compilação ou configuração da reprodução não corresponde: comparação com a referência desativada.",
    show: "Mostrar",
    display: "Exibição",
    view: "Vista",
    reactionCaption: "Força de reação em função do deslocamento superior imposto",
    topDisplacementAxis: "deslocamento superior u (mm)",
    referenceCurveNote: "A curva de referência é mostrada apenas para os parâmetros de referência.",
    againstReferenceCode: "em relação ao código de referência",
    peakForce: "Força máxima",
    knAtStep: "kN no passo",
    referenceSemicolon: "; referência",
    relativeForceDifference: "Maior diferença relativa de força até o pico:",
    relativeTestTolerance: "(tolerância do teste 1e-9).",
    absoluteForceDifference: "Maior diferença absoluta de força, incluindo a fratura:",
    absoluteTestTolerance: "kN (tolerância do teste 1e-5).",
    identicalIterations: "Número idêntico de iterações alternadas em",
    iterationsCaption: "Iterações alternadas por passo de carga",
    energies: "Energias",
    energyAxis: "energia (kN·mm)",
    youngModulus: "Módulo de Young",
    youngModulusLabel: "Módulo de Young E, kN/mm²",
    poissonRatio: "Coeficiente de Poisson",
    poissonRatioLabel: "Coeficiente de Poisson ν",
    fractureToughness: "Tenacidade à fratura",
    fractureToughnessLabel: "Tenacidade à fratura Gc, kN/mm",
    lengthScale: "Comprimento característico",
    lengthScaleLabel: "Comprimento característico ℓ, mm",
    topDisplacement: "Deslocamento superior em t = 1",
    topDisplacementLabel: "Deslocamento superior u em t = 1, mm",
    loadStep: "Passo de carga",
    mesh: "Malha",
    historyField: "Campo de histórico (irreversibilidade; não está na referência)",
    planeStrainLoaded: "Estado plano de deformação, carregado até t = 0.8 (",
    refinedRatio: "kN/mm². h/ℓ na caixa refinada =",
    underResolved: "— resolução insuficiente (h > ℓ/2)",
    stop: "Parar",
    restart: "Reiniciar",
    run: "Executar",
    solveAgain: "Resolver novamente neste navegador",
    backToReplay: "Voltar à reprodução",
    solverUnavailable: "Não foi possível carregar o solver do navegador nesta compilação; a reprodução acima continua sendo uma execução do nosso solver.",
    solvingStep: "Resolvendo o passo",
    done: "Concluído:",
    steps: "passos",
    completedRunsKept: ". As execuções concluídas são mantidas, então os parâmetros anteriores são reproduzidos instantaneamente.",
    stoppedAfter: "Parado após",
    solverRefused: "O solver rejeitou a entrada:",
    runReuseNote: "Executar reutiliza um resultado já calculado para estes parâmetros, se houver; Resolver novamente o recalcula aqui (cerca de 15–30 s).",
    runBrowserNote: "Executar calcula estes parâmetros no seu navegador.",
    loadStepShown: "Passo de carga exibido",
    afterAuthors: "A partir de",
    citeThis: "Citar isto",
    citationTitle: "Stabileo Research, “",
    reproductionOf: "”, reprodução de",
    implementationLabel: ". Implementação:",
    atCommit: "no commit",
    engineCitation: ". Motor: o motor do Stabileo no mesmo commit. Estão previstos links permanentes para esta compilação exata.",
    theExperiment: "O experimento",
    experiment: "Experimento",
    viewerUnavailable: "Não foi possível carregar o visualizador do Stabileo nesta compilação.",
    resetParameters: 'Restaurar os parâmetros do código de referência',
    sourceLabel: { 'crossref-abstract': 'a partir do resumo', 'publisher-abstract': 'a partir do resumo', 'repository-readme': 'a partir do README do código', 'full-text': 'a partir do artigo', 'reference-code': 'a partir do código de referência' },
    summaryIntro: 'Resumo em nossas palavras,',
    noAbstract: 'Resumo não disponível.',
    runIt: 'Execute',
    comingNext: 'Em breve',
    inProgress: 'em andamento',
    catalog: 'Na nossa lista: artigos que planejamos reproduzir',
    field: 'Área',
    status: 'Status',
    all: 'Todos',
    readRun: 'Ler e executar',
    paper: 'Artigo',
    code: 'Código',
    openInStabileo: 'Abrir no Stabileo',
    soon: 'chega com o modo pesquisa',
    notDesign: 'Pesquisa, não uma ferramenta de projeto',
    checksPass: 'verificações aprovadas',
    notCompared: 'Sem comparar com as figuras publicadas',
    research: 'Pesquisa',
    suggest: 'Sugira um artigo para reproduzir →',
    onlyEnglish: 'Este artigo está disponível apenas em inglês.',
    againstCode: 'Frente ao código do autor',
    againstPaper: 'Frente às figuras do artigo',
    of: 'de',
    seeChecks: 'ver as verificações',
    disclaimer: 'Pesquisa, não uma ferramenta de projeto. Nada aqui é um resultado de engenharia.',
    notComparedWhy: 'Não comparado: não tivemos acesso ao texto completo',
  },
};

export function rs(): ResearchStrings {
  return STRINGS[publicI18n.locale as keyof typeof STRINGS] ?? STRINGS.en;
}

export const FIELD_NAMES: Record<string, Record<string, string>> = {
  en: { fracture: 'Fracture and damage', 'differentiable-fem': 'Differentiable FEM', optimization: 'Optimization', metamaterials: 'Metamaterials', constitutive: 'Constitutive models', 'digital-twins': 'Digital twins', 'numerical-methods': 'Numerical methods', seismic: 'Seismic and dynamics', shells: 'Shells and buckling' },
  es: { fracture: 'Fractura y daño', 'differentiable-fem': 'FEM diferenciable', optimization: 'Optimización', metamaterials: 'Metamateriales', constitutive: 'Modelos constitutivos', 'digital-twins': 'Gemelos digitales', 'numerical-methods': 'Métodos numéricos', seismic: 'Sísmica y dinámica', shells: 'Cáscaras y pandeo' },
  pt: { fracture: 'Fratura e dano', 'differentiable-fem': 'MEF diferenciável', optimization: 'Otimização', metamaterials: 'Metamateriais', constitutive: 'Modelos constitutivos', 'digital-twins': 'Gêmeos digitais', 'numerical-methods': 'Métodos numéricos', seismic: 'Sísmica e dinâmica', shells: 'Cascas e flambagem' },
};

export const STATUS_NAMES: Record<string, Record<string, string>> = {
  en: { proposed: 'Proposed', 'reference-reproduced': 'Reference reproduced', 'implementation-in-progress': 'Implementation in progress', 'numerically-validated': 'Numerically validated', 'published-figures-compared': 'Published figures compared', interactive: 'Interactive' },
  es: { proposed: 'Propuesto', 'reference-reproduced': 'Referencia reproducida', 'implementation-in-progress': 'Implementación en curso', 'numerically-validated': 'Validado numéricamente', 'published-figures-compared': 'Comparado con las figuras publicadas', interactive: 'Interactivo' },
  pt: { proposed: 'Proposto', 'reference-reproduced': 'Referência reproduzida', 'implementation-in-progress': 'Implementação em andamento', 'numerically-validated': 'Validado numericamente', 'published-figures-compared': 'Comparado com as figuras publicadas', interactive: 'Interativo' },
};

export const fieldName = (f: string) => FIELD_NAMES[publicI18n.locale]?.[f] ?? FIELD_NAMES.en[f] ?? f;
export const statusName = (s: string) => STATUS_NAMES[publicI18n.locale]?.[s] ?? STATUS_NAMES.en[s] ?? s;

/** The language actually used by the research strings, including fallback. */
export const researchLocale = () => ['es', 'pt'].includes(publicI18n.locale) ? publicI18n.locale : 'en';
