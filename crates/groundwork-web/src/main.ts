/**
 * Groundwork Web — main entry point.
 *
 * Initializes the WASM simulation (or mock data), builds the voxel mesh,
 * sets up Three.js scene with lighting and camera, and runs the render loop.
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GRID_Z, GROUND_LEVEL } from './bridge';
import { createMockGrid } from './mesher/greedy';
import { ChunkManager } from './mesher/chunk';
import { buildChunkMesh } from './rendering/terrain';
import { OrbitCamera } from './camera/orbit';
import { createLighting } from './lighting/sun';
import { createPostProcessing } from './postprocessing/effects';

// --- Scene setup ---

const renderer = new THREE.WebGLRenderer({ antialias: true });
renderer.setPixelRatio(window.devicePixelRatio);
renderer.setSize(window.innerWidth, window.innerHeight);
renderer.shadowMap.enabled = true;
renderer.shadowMap.type = THREE.PCFSoftShadowMap;
renderer.toneMapping = THREE.ACESFilmicToneMapping;
renderer.toneMappingExposure = 1.1;
document.body.appendChild(renderer.domElement);

const scene = new THREE.Scene();
scene.background = new THREE.Color(0x87CEEB); // sky blue

// Fog for depth
scene.fog = new THREE.FogExp2(0x87CEEB, 0.005);

// Camera
const orbit = new OrbitCamera(window.innerWidth / window.innerHeight);

// Lighting
const lights = createLighting(scene);

// --- Voxel mesh ---

// Use mock data for now (WASM bridge connects later)
const grid = createMockGrid();

const chunkManager = new ChunkManager();
chunkManager.detectChanges(grid);
const updatedChunks = chunkManager.rebuildDirty(grid);

// Group for all chunk meshes
const terrainGroup = new THREE.Group();
scene.add(terrainGroup);

// Map from chunk key to Three.js mesh
const chunkMeshes = new Map<string, THREE.Mesh>();

for (const chunk of updatedChunks) {
  const mesh = buildChunkMesh(chunk);
  if (mesh) {
    mesh.castShadow = true;
    mesh.receiveShadow = true;
    terrainGroup.add(mesh);
    chunkMeshes.set(mesh.name, mesh);
  }
}

// --- Post-processing ---

const postProcessing = createPostProcessing(renderer, scene, orbit.camera);

// --- Sim state ---

let autoTick = false;
let tickAccumulator = 0;
const TICK_INTERVAL_MS = 200;

// --- Input ---

let isDragging = false;
let lastMouseX = 0;
let lastMouseY = 0;

renderer.domElement.addEventListener('mousedown', (e) => {
  isDragging = true;
  lastMouseX = e.clientX;
  lastMouseY = e.clientY;
});

renderer.domElement.addEventListener('mousemove', (e) => {
  if (!isDragging) return;
  const dx = e.clientX - lastMouseX;
  const dy = e.clientY - lastMouseY;
  orbit.rotate(-dx * 0.005, -dy * 0.005);
  lastMouseX = e.clientX;
  lastMouseY = e.clientY;
});

renderer.domElement.addEventListener('mouseup', () => { isDragging = false; });
renderer.domElement.addEventListener('mouseleave', () => { isDragging = false; });

renderer.domElement.addEventListener('wheel', (e) => {
  e.preventDefault();
  orbit.zoom(e.deltaY > 0 ? 0.9 : 1.1);
}, { passive: false });

document.addEventListener('keydown', (e) => {
  switch (e.key) {
    case ' ':
      autoTick = !autoTick;
      break;
  }
});

// --- Resize ---

window.addEventListener('resize', () => {
  renderer.setSize(window.innerWidth, window.innerHeight);
  orbit.resize(window.innerWidth / window.innerHeight);
  postProcessing.resize(window.innerWidth, window.innerHeight);
});

// --- Hide loading screen ---

const loadingEl = document.getElementById('loading');
if (loadingEl) loadingEl.classList.add('hidden');

// --- Render loop ---

const clock = new THREE.Clock();

function animate(): void {
  requestAnimationFrame(animate);

  const delta = clock.getDelta() * 1000; // ms

  // Auto-tick simulation
  if (autoTick) {
    tickAccumulator += delta;
    while (tickAccumulator >= TICK_INTERVAL_MS) {
      tickAccumulator -= TICK_INTERVAL_MS;
      // When WASM is connected: tick(1) then re-mesh dirty chunks
    }
  }

  orbit.update();
  postProcessing.composer.render();
}

animate();

console.log(
  '%c🌱 Groundwork Web',
  'color: #4a7; font-size: 16px; font-weight: bold',
);
console.log(
  `Grid: ${GRID_X}x${GRID_Y}x${GRID_Z} | ` +
  `Chunks: ${chunkMeshes.size} active | ` +
  `Drag to orbit, scroll to zoom, space to auto-tick`,
);
