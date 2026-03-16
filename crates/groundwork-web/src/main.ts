/**
 * Groundwork Web — main entry point.
 *
 * Initializes the WASM simulation (or mock data), builds the voxel mesh,
 * sets up Three.js scene with lighting and camera, HUD overlay with tool
 * palette and species picker, raycaster click-to-place, and render loop.
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GRID_Z, GROUND_LEVEL, VOXEL_BYTES, Material, ToolCode, SPECIES, initSim, isInitialized, getGridView, tick as simTick, placeTool, fillTool, getTick, getFaunaCount, getFaunaView, readFauna, resetSim, saveGrid, restoreGrid, setSelectedSpecies } from './bridge';
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
import { TaskQueue } from './gardener/queue';
import { GhostOverlay } from './gardener/ghosts';
import { GardenerSprite } from './gardener/gardener';
import { QuestLog } from './ui/quests';
import { initScreenshot, captureScreenshot } from './ui/screenshot';
import { DayCycle } from './lighting/daycycle';
import { createSkyGradient } from './lighting/sky';
import { initAgentAPI } from './agent-api';

/** Scan the grid and count plant voxels, unique species, and fauna */
function computeGardenStats(grid: Uint8Array): { plants: number; fauna: number; species: number; speciesIds: Set<number> } {
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
  return { plants, fauna, species: speciesSet.size, speciesIds: speciesSet };
}

/** Species names for event messages */
const SPECIES_NAMES: Record<number, string> = {
  0: 'Oak', 1: 'Birch', 2: 'Willow', 3: 'Pine',
  4: 'Fern', 5: 'Berry Bush', 6: 'Holly',
  7: 'Wildflower', 8: 'Daisy', 9: 'Moss', 10: 'Grass', 11: 'Clover',
};

/** Pioneer succession species (appear autonomously) */
const PIONEER_SPECIES = new Set([9, 10, 7]); // moss, grass, wildflower

const SUCCESSION_MESSAGES: Record<number, string> = {
  9: 'Moss is colonizing bare soil — the first sign of life',
  10: 'Grass is spreading through the mossy patches',
  7: 'Wildflowers appeared in the grassy areas — succession is working!',
};

/** Track previous stats for event detection */
let _prevStats = { plants: 0, fauna: 0, species: 0 } as any;
let _prevSpeciesIds = new Set<number>();
let _eventCooldown = 0;

const FAUNA_NAMES: Record<number, string> = {
  0: 'bee', 1: 'butterfly', 2: 'bird', 3: 'worm', 4: 'beetle',
};

/** Fauna arrival messages that teach ecology */
const FAUNA_MESSAGES: Record<number, string[]> = {
  0: [ // Bee
    'A bee arrived — it will pollinate nearby flowers',
    'Bee spotted! Flowers nearby will spread seeds faster',
    'A bee is visiting your garden — plant more flowers to attract more',
  ],
  1: [ // Butterfly
    'A butterfly appeared — it boosts seed nutrients when visiting flowers',
    'Butterfly visiting! It helps flowers reproduce',
  ],
  2: [ // Bird
    'A bird is nesting near your trees — it may carry seeds to new spots',
    'Bird spotted! It picks up seeds and drops them across the garden',
    'A bird arrived — watch for surprise plants it delivers',
  ],
  3: [ // Worm
    'A worm is enriching the soil underground — nutrients are rising',
    'Worm activity detected — soil quality is improving',
  ],
  4: [ // Beetle
    'A beetle appeared near dead wood — it breaks down old plants into soil',
    'Beetle at work — decomposition feeds the next generation',
  ],
};

/** Periodic ecology tips */
const ECO_TIPS = [
  'Tip: Plant clover near oak trees — nitrogen fixing boosts their growth 50%',
  'Tip: More flower variety = more pollinators = faster seed spread',
  'Tip: Press Q for x-ray mode — see root networks competing underground',
  'Tip: Press V to see water, light, or nutrient overlays',
  'Tip: Birds near berry bushes carry seeds to surprising new spots',
  'Tip: Each new species adds +100 to your score',
  'Tip: Groundcover near trees enriches the soil for everyone',
];
let _tipIndex = 0;
let _tipTimer = 0;

