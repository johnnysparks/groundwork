/**
 * Groundwork Web — main entry point.
 *
 * Initializes the WASM simulation (or mock data), builds the voxel mesh,
 * sets up Three.js scene with lighting and camera, HUD overlay with tool
 * palette and species picker, raycaster click-to-place, and render loop.
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GRID_Z, GROUND_LEVEL, VOXEL_BYTES, Material, ToolCode, initSim, isInitialized, getGridView, tick as simTick, placeTool, fillTool, getTick, getFaunaCount, getFaunaView, readFauna } from './bridge';
import { CHUNK_SIZE } from './mesher/greedy';
import { SCENES, getSceneId } from './mesher/mockGrid';
import { ChunkManager } from './mesher/chunk';
import { buildChunkMesh, setXrayMode, adjustCutawayDepth } from './rendering/terrain';
import { buildWaterMesh, updateWaterTime, updateWaterSun } from './rendering/water';
import { FoliageRenderer } from './rendering/foliage';
import { SeedRenderer } from './rendering/seeds';
import { GrowthParticles } from './rendering/particles';
import { FaunaRenderer } from './rendering/fauna';
import { EcologyParticles } from './rendering/ecology';
import { DataOverlay, OverlayMode } from './rendering/overlay';
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

/** Scan the grid and count plant voxels, unique species, and fauna */
function computeGardenStats(grid: Uint8Array): { plants: number; fauna: number; species: number } {
  let plants = 0;
  const speciesSet = new Set<number>();
  const total = GRID_X * GRID_Y * GRID_Z;
  for (let i = 0; i < total; i++) {
    const mat = grid[i * VOXEL_BYTES];
    if (mat === Material.Trunk || mat === Material.Leaf || mat === Material.Branch) {
      plants++;
      const speciesId = grid[i * VOXEL_BYTES + 3];
      speciesSet.add(speciesId);
    }
  }
  // Fauna count from bridge
  let fauna = 0;
  try { fauna = getFaunaCount(); } catch {}
  return { plants, fauna, species: speciesSet.size };
}

/** Track previous stats for event detection */
let _prevStats = { plants: 0, fauna: 0, species: 0 };
let _eventCooldown = 0;

const FAUNA_NAMES: Record<number, string> = {
  0: 'bee', 1: 'butterfly', 2: 'bird', 3: 'worm', 4: 'beetle',
};

