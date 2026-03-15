/**
 * Groundwork Web — main entry point.
 *
 * Initializes the WASM simulation (or mock data), builds the voxel mesh,
 * sets up Three.js scene with lighting and camera, HUD overlay with tool
 * palette and species picker, raycaster click-to-place, and render loop.
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GRID_Z, GROUND_LEVEL, VOXEL_BYTES, Material, ToolCode, initSim, isInitialized, getGridView, tick as simTick, placeTool } from './bridge';
import { CHUNK_SIZE } from './mesher/greedy';
import { createPlantDemoGrid } from './mesher/mockGrid';
import { ChunkManager } from './mesher/chunk';
import { buildChunkMesh, setXrayMode, adjustCutawayDepth } from './rendering/terrain';
import { buildWaterMesh, updateWaterTime, updateWaterSun } from './rendering/water';
import { FoliageRenderer } from './rendering/foliage';
import { SeedRenderer } from './rendering/seeds';
import { GrowthParticles } from './rendering/particles';
import { FaunaRenderer } from './rendering/fauna';
import { EcologyParticles } from './rendering/ecology';
import { buildSkirtMesh, buildForestRing, updateForestCulling, type SkirtWall } from './rendering/skirt';
import { OrbitCamera } from './camera/orbit';
import { createLighting } from './lighting/sun';
import { createPostProcessing } from './postprocessing/effects';
import { Hud } from './ui/hud';
import { setupControls } from './ui/controls';
import { QuestLog } from './ui/quests';
import { initScreenshot, captureScreenshot } from './ui/screenshot';
import { DayCycle } from './lighting/daycycle';
import { createSkyGradient } from './lighting/sky';
import { initAgentAPI } from './agent-api';

async function main() {
  // --- WASM init ---
  const wasmReady = await initSim();

  // --- Scene setup ---

  const renderer = new THREE.WebGLRenderer({ antialias: true, preserveDrawingBuffer: true });
  renderer.setPixelRatio(window.devicePixelRatio);
  renderer.setSize(window.innerWidth, window.innerHeight);
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type = THREE.PCFSoftShadowMap;
  renderer.localClippingEnabled = true;
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  renderer.toneMappingExposure = 1.1;
  document.body.appendChild(renderer.domElement);
  initScreenshot(renderer.domElement);

  const scene = new THREE.Scene();
  // Very light fog for subtle depth. The sky dome paints the full landscape
  // backdrop, so fog only needs to soften the 3D tree ring slightly.
  scene.fog = new THREE.FogExp2(0xc8ddb8, 0.001);

  // Camera
  const orbit = new OrbitCamera(window.innerWidth / window.innerHeight);

  // Lighting
  const lights = createLighting(scene);

  // Sky gradient dome (replaces flat background color)
  const skyUniforms = createSkyGradient(scene);

  // Day cycle controller
  const dayCycle = new DayCycle();

  // --- Voxel mesh ---

  // Use real sim grid if WASM loaded, otherwise mock
  let grid: Uint8Array;
  if (wasmReady) {
    // Run a few ticks to populate the world (water/light propagation)
    simTick(5);
    grid = getGridView();
  } else {
    grid = createPlantDemoGrid();
  }

  const chunkManager = new ChunkManager();
  chunkManager.detectChanges(grid);
  const updatedChunks = chunkManager.rebuildDirty(grid);

  // Group for all chunk meshes
  const terrainGroup = new THREE.Group();
  scene.add(terrainGroup);

  // Map from chunk key to Three.js meshes (solid + soil)
  const chunkMeshes = new Map<string, THREE.Mesh>();

  // X-ray mode state
  let xrayActive = false;

  for (const chunk of updatedChunks) {
    const { solidMesh, soilMesh, rootMesh } = buildChunkMesh(chunk);
    if (solidMesh) {
      solidMesh.castShadow = true;
      solidMesh.receiveShadow = true;
      terrainGroup.add(solidMesh);
      chunkMeshes.set(solidMesh.name, solidMesh);
    }
    if (soilMesh) {
      soilMesh.castShadow = true;
      soilMesh.receiveShadow = true;
      terrainGroup.add(soilMesh);
      chunkMeshes.set(soilMesh.name, soilMesh);
    }
    if (rootMesh) {
      rootMesh.castShadow = true;
      rootMesh.receiveShadow = true;
      terrainGroup.add(rootMesh);
      chunkMeshes.set(rootMesh.name, rootMesh);
    }
  }

  // --- Ground skirt (hides underground cross-section) ---

  const { group: skirt, walls: skirtWalls } = buildSkirtMesh();
  scene.add(skirt);

  // --- Decorative forest ring (pure Three.js scenery, no sim cost) ---

  const forestRing = buildForestRing();
  scene.add(forestRing);
  const meadowGround = forestRing.getObjectByName('meadow_ground') as THREE.Mesh;
  const meadowClipPlane = new THREE.Plane();
  // Attach clip plane to meadow material — updated each frame when x-ray is active
  (meadowGround.material as THREE.MeshLambertMaterial).clippingPlanes = [meadowClipPlane];

  // --- Water surface ---

  const waterMesh = buildWaterMesh(grid);
  if (waterMesh) {
    scene.add(waterMesh);
  }

  // Sync sun direction to water shader
  const sunDir = new THREE.Vector3();
  sunDir.subVectors(lights.sun.target.position, lights.sun.position).negate();
  updateWaterSun(sunDir, lights.sun.intensity);

  // --- Foliage (billboard sprites with wind sway) ---

  const foliage = new FoliageRenderer();
  scene.add(foliage.group);
  foliage.rebuild(grid);

  // --- Seeds (tiny mound sprites) ---

  const seeds = new SeedRenderer();
  scene.add(seeds.group);
  seeds.rebuild(grid);

  // --- Fauna (ecological creature sprites) ---

  const fauna = new FaunaRenderer();
  scene.add(fauna.group);

  // --- Growth particles ---

  const particles = new GrowthParticles();
  scene.add(particles.points);
  // Initial detection pass (no bursts on first load)
  particles.detectGrowth(grid);

  // --- Ecology interaction particles ---

  const ecology = new EcologyParticles();
  scene.add(ecology.points);

  // --- Post-processing ---

  const postProcessing = createPostProcessing(renderer, scene, orbit.camera);

  // --- HUD & Controls ---

  const hud = new Hud();
  const questLog = new QuestLog();

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

  /** Re-mesh dirty chunks, re-fetching the grid view if using WASM */
  function remeshDirty(): void {
    const freshGrid = isInitialized() ? getGridView() : grid;
    chunkManager.detectChanges(freshGrid);
    const rebuilt = chunkManager.rebuildDirty(freshGrid);
    for (const chunk of rebuilt) {
      // Remove old solid, soil, and root meshes for this chunk
      const solidName = `chunk_${chunk.cx}_${chunk.cy}_${chunk.cz}`;
      const soilName = `soil_${chunk.cx}_${chunk.cy}_${chunk.cz}`;
      const rootName = `root_${chunk.cx}_${chunk.cy}_${chunk.cz}`;
      for (const name of [solidName, soilName, rootName]) {
        const old = chunkMeshes.get(name);
        if (old) {
          terrainGroup.remove(old);
          old.geometry.dispose();
          chunkMeshes.delete(name);
        }
      }
      const { solidMesh, soilMesh, rootMesh } = buildChunkMesh(chunk);
      if (solidMesh) {
        solidMesh.castShadow = true;
        solidMesh.receiveShadow = true;
        terrainGroup.add(solidMesh);
        chunkMeshes.set(solidMesh.name, solidMesh);
      }
      if (soilMesh) {
        soilMesh.castShadow = true;
        soilMesh.receiveShadow = true;
        terrainGroup.add(soilMesh);
        chunkMeshes.set(soilMesh.name, soilMesh);
      }
      if (rootMesh) {
        rootMesh.castShadow = true;
        rootMesh.receiveShadow = true;
        terrainGroup.add(rootMesh);
        chunkMeshes.set(rootMesh.name, rootMesh);
      }
    }
    // Rebuild foliage, seeds, and detect growth after grid changes
    foliage.rebuild(freshGrid);
    seeds.rebuild(freshGrid);
    particles.detectGrowth(freshGrid);
  }

  setupControls({
    hud,
    questLog,
    camera: orbit.camera,
    terrainGroup,
    canvas: renderer.domElement,
    onToolPlaced: (hit) => {
      if (isInitialized()) {
        // WASM mode: use the real sim's place_tool (handles gravity, validation)
        placeTool(hud.state.activeTool, hit.x, hit.y, hit.z);
      } else {
        // Mock mode: apply directly to the grid
        applyToolToMockGrid(hud.state.activeTool, hit.x, hit.y, hit.z);
      }
      // Record tool use for quest tracking
      const speciesIdx = hud.state.activeSpeciesIndex;
      questLog.recordToolUse(hud.state.activeTool, speciesIdx);
      questLog.recordClick(hit.x, hit.y, hit.z);
      // Check quests against updated grid
      const freshGrid = isInitialized() ? getGridView() : grid;
      questLog.check(freshGrid);
      remeshDirty();
    },
  });

  // --- Agent API (for Playwright screenshot harness) ---

  initAgentAPI({
    orbitCamera: orbit,
    remeshDirty: remeshDirty,
    dayCycle: dayCycle,
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
    if (Math.abs(dx) + Math.abs(dy) > 3) questLog.recordOrbit();
  });

  renderer.domElement.addEventListener('mouseup', () => { isDragging = false; });
  renderer.domElement.addEventListener('mouseleave', () => { isDragging = false; });

  renderer.domElement.addEventListener('wheel', (e) => {
    e.preventDefault();
    if (xrayActive && e.shiftKey) {
      // Shift+scroll in x-ray mode: adjust cutaway depth
      adjustCutawayDepth(e.deltaY > 0 ? -1 : 1);
    } else {
      orbit.zoom(e.deltaY > 0 ? 0.9 : 1.1);
    }
  }, { passive: false });

  // Keyboard: WASD/arrows for pan, Q for x-ray toggle, R for reset, Space for auto-tick
  document.addEventListener('keydown', (e) => {
    // Pass to camera for continuous movement keys
    orbit.keyDown(e.key);

    // Track pan for quests (WASD / arrows)
    if (['w', 'a', 's', 'd', 'arrowup', 'arrowdown', 'arrowleft', 'arrowright'].includes(e.key.toLowerCase())) {
      questLog.recordPan();
    }

    switch (e.key.toLowerCase()) {
      case ' ':
        e.preventDefault();
        autoTick = !autoTick;
        hud.setAutoTick(autoTick);
        questLog.recordToggleAutoTick();
        console.log(`Auto-tick: ${autoTick ? 'ON' : 'OFF'}`);
        break;
      case 'r':
        orbit.reset();
        break;
      case 't':
        // Manual single tick
        if (isInitialized()) {
          simTick(1);
          questLog.recordStepManually();
          const freshGrid = getGridView();
          questLog.check(freshGrid);
          remeshDirty();
          console.log('Ticked 1');
        }
        break;
      case 'q':
        // Toggle x-ray: make soil/stone transparent to see roots underground
        xrayActive = !xrayActive;
        setXrayMode(xrayActive);
        questLog.recordDepthChange();
        console.log(`X-ray: ${xrayActive ? 'ON' : 'OFF'}`);
        break;
      case '[':
        dayCycle.step(-0.04);
        break;
      case ']':
        dayCycle.step(0.04);
        break;
      case '\\':
        dayCycle.toggleAuto();
        break;
      case 'f2':
        e.preventDefault();
        captureScreenshot();
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
    const elapsed = clock.elapsedTime;

    // Auto-tick simulation
    if (autoTick && isInitialized()) {
      tickAccumulator += dt * 1000;
      let ticked = false;
      while (tickAccumulator >= TICK_INTERVAL_MS) {
        tickAccumulator -= TICK_INTERVAL_MS;
        simTick(1);
        ticked = true;
      }
      if (ticked) {
        const freshGrid = getGridView();
        questLog.check(freshGrid);
      }
      remeshDirty();
    }

    // Fade quest notifications
    questLog.tickNotification();

    // Animate water ripples
    updateWaterTime(elapsed);

    // Update day cycle (sun position, colors, sky gradient)
    dayCycle.update(dt, lights, scene, skyUniforms);

    // Animate foliage wind sway
    foliage.update(elapsed);

    // Update fauna positions and animation
    fauna.update(elapsed);

    // Animate growth particles
    particles.update(dt);

    // Ecology interaction indicators
    const freshGridForEco = isInitialized() ? getGridView() : grid;
    ecology.update(dt, freshGridForEco);

    orbit.update(dt);
    updateForestCulling(forestRing, orbit.getTheta());

    // X-ray culling — same dot-product math as tree culling.
    // Skirt walls: hide camera-facing walls.
    // Meadow: clip the camera-facing half with a plane through garden center.
    {
      const theta = orbit.getTheta();
      const camDirX = Math.cos(theta);
      const camDirZ = Math.sin(theta);
      for (const wall of skirtWalls) {
        if (!xrayActive) {
          wall.mesh.visible = true;
        } else {
          const dot = camDirX * wall.normalXZ[0] + camDirZ * wall.normalXZ[1];
          wall.mesh.visible = dot < 0.5;
        }
      }
      // Clip plane slices the meadow: normal points away from camera,
      // plane passes through garden center. Camera-side half is clipped.
      if (xrayActive) {
        const cx = GRID_X / 2;
        const cz = GRID_Y / 2;
        meadowClipPlane.set(
          new THREE.Vector3(-camDirX, 0, -camDirZ),
          camDirX * cx + camDirZ * cz,
        );
      } else {
        // Move plane far away so nothing is clipped
        meadowClipPlane.set(new THREE.Vector3(0, -1, 0), 9999);
      }
    }

    postProcessing.composer.render();
  }

  animate();

  // Build stamp — visible on screen so we can confirm code is live
  const BUILD_STAMP = `build:${Date.now().toString(36)} t=${dayCycle.getTime().toFixed(2)}`;
  const stampEl = document.createElement('div');
  stampEl.textContent = BUILD_STAMP;
  stampEl.style.cssText = 'position:fixed;bottom:2px;left:2px;color:#fff;font:12px monospace;opacity:0.7;pointer-events:none;z-index:9999;background:rgba(0,0,0,0.5);padding:2px 6px';
  document.body.appendChild(stampEl);

  console.log(
    '%c🌱 Groundwork Web',
    'color: #4a7; font-size: 16px; font-weight: bold',
  );
  console.log(`Build: ${BUILD_STAMP}`);
  console.log(
    `Grid: ${GRID_X}x${GRID_Y}x${GRID_Z} | ` +
    `Mode: ${wasmReady ? 'WASM sim' : 'Mock data'} | ` +
    `Chunks: ${chunkMeshes.size} active | ` +
    `Foliage: ${foliage.count} sprites | ` +
    `Seeds: ${seeds.count} sprites | ` +
    `Fauna: ${fauna.count} creatures`,
  );
  console.log(
    'Controls: 1-5=tools, WASD/Arrows=pan, Q=x-ray, R=reset, T=tick, drag=orbit, scroll=zoom, space=auto-tick, []=time, \\=auto-cycle, F2=screenshot',
  );
}

main();
