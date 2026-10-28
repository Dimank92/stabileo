<script lang="ts">
  /**
   * The 3D viewport's look and navigation (shared scene setup) showing one scalar field on a 2D
   * mesh, with the contour legend. For views that are not the editor's model: research
   * experiments, and anything else that has a mesh and a field but no Stabileo model.
   *
   * Navigation as in the app: left drag orbits, right drag pans, the wheel zooms to the pointer.
   * The camera starts looking straight down at the XY plane, fitted to `view` (or the mesh).
   */
  import { onDestroy, onMount } from 'svelte';
  import * as THREE from 'three';
  import type { OrbitControls } from 'three/addons/controls/OrbitControls.js';
  import {
    addViewerLights,
    applyDefaultNavigation,
    createViewerControls,
    createViewerPerspectiveCamera,
    createViewerRenderer,
    createViewerScene,
  } from '../../lib/viewport3d/scene-setup';
  import { createFrameCoalescer } from '../../lib/viewport3d/frame-coalesce';
  import { createScalarFieldMesh, type ScalarFieldMesh } from '../../lib/three/scalar-field-mesh';
  import { TOP_VIEW_UP_VECTOR } from '../../lib/geometry/coordinate-system';
  import ContourLegend from './ContourLegend.svelte';

  type Props = {
    /** Node coordinates [x0, y0, x1, y1, ...]. */
    xy: ArrayLike<number>;
    /** Cell connectivity, `cellSize` nodes per cell (counter-clockwise). */
    cells: ArrayLike<number>;
    cellSize?: 3 | 4;
    /** The field: one value per node, or per cell with `perCell`. */
    values: ArrayLike<number> | null;
    perCell?: boolean;
    range: { min: number; max: number };
    title: string;
    unit?: string;
    legendFooter?: string;
    bands?: number;
    /** Nodal displacements [ux0, uy0, ...] drawn at `deformScale`; null for the undeformed mesh. */
    displacement?: ArrayLike<number> | null;
    deformScale?: number;
    showMesh?: boolean;
    /** Cells to leave out (for instance fully broken elements). */
    hideCell?: ((cell: number) => boolean) | null;
    /** The region [xmin, ymin, xmax, ymax] the camera fits when it changes. */
    view?: [number, number, number, number] | null;
    height?: string;
    ariaLabel?: string;
  };
  let {
    xy,
    cells,
    cellSize = 4,
    values,
    perCell = false,
    range,
    title,
    unit = '',
    legendFooter = '',
    bands = 0,
    displacement = null,
    deformScale = 1,
    showMesh = true,
    hideCell = null,
    view = null,
    height = '28rem',
    ariaLabel = title,
  }: Props = $props();

  let container: HTMLDivElement;
  let renderer: THREE.WebGLRenderer | null = null;
  let camera: THREE.PerspectiveCamera;
  let controls: OrbitControls;
  let scene: THREE.Scene;
  let field: ScalarFieldMesh | null = null;
  let ro: ResizeObserver | null = null;
  let damping = 0;
  const frame = createFrameCoalescer<void>(() => render());

  function render() {
    if (!renderer) return;
    controls.update();
    renderer.render(scene, camera);
    if (damping > 0) {
      damping -= 1;
      frame.schedule();
    }
  }
  function invalidate() {
    damping = 12;
    frame.schedule();
  }

  function bounds(): [number, number, number, number] {
    if (view) return view;
    let [x0, y0, x1, y1] = [Infinity, Infinity, -Infinity, -Infinity];
    for (let i = 0; i < xy.length; i += 2) {
      x0 = Math.min(x0, xy[i]);
      x1 = Math.max(x1, xy[i]);
      y0 = Math.min(y0, xy[i + 1]);
      y1 = Math.max(y1, xy[i + 1]);
    }
    return [x0, y0, x1, y1];
  }

  /** Look straight down at the region, fitting it to the canvas. */
  export function fit() {
    if (!renderer) return;
    const [x0, y0, x1, y1] = bounds();
    const cx = (x0 + x1) / 2;
    const cy = (y0 + y1) / 2;
    const w = Math.max(1e-9, x1 - x0);
    const h = Math.max(1e-9, y1 - y0);
    const aspect = container.clientWidth / Math.max(1, container.clientHeight);
    const fov = THREE.MathUtils.degToRad(camera.fov);
    const dist = (1.08 * Math.max(h, w / aspect)) / 2 / Math.tan(fov / 2);
    camera.up.copy(TOP_VIEW_UP_VECTOR);
    camera.position.set(cx, cy, dist);
    camera.near = dist / 1000;
    camera.far = dist * 100;
    camera.updateProjectionMatrix();
    controls.target.set(cx, cy, 0);
    camera.lookAt(cx, cy, 0);
    invalidate();
  }

  function resize() {
    if (!renderer) return;
    const w = container.clientWidth;
    const h = container.clientHeight;
    if (!w || !h) return;
    renderer.setSize(w, h);
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
    invalidate();
  }

  onMount(() => {
    scene = createViewerScene();
    addViewerLights(scene);
    camera = createViewerPerspectiveCamera();
    renderer = createViewerRenderer(container);
    controls = createViewerControls(camera, renderer.domElement);
    applyDefaultNavigation(controls);
    controls.addEventListener('change', () => {
      const d = camera.position.distanceTo(controls.target);
      camera.near = Math.max(1e-9, d / 1000);
      camera.far = d * 100;
      camera.updateProjectionMatrix();
      frame.schedule();
    });
    controls.addEventListener('start', invalidate);
    field = createScalarFieldMesh(xy, cells, cellSize);
    scene.add(field.group);
    ro = new ResizeObserver(resize);
    ro.observe(container);
    resize();
    fit();
  });

  onDestroy(() => {
    ro?.disconnect();
    field?.dispose();
    controls?.dispose();
    renderer?.dispose();
    renderer?.domElement.remove();
    renderer = null;
  });

  $effect(() => {
    if (!field) return;
    if (values) field.setField(values, range, { perCell, bands });
    invalidate();
  });
  $effect(() => {
    if (!field) return;
    field.setDisplacement(displacement, deformScale);
    invalidate();
  });
  $effect(() => {
    if (!field) return;
    field.setHidden(hideCell);
    invalidate();
  });
  $effect(() => {
    if (!field) return;
    field.setEdgesVisible(showMesh);
    invalidate();
  });
  $effect(() => {
    void view;
    fit();
  });
</script>

<div class="sfv" style:height role="img" aria-label={ariaLabel}>
  <div class="sfv-canvas" bind:this={container}></div>
  {#if values}
    <ContourLegend {title} {unit} min={range.min} max={range.max} {bands} footer={legendFooter} />
  {/if}
  <button class="sfv-fit" type="button" onclick={() => fit()} title="Fit the view">Fit</button>
</div>

<style>
  .sfv {
    position: relative;
    width: 100%;
    border-radius: 3px;
    overflow: hidden;
    border: 1px solid rgba(143, 163, 179, 0.22);
  }
  .sfv-canvas {
    position: absolute;
    inset: 0;
  }
  .sfv-canvas :global(canvas) {
    display: block;
    touch-action: none;
  }
  .sfv :global(.shell-legend) {
    bottom: 12px;
  }
  .sfv-fit {
    position: absolute;
    left: 10px;
    top: 10px;
    background: rgba(20, 24, 38, 0.82);
    color: #e6e9f0;
    border: 1px solid rgba(255, 255, 255, 0.18);
    border-radius: 4px;
    font: 11px var(--lp-sans, sans-serif);
    padding: 3px 8px;
    cursor: pointer;
  }
</style>