/** Detect ecological events by comparing with previous stats */
function detectEvents(stats: { plants: number; fauna: number; species: number; speciesIds?: Set<number> }, hud: any): void {
  _eventCooldown--;
  if (_eventCooldown > 0) return;

  // New fauna appeared — with ecological explanation
  if (stats.fauna > _prevStats.fauna) {
    const fView = getFaunaView?.();
    if (fView) {
      const f = readFauna(fView, stats.fauna - 1);
      const msgs = FAUNA_MESSAGES[f.type];
      if (msgs) {
        hud.addEvent(msgs[Math.floor(Math.random() * msgs.length)]);
      }
    }
    _eventCooldown = 25;
  }

  // New species growing — check if it's pioneer succession
  if (stats.species > _prevStats.species && stats.speciesIds) {
    for (const sid of stats.speciesIds) {
      if (!_prevSpeciesIds.has(sid)) {
        const msg = SUCCESSION_MESSAGES[sid];
        if (msg) {
          hud.addEvent(msg);
        } else {
          const name = SPECIES_NAMES[sid] ?? `Species ${sid}`;
          hud.addEvent(`${name} is now growing in your garden (+100 score)`);
        }
        _eventCooldown = 20;
        break; // one event per tick
      }
    }
    _prevSpeciesIds = new Set(stats.speciesIds);
  }

  // Major plant growth burst
  if (stats.plants > _prevStats.plants + 500 && _prevStats.plants > 0) {
    hud.addEvent('Growth burst — your garden is flourishing');
    _eventCooldown = 30;
  }

  // Plant die-off from competition
  if (_prevStats.plants > 100 && stats.plants < _prevStats.plants - 200) {
    hud.addEvent('A plant died from competition — the strongest survive');
    _eventCooldown = 40;
  }

  // Periodic ecology tips (every ~60 ticks when nothing else happens)
  _tipTimer++;
  if (_tipTimer > 60 && _eventCooldown <= 0) {
    hud.addEvent(ECO_TIPS[_tipIndex % ECO_TIPS.length]);
    _tipIndex++;
    _tipTimer = 0;
    _eventCooldown = 10;
  }

  if (stats.speciesIds) _prevSpeciesIds = new Set(stats.speciesIds);
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
  // Atmospheric fog: blends the peripheral forest ring into the sky gradient.
  // Color and density are updated each frame by the DayCycle to match lighting.
  scene.fog = new THREE.FogExp2(0xccddcc, 0.0025);

  // Camera
  const orbit = new OrbitCamera(window.innerWidth / window.innerHeight);

  // Lighting
  const lights = createLighting(scene);

  // Sky gradient dome (replaces flat background color)
  const skyUniforms = createSkyGradient(scene);

  // Day cycle controller
  const dayCycle = new DayCycle();
  dayCycle.setTime(0.55); // Start just past noon — bright, warm, lets greens read as green

  // --- Voxel mesh ---

  // Select scene based on URL parameter (or auto-detect)
  const sceneId = getSceneId(wasmReady);
  const sceneDef = SCENES.find(s => s.id === sceneId);
  let grid: Uint8Array;
  if (sceneDef?.createGrid) {
    // Mock scene — use the factory function
    grid = sceneDef.createGrid();
  } else if (wasmReady) {
    // WASM simulation — try to restore saved garden
    simTick(5);
    const savedData = localStorage.getItem('groundwork-garden');
    if (savedData) {
      try {
        const bytes = Uint8Array.from(atob(savedData), c => c.charCodeAt(0));
        if (restoreGrid(bytes)) {
          console.log('Restored saved garden');
        }
      } catch (e) {
        console.warn('Failed to restore save:', e);
      }
    }
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

  // Spring highlight: gentle blue point light at the water spring
  const springLight = new THREE.PointLight(0x4488cc, 2.0, 15, 1.5);
  springLight.position.set(GRID_X / 2 + 0.5, GROUND_LEVEL + 3, GRID_Y / 2 + 0.5);
  scene.add(springLight);

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

  // --- Garden gnome task queue + ghost overlay ---

  const taskQueue = new TaskQueue();
  const ghosts = new GhostOverlay();
  scene.add(ghosts.group);
  const gardener = new GardenerSprite();
  scene.add(gardener.group);

  // --- Post-processing ---

  const postProcessing = createPostProcessing(renderer, scene, orbit.camera);

  // --- HUD & Controls ---

  const hud = new Hud();
  if (isInitialized()) {
    hud.setTickCount(Number(getTick()));
    hud.setAutoTick(true);
  }

  // New Garden button registered after remeshDirty is defined (see below)
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

  // New Garden button — resets sim, HUD, and re-meshes
  hud.onNewGarden(() => {
    if (!isInitialized()) return;
    resetSim();
    simTick(5);
    hud.resetForNewGarden();
    hud.setTickCount(Number(getTick()));
    _prevStats = { plants: 0, fauna: 0, species: 0 };
    _tipIndex = 0;
    _tipTimer = 0;
    remeshDirty();
    try { localStorage.removeItem('groundwork-garden'); } catch {}
    hud.addEvent('Fresh garden — the spring is flowing');
    hud.addEvent('Click to start planting zones');
  });

  setupControls({
    hud,
    questLog,
    camera: orbit.camera,
    terrainGroup,
    canvas: renderer.domElement,
    onToolPlaced: (hit) => {
      const tool = hud.state.activeTool;
      // Water costs
      const costs: Record<number, number> = {
        [ToolCode.Seed]: 15, [ToolCode.Water]: 20,
        [ToolCode.Soil]: 10, [ToolCode.Shovel]: 5, [ToolCode.Stone]: 10,
      };
      const cost = costs[tool] ?? 10;
      if (!hud.spendWater(cost)) {
        hud.addEvent('Not enough water — wait for the spring to refill');
        return;
      }

      // Queue tasks instead of instant execution — gnome will do the work.
      // Species-aware spacing: trees need territory, groundcover can pack tight.
      if (tool === ToolCode.Seed) {
        const speciesIdx = hud.state.activeSpeciesIndex;
        const speciesType = SPECIES[speciesIdx]?.type ?? 'Ground';
        // Trees: wide spacing (crown_radius ~24 voxels), place 1-2 per click
        // Shrubs: medium spacing, Flowers/Ground: tight packing
        const spacing = speciesType === 'Tree' ? 16
          : speciesType === 'Shrub' ? 8
          : speciesType === 'Flower' ? 4
          : 3;
        const r = speciesType === 'Tree' ? 16
          : speciesType === 'Shrub' ? 8
          : 6;
        for (let dy = -r; dy <= r; dy += spacing) {
          for (let dx = -r; dx <= r; dx += spacing) {
            const sx = hit.x + dx;
            const sy = hit.y + dy;
            if (sx < 0 || sy < 0 || sx >= GRID_X || sy >= GRID_Y) continue;
            taskQueue.enqueue({ tool, x: sx, y: sy, z: hit.z, species: hud.state.activeSpeciesIndex });
          }
        }
      } else {
        const r = 3;
        for (let dy = -r; dy <= r; dy++) {
          for (let dx = -r; dx <= r; dx++) {
            const sx = hit.x + dx;
            const sy = hit.y + dy;
            if (sx < 0 || sy < 0 || sx >= GRID_X || sy >= GRID_Y) continue;
            taskQueue.enqueue({ tool, x: sx, y: sy, z: hit.z });
          }
        }
      }

      // Visual feedback: particle burst + event
      particles.emit(hit.x + 0.5, hit.z + 0.5, hit.y + 0.5);
      const toolNames: Record<number, string> = {
        [ToolCode.Seed]: 'planting', [ToolCode.Water]: 'watering',
        [ToolCode.Shovel]: 'digging', [ToolCode.Soil]: 'soil', [ToolCode.Stone]: 'stone',
      };
      const qLen = taskQueue.length;
      if (qLen > 0) {
        hud.addEvent(`Gnome: ${toolNames[tool] ?? 'working'} zone queued (${qLen} tasks)`);
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
    taskQueue: taskQueue,
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

  // --- Touch controls: 1-finger orbit, 2-finger pinch zoom ---
  let touchStartX = 0;
  let touchStartY = 0;
  let lastPinchDist = 0;

  renderer.domElement.addEventListener('touchstart', (e) => {
    if (e.touches.length === 1) {
      touchStartX = e.touches[0].clientX;
      touchStartY = e.touches[0].clientY;
      isDragging = true;
    } else if (e.touches.length === 2) {
      isDragging = false;
      const dx = e.touches[1].clientX - e.touches[0].clientX;
      const dy = e.touches[1].clientY - e.touches[0].clientY;
      lastPinchDist = Math.sqrt(dx * dx + dy * dy);
    }
  }, { passive: true });

  renderer.domElement.addEventListener('touchmove', (e) => {
    e.preventDefault();
    if (e.touches.length === 1 && isDragging) {
      // 1-finger drag → orbit
      const dx = e.touches[0].clientX - touchStartX;
      const dy = e.touches[0].clientY - touchStartY;
      orbit.rotate(-dx * 0.005, -dy * 0.005);
      touchStartX = e.touches[0].clientX;
      touchStartY = e.touches[0].clientY;
      if (Math.abs(dx) + Math.abs(dy) > 3) questLog.recordOrbit();
    } else if (e.touches.length === 2) {
      // 2-finger pinch → zoom
      const dx = e.touches[1].clientX - e.touches[0].clientX;
      const dy = e.touches[1].clientY - e.touches[0].clientY;
      const pinchDist = Math.sqrt(dx * dx + dy * dy);
      if (lastPinchDist > 0) {
        const scale = pinchDist / lastPinchDist;
        orbit.zoom(scale);
      }
      lastPinchDist = pinchDist;
    }
  }, { passive: false });

  renderer.domElement.addEventListener('touchend', () => {
    isDragging = false;
    lastPinchDist = 0;
  }, { passive: true });

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
  let saveTimer = 0;

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
        // Gnome processes tasks — walks to each one and executes
        // (gnome update happens in the frame loop below)
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

    // Update gnome (walks, works, returns completed tasks)
    const completedTask = gardener.update(dt, elapsed, taskQueue);
    if (completedTask && isInitialized()) {
      if (completedTask.species !== undefined) {
        setSelectedSpecies(completedTask.species);
      }
      placeTool(completedTask.tool, completedTask.x, completedTask.y, completedTask.z);
      particles.emit(completedTask.x + 0.5, completedTask.z + 0.5, completedTask.y + 0.5);
      remeshDirty();
    }

    // Update ghost overlay + gnome status
    ghosts.rebuild(taskQueue, elapsed);
    hud.setQueueCount(taskQueue.length);
    hud.setGnomeStatus(taskQueue.length);


    // Auto-save every ~10 seconds
    saveTimer += dt;
    if (saveTimer > 10 && isInitialized()) {
      saveTimer = 0;
      const gridData = saveGrid();
      if (gridData) {
        try {
          // Encode binary as base64 in chunks (spread operator can't handle 2.5MB)
          let binary = '';
          const chunk = 8192;
          for (let i = 0; i < gridData.length; i += chunk) {
            binary += String.fromCharCode(...gridData.subarray(i, i + chunk));
          }
          localStorage.setItem('groundwork-garden', btoa(binary));
        } catch { /* localStorage full or unavailable */ }
      }
    }

    // Fade quest notifications
    questLog.tickNotification();

    // Animate water ripples + spring pulse
    updateWaterTime(elapsed);
    springLight.intensity = 1.5 + Math.sin(elapsed * 2.0) * 0.8;

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
