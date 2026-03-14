/**
 * Groundwork Web — main entry point.
 *
 * Initializes the WASM simulation (or mock data), builds the voxel mesh,
 * sets up Three.js scene with lighting and camera, HUD overlay with tool
 * palette and species picker, raycaster click-to-place, and render loop.
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GRID_Z, GROUND_LEVEL, VOXEL_BYTES, Material, ToolCode } from './bridge';
import { createMockGrid, CHUNK_SIZE } from './mesher/greedy';
import { ChunkManager } from './mesher/chunk';
import { buildChunkMesh } from './rendering/terrain';
import { buildWaterMesh, updateWaterTime, updateWaterSun } from './rendering/water';
import { OrbitCamera } from './camera/orbit';
import { createLighting } from './lighting/sun';
import { createPostProcessing } from './postprocessing/effects';
import { Hud } from './ui/hud';
import { setupControls } from './ui/controls';

// --- Scene setup ---

const renderer = new THREE.WebGLRenderer({ antialias: true });
renderer.setPixelRatio(window.devicePixelRatio);
renderer.setSize(window.innerWidth, window.innerHeight);
renderer.shadowMap.enabled = true;
renderer.shadowMap.type = THREE.PCFSoftShadowMap;
renderer.toneMapping = THREE.ACESFilmicToneMapping;
renderer.toneMappingExposure = 1.1;
renderer.localClippingEnabled = true; // Required for cutaway clipping planes
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

// Cutaway clipping planes array (shared reference — plane constant updates each frame)
const clippingPlanes = [orbit.cutawayPlane];

for (const chunk of updatedChunks) {
  const mesh = buildChunkMesh(chunk, clippingPlanes);
  if (mesh) {
    mesh.castShadow = true;
    mesh.receiveShadow = true;
    terrainGroup.add(mesh);
    chunkMeshes.set(mesh.name, mesh);
  }
}

// --- Water surface ---

const waterMesh = buildWaterMesh(grid);
if (waterMesh) {
  scene.add(waterMesh);
}

// Sync sun direction to water shader
const sunDir = new THREE.Vector3();
sunDir.subVectors(lights.sun.target.position, lights.sun.position).negate();
updateWaterSun(sunDir, lights.sun.intensity);

// --- Post-processing ---

const postProcessing = createPostProcessing(renderer, scene, orbit.camera);

// --- HUD & Controls ---

const hud = new Hud();

/** Apply a tool to the mock grid and re-mesh affected chunks */
function applyToolToMockGrid(toolCode: number, x: number, y: number, z: number): void {
  const idx = (x + y * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
  if (idx < 0 || idx >= grid.length) return;

  switch (toolCode) {
    case ToolCode.Shovel:
      grid[idx] = Material.Air;
      grid[idx + 1] = 0; // water
      grid[idx + 2] = 0; // light
      grid[idx + 3] = 0; // nutrients
      break;
    case ToolCode.Seed:
      grid[idx] = Material.Seed;
      break;
    case ToolCode.Water:
      grid[idx] = Material.Water;
      grid[idx + 1] = 255;
      break;
    case ToolCode.Soil:
      grid[idx] = Material.Soil;
      break;
    case ToolCode.Stone:
      grid[idx] = Material.Stone;
      break;
  }
}

/** Re-mesh chunks affected by a voxel change at (x, y, z) */
function remeshAt(x: number, y: number, z: number): void {
  chunkManager.detectChanges(grid);
  const rebuilt = chunkManager.rebuildDirty(grid);
  for (const chunk of rebuilt) {
    const name = `chunk_${chunk.cx}_${chunk.cy}_${chunk.cz}`;
    // Remove old mesh
    const old = chunkMeshes.get(name);
    if (old) {
      terrainGroup.remove(old);
      old.geometry.dispose();
      chunkMeshes.delete(name);
    }
    // Add new mesh
    const mesh = buildChunkMesh(chunk);
    if (mesh) {
      mesh.castShadow = true;
      mesh.receiveShadow = true;
      terrainGroup.add(mesh);
      chunkMeshes.set(mesh.name, mesh);
    }
  }
}

setupControls({
  hud,
  camera: orbit.camera,
  terrainGroup,
  canvas: renderer.domElement,
  onToolPlaced: (hit) => {
    // In mock mode, apply directly to the grid
    applyToolToMockGrid(hud.state.activeTool, hit.x, hit.y, hit.z);
    remeshAt(hit.x, hit.y, hit.z);
  },
});

// --- Sim state ---

let autoTick = false;
let tickAccumulator = 0;
const TICK_INTERVAL_MS = 200;

// --- Camera orbit input ---

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

// Keyboard: WASD/arrows for pan, Q/E for cutaway depth, R for reset, Space for auto-tick
document.addEventListener('keydown', (e) => {
  // Pass to camera for continuous movement keys
  orbit.keyDown(e.key);

  switch (e.key.toLowerCase()) {
    case ' ':
      e.preventDefault();
      autoTick = !autoTick;
      hud.setAutoTick(autoTick);
      break;
    case 'r':
      orbit.reset();
      break;
  }
});

document.addEventListener('keyup', (e) => {
  orbit.keyUp(e.key);
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

  const dt = clock.getDelta(); // seconds

  // Auto-tick simulation
  if (autoTick) {
    tickAccumulator += dt * 1000;
    while (tickAccumulator >= TICK_INTERVAL_MS) {
      tickAccumulator -= TICK_INTERVAL_MS;
      // When WASM is connected: tick(1) then re-mesh dirty chunks
    }
  }

  // Animate water ripples
  updateWaterTime(clock.elapsedTime);

  orbit.update(dt);
  postProcessing.composer.render();
}

animate();

console.log(
  '%c🌱 Groundwork Web',
  'color: #4a7; font-size: 16px; font-weight: bold',
);
console.log(
  `Grid: ${GRID_X}x${GRID_Y}x${GRID_Z} | ` +
  `Chunks: ${chunkMeshes.size} active`,
);
console.log(
  'Controls: 1-5=tools, WASD/Arrows=pan, Q/E=cutaway, R=reset, drag=orbit, scroll=zoom, space=auto-tick',
);