/** Detect ecological events by comparing with previous stats */
function detectEvents(stats: { plants: number; fauna: number; species: number }, hud: any): void {
  _eventCooldown--;
  if (_eventCooldown > 0) return;

  // New fauna appeared
  if (stats.fauna > _prevStats.fauna) {
    const diff = stats.fauna - _prevStats.fauna;
    // Try to identify what kind by reading fauna data
    const fView = getFaunaView?.();
    if (fView) {
      const f = readFauna(fView, stats.fauna - 1);
      const name = FAUNA_NAMES[f.type] ?? 'creature';
      hud.addEvent(`A ${name} appeared in your garden`);
    } else {
      hud.addEvent(`${diff} new creature${diff > 1 ? 's' : ''} appeared`);
    }
    _eventCooldown = 20; // Don't spam
  }

  // New species growing
  if (stats.species > _prevStats.species) {
    hud.addEvent(`New species growing — ${stats.species} types in your garden`);
    _eventCooldown = 15;
  }

  // Major plant growth burst (>500 new voxels)
  if (stats.plants > _prevStats.plants + 500 && _prevStats.plants > 0) {
    hud.addEvent('Growth burst — your garden is flourishing');
    _eventCooldown = 30;
  }

  _prevStats = { ...stats };
}

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
  dayCycle.setTime(0.75); // Start at golden hour — warm and inviting

  // --- Voxel mesh ---

  // Select scene based on URL parameter (or auto-detect)
  const sceneId = getSceneId(wasmReady);
  const sceneDef = SCENES.find(s => s.id === sceneId);
  let grid: Uint8Array;
  if (sceneDef?.createGrid) {
    // Mock scene — use the factory function
    grid = sceneDef.createGrid();
  } else if (wasmReady) {
    // WASM simulation
    simTick(5);
    grid = getGridView();
  } else {
    // Fallback: first scene with a grid factory
    const fallback = SCENES.find(s => s.createGrid);
    grid = fallback!.createGrid!();
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
    const { solidMesh, soilMesh, rootMesh } = buildChunkMesh(chunk, grid);
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

  // --- Data overlay (V key: water/light/nutrient heat maps) ---

  const overlay = new DataOverlay();
  scene.add(overlay.group);

  // --- Post-processing ---

  const postProcessing = createPostProcessing(renderer, scene, orbit.camera);

  // --- HUD & Controls ---

  const hud = new Hud();
  if (isInitialized()) {
    hud.setTickCount(Number(getTick()));
    hud.setAutoTick(true);
  }
  const questLog = new QuestLog();

  // Welcome message
  if (wasmReady) {
    hud.addEvent('Your garden is growing — click to place zones');
    hud.addEvent('Press 3 for water, 2 for seeds');
  }

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
      const { solidMesh, soilMesh, rootMesh } = buildChunkMesh(chunk, freshGrid);
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
      // Zone-based placement with water cost.
      const tool = hud.state.activeTool;
      const r = tool === ToolCode.Seed ? 4 : tool === ToolCode.Shovel ? 3 : 3;
      // Water costs: seeds=15, water=20, soil=10, shovel=5, stone=10
      const costs: Record<number, number> = {
        [ToolCode.Seed]: 15, [ToolCode.Water]: 20,
        [ToolCode.Soil]: 10, [ToolCode.Shovel]: 5, [ToolCode.Stone]: 10,
      };
      const cost = costs[tool] ?? 10;
      if (!hud.spendWater(cost)) {
        hud.addEvent('Not enough water — wait for the spring to refill');
        return;
      }
      if (isInitialized()) {
        fillTool(tool, hit.x - r, hit.y - r, hit.z, hit.x + r, hit.y + r, hit.z);
      } else {
        applyToolToMockGrid(tool, hit.x, hit.y, hit.z);
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
    setXrayMode: (active: boolean) => {
      xrayActive = active;
      setXrayMode(xrayActive);
    },
    setTickCount: (count: number) => hud.setTickCount(count),
    overlay: overlay,
  });

  // --- Sim state ---

  let autoTick = wasmReady; // Auto-tick ON by default when sim is ready
  let tickAccumulator = 0;
  let tickSpeed = 1; // 1x, 2x, 5x
  const BASE_TICK_MS = 100;
  let TICK_INTERVAL_MS = BASE_TICK_MS;

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
      case '-':
      case '_':
        // Slow down
        if (tickSpeed > 1) {
          tickSpeed = tickSpeed === 5 ? 2 : 1;
          TICK_INTERVAL_MS = BASE_TICK_MS / tickSpeed;
          hud.addEvent(`Speed: ${tickSpeed}x`);
        }
        break;
      case '=':
      case '+':
        // Speed up
        if (tickSpeed < 5) {
          tickSpeed = tickSpeed === 1 ? 2 : 5;
          TICK_INTERVAL_MS = BASE_TICK_MS / tickSpeed;
          hud.addEvent(`Speed: ${tickSpeed}x`);
        }
        break;
      case 't':
        // Manual single tick
        if (isInitialized()) {
          simTick(1);
          hud.setTickCount(Number(getTick()));
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
      case 'v': {
        // Data overlay: V=toggle, Shift+V=cycle mode
        const freshGrid = isInitialized() ? getGridView() : grid;
        if (e.shiftKey) {
          overlay.cycle();
        } else {
          overlay.toggle();
        }
        if (overlay.mode !== OverlayMode.Off) {
          overlay.rebuild(freshGrid);
        }
        console.log(`Overlay: ${overlay.modeName}`);
        break;
      }
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
        hud.setTickCount(Number(getTick()));
        const freshGrid = getGridView();
        questLog.check(freshGrid);
        if (overlay.mode !== OverlayMode.Off) overlay.rebuild(freshGrid);
        // Replenish water budget from spring (2 per tick)
        hud.replenishWater(2);
        // Update garden stats for HUD + detect ecological events
        const stats = computeGardenStats(freshGrid);
        hud.setGardenStats(stats);
        detectEvents(stats, hud);
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
