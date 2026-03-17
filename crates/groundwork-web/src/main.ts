/**
 * Groundwork Web — main entry point.
 *
 * Initializes the WASM simulation (or mock data), builds the voxel mesh,
 * sets up Three.js scene with lighting and camera, HUD overlay with tool
 * palette and species picker, raycaster click-to-place, and render loop.
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GRID_Z, GROUND_LEVEL, VOXEL_BYTES, Material, ToolCode, SPECIES, FaunaType, FaunaState, GrowthStage, initSim, isInitialized, getGridView, tick as simTick, fillTool, getTick, getFaunaCount, getFaunaView, readFauna, resetSim, saveGrid, restoreGrid, setSelectedSpecies, getMilestones, queueGnomeTask, getGnomeState, getWeatherState, packTreeStats, getTreeStatsView, readTreeStat } from './bridge';
import { CHUNK_SIZE } from './mesher/greedy';
import { SCENES, getSceneId } from './mesher/mockGrid';
import { ChunkManager } from './mesher/chunk';
import { buildChunkMesh, setXrayMode, adjustCutawayDepth, setTerrainDayTint } from './rendering/terrain';
import { buildWaterMesh, updateWaterTime, updateWaterSun, updateWaterRain, updateWaterDayTint, updateWaterNight, scanWaterFrontier } from './rendering/water';
import { FoliageRenderer } from './rendering/foliage';
import { SeedRenderer } from './rendering/seeds';
import { GrowthParticles } from './rendering/particles';
import { RainRenderer } from './rendering/rain';
import { FaunaRenderer } from './rendering/fauna';
import { FireflyRenderer } from './rendering/fireflies';
import { FallingLeaves } from './rendering/leaves';
import { EcologyParticles } from './rendering/ecology';
import { DewRenderer } from './rendering/dew';
import { DustMoteRenderer } from './rendering/dustmotes';
import { GnatRenderer } from './rendering/gnats';
import { MistRenderer } from './rendering/mist';
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
import { raycastVoxel } from './ui/raycaster';
import { initAmbientAudio, setRaining, setNightAmbient, setWindAmbient, setLeafRustle, setPollinatorHum, setFrogChorus, setBeetleClick, setWaterBabble, setGardenDrone, setGardenVitality } from './audio/ambient';
import { playPlant, playDig, playFaunaArrival, playBirdCall, playBirdWarble, playRobinSong, playDistantBird, playBuzz, playSquirrelChitter, playDewDrop, playTreeCreak, playRootCrackle, playRaindropPlink, playGardenWhisper, playGrowth, playDiscovery, playRainStart, playDroughtStart, playWindGust, playWindChime, playGnomeSound, playOwlHoot, playShootingStar } from './audio/sfx';

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
let _particles: GrowthParticles | null = null;
let _squirrelCacheNotified = false;
let _pollinatorActNotified = false;
let _birdDropNotified = false;
let _gardenAliveNotified = false;

/** Species the player has deliberately planted (via seed tool) */
const _playerPlantedSpecies = new Set<number>();
let _xrayTipShown = false;
let _recentDieOff = false;
let _dieOffPlantCount = 0;

/** Previous tree growth stages — keyed by "rootX,rootY" to detect transitions */
const _prevTreeStages = new Map<string, number>();

/** Squirrel trust milestone tracking */
let _prevSquirrelTrust = 0;
const TRUST_MILESTONES: [number, string][] = [
  [50, 'The squirrel is getting curious about your gnome...'],
  [100, 'The squirrel is warming up — it stays closer now'],
  [150, 'The squirrel trusts your gnome! It lingers nearby'],
  [180, 'The squirrel is following your gnome — a loyal companion!'],
];

/** Companion species suggestions — shown once per species per session */
const _companionSuggested = new Set<number>();
const COMPANION_TIPS: Record<number, string> = {
  0: 'Try planting Clover nearby — nitrogen fixing boosts Oak growth 50%!',
  1: 'Birch grows fast in open ground — add Wildflowers to attract pollinators',
  2: 'Willow loves water — plant Moss at its base to hold moisture',
  3: 'Pine acidifies soil — Fern and Moss tolerate it, others struggle nearby',
  4: 'Ferns love shade — plant near a tall tree for the Canopy Effect',
  5: 'Berry bushes attract birds — birds will spread seeds across the garden!',
  7: 'Flowers attract bees and butterflies — cluster them for a pollinator bridge',
  8: 'Daisies attract pollinators — plant near trees to boost their health',
  11: 'Clover fixes nitrogen — plant near trees for a 50% growth boost!',
};

/** Wild plant messages — fauna attribution based on species */
const WILD_PLANT_MESSAGES: Record<number, string[]> = {
  0: [ // Oak — likely squirrel
    'A wild oak seedling appeared — a squirrel must have buried an acorn here!',
    'An oak is growing where you didn\'t plant one — thank the squirrels',
  ],
  5: [ // Berry Bush — likely bird
    'A berry bush appeared in a new spot — a bird must have dropped the seed!',
    'Berry bushes are spreading — birds carry their seeds across the garden',
  ],
};

/** Growth stage transition messages */
const STAGE_MESSAGES: Record<number, string> = {
  [GrowthStage.Sapling]: 'sprouted into a sapling',
  [GrowthStage.YoungTree]: 'is growing tall — branches are spreading',
  [GrowthStage.Mature]: 'has matured — a full canopy!',
  [GrowthStage.OldGrowth]: 'reached old growth — a towering giant',
};

const FAUNA_NAMES: Record<number, string> = {
  0: 'bee', 1: 'butterfly', 2: 'bird', 3: 'worm', 4: 'beetle',
};

/** Fauna arrival messages that teach ecology */
const FAUNA_MESSAGES: Record<number, string[]> = {
  0: [ // Bee
    'A bee arrived — it pollinates flowers AND boosts nearby tree health',
    'Bee spotted! Trees near bees recover health faster',
    'A bee is visiting — plant more flowers to attract more pollinators',
  ],
  1: [ // Butterfly
    'A butterfly appeared — it helps flowers spread and boosts nearby plants',
    'Butterfly visiting! It creates a pollinator bridge between species',
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
  5: [ // Squirrel
    'A squirrel is caching seeds — some may sprout in unexpected places',
    'Squirrel spotted! It buries acorns — watch for surprise oak seedlings',
    'A squirrel is foraging — the gnome can befriend it with patience',
  ],
};

/** Periodic ecology tips — teach interactions through play */
const ECO_TIPS = [
  'Tip: Plant clover near oak — nitrogen fixing boosts growth 50%',
  'Tip: Flowers attract bees — bees boost health of nearby trees',
  'Tip: Press Q for x-ray — see roots competing underground',
  'Tip: Press V to see water, light, or nutrient overlays',
  'Tip: Birds near trees carry seeds to surprising new spots',
  'Tip: Pine roots acidify soil — fern and moss tolerate it, others struggle',
  'Tip: Groundcover enriches soil for everyone nearby',
  'Tip: Blue particles under trees = dappled shade. Ferns and moss love it!',
  'Tip: Crowded plants die — give trees space to claim territory',
  'Tip: The garden gnome does the work — zone areas and watch',
  'Tip: Trees share nutrients through underground mycorrhizal networks',
  'Tip: Dead wood becomes a nurse log — seedlings grow faster on it',
  'Tip: Too many plants? The garden has a carrying capacity — thin for health',
  'Tip: Berry bushes attract birds — birds spread seeds to new areas',
  'Tip: Birch grows fast in open ground — perfect pioneer for bare patches',
  'Tip: Each new species adds +100 to your garden score',
];
let _tipIndex = 0;

/** Pick the most relevant tip based on current garden state */
function getContextualTip(stats: { plants: number; fauna: number; species: number; speciesIds?: Set<number> }): string {
  const ids = stats.speciesIds ?? new Set<number>();
  const hasTree = ids.has(0) || ids.has(1) || ids.has(2) || ids.has(3);   // oak/birch/willow/pine
  const hasFlower = ids.has(7) || ids.has(8);                               // wildflower/daisy
  const hasGroundcover = ids.has(9) || ids.has(10) || ids.has(11);          // moss/grass/clover
  const hasClover = ids.has(11);
  const hasBerryBush = ids.has(5);

  // Prioritized contextual advice
  if (stats.plants < 50)
    return 'Tip: Plant groundcover near the spring — it needs water to germinate';
  if (hasGroundcover && !hasFlower && stats.plants > 200)
    return 'Tip: Plant flowers next — they attract pollinators that boost everyone';
  if (hasFlower && stats.fauna === 0)
    return 'Tip: More flowers = more pollinators. Cluster them together for bees';
  if (stats.fauna > 0 && !hasTree && stats.plants > 500)
    return 'Tip: Your ecosystem earned trees! Plant an oak — it will dominate the garden';
  if (hasTree && !hasClover)
    return 'Tip: Plant clover near your tree — nitrogen fixing boosts growth 50%';
  if (hasTree && hasClover)
    return 'Tip: See green shimmer near your tree? Clover is fixing nitrogen — 50% growth boost!';
  if (hasTree && !hasBerryBush)
    return 'Tip: Berry bushes attract birds — birds carry seeds to surprising new spots';
  if (hasBerryBush && stats.fauna > 2)
    return 'Tip: Watch the blue particles under the canopy — shade-loving plants thrive there';
  if (stats.fauna > 3)
    return 'Tip: Your ecosystem is thriving! Watch how species affect each other';

  // Fallback to random tips
  return ECO_TIPS[_tipIndex++ % ECO_TIPS.length];
}
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
        playFaunaArrival(f.type);
        hud.addEvent(msgs[Math.floor(Math.random() * msgs.length)]);
        // Celebration sparkle burst at the new fauna's position
        if (_particles) {
          // Sim coords → Three.js Y-up (x, z, y)
          _particles.emitFaunaArrival(f.x, f.z, f.y);
        }
      }
    }
    _eventCooldown = 25;
  }

  // New species growing — check if it's pioneer succession or wild plant
  if (stats.species > _prevStats.species && stats.speciesIds) {
    for (const sid of stats.speciesIds) {
      if (!_prevSpeciesIds.has(sid)) {
        const msg = SUCCESSION_MESSAGES[sid];
        if (msg) {
          hud.addEvent(msg);
        } else if (!_playerPlantedSpecies.has(sid) && !PIONEER_SPECIES.has(sid)) {
          // Wild plant — fauna-dispersed!
          const wildMsgs = WILD_PLANT_MESSAGES[sid];
          if (wildMsgs) {
            hud.addEvent(wildMsgs[Math.floor(Math.random() * wildMsgs.length)]);
          } else {
            const name = SPECIES_NAMES[sid] ?? `Species ${sid}`;
            hud.addEvent(`A wild ${name} appeared — the garden is planting itself!`);
          }
          playDiscovery();
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

  // Ecological interactions — detect fauna in Acting state
  const fView2 = getFaunaView?.();
  if (fView2 && stats.fauna > 0) {
    const fCount = Math.min(stats.fauna, 128);
    for (let i = 0; i < fCount; i++) {
      const f = readFauna(fView2, i);
      if (f.state !== FaunaState.Acting) continue;
      if (f.type === FaunaType.Squirrel && !_squirrelCacheNotified) {
        const msgs = [
          'A squirrel is burying an acorn — an oak may sprout here later!',
          'The squirrel is caching seeds — nature plants its own garden',
          'Squirrel at work — it buries acorns that become oak trees',
        ];
        hud.addEvent(msgs[Math.floor(Math.random() * msgs.length)]);
        playDiscovery();
        _squirrelCacheNotified = true;
        _eventCooldown = 40;
        break;
      }
      if (f.type === FaunaType.Bird && !_birdDropNotified) {
        const msgs = [
          'A bird is carrying seeds across the garden — watch for surprises!',
          'Bird at work — it picks up seeds and drops them in new spots',
        ];
        hud.addEvent(msgs[Math.floor(Math.random() * msgs.length)]);
        playDiscovery();
        _birdDropNotified = true;
        _eventCooldown = 40;
        break;
      }
      if ((f.type === FaunaType.Bee || f.type === FaunaType.Butterfly) && !_pollinatorActNotified) {
        const msgs = [
          'A pollinator is visiting a flower — nearby plants get a health boost',
          'Pollination in progress — flowers spread faster near pollinators',
        ];
        hud.addEvent(msgs[Math.floor(Math.random() * msgs.length)]);
        _pollinatorActNotified = true;
        _eventCooldown = 40;
        break;
      }
    }
    // Reset notification flags after cooldown (allow re-notification later)
    if (_eventCooldown <= -100) {
      _squirrelCacheNotified = false;
      _pollinatorActNotified = false;
      _birdDropNotified = false;
    }
  }

  // Major plant growth burst
  if (stats.plants > _prevStats.plants + 500 && _prevStats.plants > 0) {
    hud.addEvent('Growth burst — your garden is flourishing');
    _eventCooldown = 30;
  }

  // "The garden is alive" milestone — one-time celebration when ecosystem is thriving
  if (!_gardenAliveNotified && stats.plants > 1000 && stats.fauna >= 5 && stats.species >= 3) {
    hud.addEvent('The garden is alive \u2014 an ecosystem hums with interconnected life');
    playDiscovery();
    // Emit a large celebration burst from the center
    if (_particles) {
      for (let i = 0; i < 5; i++) {
        const cx = GRID_X / 2 + (Math.random() - 0.5) * 20;
        const cz = GRID_Y / 2 + (Math.random() - 0.5) * 20;
        _particles.emitFaunaArrival(cx, GROUND_LEVEL + 3, cz);
      }
    }
    _gardenAliveNotified = true;
    _eventCooldown = 50;
  }

  // Plant die-off from competition
  if (_prevStats.plants > 100 && stats.plants < _prevStats.plants - 200) {
    const msgs = [
      'A plant died from competition — the strongest survive',
      'Overcrowding! Plants are competing for water and light',
      'Natural selection at work — the garden finds its balance',
    ];
    hud.addEvent(msgs[Math.floor(Math.random() * msgs.length)]);
    _recentDieOff = true;
    _dieOffPlantCount = stats.plants;
    _eventCooldown = 40;
  }

  // Recovery after die-off — pioneer species recolonize
  if (_recentDieOff && stats.plants > _dieOffPlantCount + 100) {
    const msgs = [
      'The garden is recovering — pioneer species are recolonizing!',
      'Life finds a way — new growth is filling the gaps',
      'Recovery in progress — the ecosystem is resilient',
    ];
    hud.addEvent(msgs[Math.floor(Math.random() * msgs.length)]);
    _recentDieOff = false;
    _eventCooldown = 30;
  }

  // Contextual ecology tips — suggest next discovery based on garden state
  _tipTimer++;
  if (_tipTimer > 60 && _eventCooldown <= 0) {
    const tip = getContextualTip(stats);
    hud.addEvent(tip);
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

  // Mobile detection: touch-primary devices or narrow viewports
  const isMobile = navigator.maxTouchPoints > 0 && window.innerWidth < 1024;

  const renderer = new THREE.WebGLRenderer({ antialias: !isMobile, preserveDrawingBuffer: true });
  // Clamp DPR to 2 — 3x devices render 9x pixels for negligible visual gain
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
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
  const orbit = new OrbitCamera(window.innerWidth / window.innerHeight, { mobile: isMobile });

  // Lighting
  const lights = createLighting(scene, { mobile: isMobile });

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

  /** Toggle x-ray on/off — single source of truth for all side effects */
  function setXray(active: boolean): void {
    xrayActive = active;
    setXrayMode(active);
    postProcessing.setDesaturation(active ? 1.0 : 0.0);
    hud.setXrayUI(active);
    // Hide foliage + fauna in x-ray — just terrain, roots, and data
    foliage.group.visible = !active;
    fauna.group.visible = !active;
    if (!active) overlay.setMode(OverlayMode.Off);
  }

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
  _particles = particles;
  scene.add(particles.points);
  // Initial detection pass (no bursts on first load)
  particles.detectGrowth(grid);

  // --- Rain particles ---

  const rain = new RainRenderer();
  rain.onSplash((x, y, z) => {
    particles.emitRainSplash(x, y, z);
    // ~10% of splashes get an audible plink — ambient not overwhelming
    if (Math.random() < 0.1) playRaindropPlink();
  });
  scene.add(rain.points);

  // --- Ecology interaction particles ---

  const ecology = new EcologyParticles({ mobile: isMobile });
  scene.add(ecology.points);

  // --- Fireflies (dusk/night ambient) ---

  const fireflies = new FireflyRenderer();
  scene.add(fireflies.group);

  // --- Morning dew (dawn ambient) ---

  const dew = new DewRenderer();
  scene.add(dew.group);

  // --- Dawn mist ---

  const mist = new MistRenderer();
  scene.add(mist.group);

  // --- Dust motes (midday ambient) ---

  const dustMotes = new DustMoteRenderer();
  scene.add(dustMotes.group);

  // --- Ambient gnats (swarming over vegetation) ---

  const gnats = new GnatRenderer();
  scene.add(gnats.group);

  // --- Falling leaves ---

  const fallingLeaves = new FallingLeaves();
  scene.add(fallingLeaves.group);

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

  const postProcessing = createPostProcessing(renderer, scene, orbit.camera, { mobile: isMobile });

  // --- HUD & Controls ---

  const hud = new Hud();
  if (isInitialized()) {
    hud.setTickCount(Number(getTick()));
    hud.setAutoTick(true);
  }

  // Ambient audio — procedural water spring sound, fades in on first interaction
  initAmbientAudio();

  // New Garden button registered after remeshDirty is defined (see below)
  const questLog = new QuestLog();

  // Progressive UI reveal: show HUD elements as player advances through chapters
  questLog.onChapterChange((chapter) => {
    // Phase 0 is controlled by auto-advance below, not quest chapters
    if (chapter >= 1) {
      hud.setPhase(Math.max(chapter, 1));
    }
    if (chapter === 1) {
      hud.selectTool(ToolCode.Seed);
    }
    if (chapter === 2) {
      // Shovel unlocks for irrigation — select it so the player sees it
      hud.selectTool(ToolCode.Shovel);
      hud.addEvent('Shovel unlocked — dig channels from the spring to irrigate!');
    }
  });

  // Phase 0 → 1 auto-advance: after 3 seconds OR first mouse interaction,
  // show just the "sow" tool. The player meets the gnome and pond first.
  let phase0Advanced = false;
  const advanceFromPhase0 = () => {
    if (phase0Advanced) return;
    phase0Advanced = true;
    hud.setPhase(1);
    hud.selectTool(ToolCode.Seed);
  };
  setTimeout(advanceFromPhase0, 3000);
  renderer.domElement.addEventListener('pointerdown', advanceFromPhase0, { once: true });

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

  /** Re-mesh dirty chunks, re-fetching the grid view if using WASM.
   *  On mobile, budgets to 4 chunks per call to avoid frame stutter. */
  function remeshDirty(): void {
    const freshGrid = isInitialized() ? getGridView() : grid;
    chunkManager.detectChanges(freshGrid);
    const rebuilt = chunkManager.rebuildDirty(freshGrid, isMobile ? 4 : undefined);
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

    // Detect tree growth stage transitions → celebratory burst + HUD message
    const treeCount = packTreeStats();
    const treeView = getTreeStatsView();
    if (treeView) {
      for (let i = 0; i < treeCount; i++) {
        const t = readTreeStat(treeView, i);
        const key = `${t.rootX},${t.rootY}`;
        const prevStage = _prevTreeStages.get(key);
        if (prevStage !== undefined && t.stage > prevStage && t.stage !== GrowthStage.Dead) {
          // Sim coords (x,y) → Three.js (x, GROUND_LEVEL, y)
          particles.emitStageBurst(t.rootX + 0.5, GROUND_LEVEL + 2, t.rootY + 0.5);
          const name = SPECIES[t.speciesId]?.name ?? 'A tree';
          const msg = STAGE_MESSAGES[t.stage];
          if (msg) hud.addEvent(`${name} ${msg}`);
          playTreeCreak();
          if (t.stage >= GrowthStage.Mature) playDiscovery();
        }
        _prevTreeStages.set(key, t.stage);
      }
    }

    // Growth sound: soft shimmer when vegetation increases noticeably
    if (growthSoundCooldown > 0) growthSoundCooldown--;
    const plantCount = foliage.count;
    if (plantCount > prevPlantCount + 30 && growthSoundCooldown <= 0) {
      playGrowth();
      growthSoundCooldown = 15; // ~15 ticks cooldown (~1.5s at 100ms/tick)
    }
    prevPlantCount = plantCount;

    // Water flow detection: emit blue sparkle particles at expanding water edges
    const { count: waterCount, frontier } = scanWaterFrontier(freshGrid);
    if (waterCount > prevWaterCount && frontier.length > 0) {
      particles.emitWaterFlow(frontier);
    }
    prevWaterCount = waterCount;

    // Root growth crackle: subtle underground sound when roots expand
    if (particles.rootGrowthDelta > 5 && Math.random() < 0.3) {
      playRootCrackle();
    }

    // Decomposition fungi spores: slow particles near dead wood
    if (particles.hasDeadWood && Math.random() < 0.15) {
      particles.emitFungiSpore();
    }
  }

  // New Garden button — resets sim, HUD, and re-meshes
  // X-ray toggle from the HUD button (works on mobile + desktop)
  hud.onXrayToggle((active: boolean) => {
    setXray(active);
    questLog.recordDepthChange();
  });

  // X-ray lens picker: switch data visualization when user picks a lens
  hud.onLensChange((lens: string) => {
    const freshGrid = isInitialized() ? getGridView() : grid;
    switch (lens) {
      case 'roots':
        // Roots lens: transparent soil shows root networks (default x-ray)
        overlay.setMode(OverlayMode.Off);
        setXrayMode(true);
        break;
      case 'moisture':
        overlay.setMode(OverlayMode.Water);
        overlay.rebuild(freshGrid);
        break;
      case 'light':
        overlay.setMode(OverlayMode.Light);
        overlay.rebuild(freshGrid);
        break;
      case 'nutrients':
        overlay.setMode(OverlayMode.Nutrient);
        overlay.rebuild(freshGrid);
        break;
    }
  });

  hud.onNewGarden(() => {
    if (!isInitialized()) return;
    resetSim();
    simTick(5);
    hud.resetForNewGarden();
    hud.setTickCount(Number(getTick()));
    _prevStats = { plants: 0, fauna: 0, species: 0 };
    _prevSpeciesIds = new Set<number>();
    _prevTreeStages.clear();
    _prevSquirrelTrust = 0;
    _playerPlantedSpecies.clear();
    _companionSuggested.clear();
    _squirrelCacheNotified = false;
    _pollinatorActNotified = false;
    _birdDropNotified = false;
    _xrayTipShown = false;
    _recentDieOff = false;
    _tipIndex = 0;
    _tipTimer = 0;
    taskQueue.clear();
    questLog.reset();
    remeshDirty();
    try { localStorage.removeItem('groundwork-garden'); } catch {}
    // Re-arm phase 0→1 auto-advance
    phase0Advanced = false;
    setTimeout(advanceFromPhase0, 3000);
  });

  /** Enqueue a task to the WASM sim gnome (single source of truth).
   *  JS taskQueue is kept in sync for ghost overlay rendering. */
  function enqueueTask(tool: number, x: number, y: number, z: number, species?: number) {
    taskQueue.enqueue({ tool, x, y, z, species });
    if (isInitialized()) {
      queueGnomeTask(tool, x, y, z, species ?? 255);
    }
  }

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
        [ToolCode.Seed]: 15,
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
            enqueueTask(tool, sx, sy, hit.z, hud.state.activeSpeciesIndex);
          }
        }
      } else {
        const r = 3;
        for (let dy = -r; dy <= r; dy++) {
          for (let dx = -r; dx <= r; dx++) {
            const sx = hit.x + dx;
            const sy = hit.y + dy;
            if (sx < 0 || sy < 0 || sx >= GRID_X || sy >= GRID_Y) continue;
            enqueueTask(tool, sx, sy, hit.z);
          }
        }
      }

      // Track player-planted species for wild plant detection
      if (tool === ToolCode.Seed) {
        _playerPlantedSpecies.add(hud.state.activeSpeciesIndex);
      }

      // Visual + audio feedback
      particles.emit(hit.x + 0.5, hit.z + 0.5, hit.y + 0.5);
      if (tool === ToolCode.Seed) playPlant();
      else if (tool === ToolCode.Shovel) playDig();
      else playPlant(); // soil/stone use plant sound
      const toolNames: Record<number, string> = {
        [ToolCode.Seed]: 'planting',
        [ToolCode.Shovel]: 'digging', [ToolCode.Soil]: 'soil', [ToolCode.Stone]: 'stone',
      };
      const qLen = taskQueue.length;
      if (qLen > 0) {
        hud.addEvent(`Gnome: ${toolNames[tool] ?? 'working'} zone queued (${qLen} tasks)`);
      }
      // Companion species suggestion — once per species
      if (tool === ToolCode.Seed) {
        const sid = hud.state.activeSpeciesIndex;
        if (!_companionSuggested.has(sid) && COMPANION_TIPS[sid]) {
          setTimeout(() => hud.addEvent(COMPANION_TIPS[sid]), 2000);
          _companionSuggested.add(sid);
        }
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
    onZoneCommit: (start, end) => {
      // Drag-to-zone: queue all voxels in the rectangle
      const tool = hud.state.activeTool;
      const cost = tool === ToolCode.Seed ? 15 : tool === ToolCode.Water ? 20 : 10;
      if (!hud.spendWater(cost)) {
        hud.addEvent('Not enough water');
        return;
      }
      const x1 = Math.min(start.x, end.x);
      const x2 = Math.max(start.x, end.x);
      const y1 = Math.min(start.y, end.y);
      const y2 = Math.max(start.y, end.y);
      const z = start.z;
      const spacing = tool === ToolCode.Seed ? 4 : 1;
      for (let y = y1; y <= y2; y += spacing) {
        for (let x = x1; x <= x2; x += spacing) {
          if (x >= 0 && y >= 0 && x < GRID_X && y < GRID_Y) {
            enqueueTask(tool, x, y, z, tool === ToolCode.Seed ? hud.state.activeSpeciesIndex : undefined);
          }
        }
      }
      if (tool === ToolCode.Seed) {
        _playerPlantedSpecies.add(hud.state.activeSpeciesIndex);
      }
      particles.emit((x1 + x2) / 2 + 0.5, z + 0.5, (y1 + y2) / 2 + 0.5);
      if (tool === ToolCode.Seed) playPlant();
      else playDig();
      hud.addEvent(`Zone painted: ${(x2 - x1 + 1)}×${(y2 - y1 + 1)} area queued`);
    },
  });

  // --- Agent API (for Playwright screenshot harness) ---

  initAgentAPI({
    orbitCamera: orbit,
    remeshDirty: remeshDirty,
    dayCycle: dayCycle,
    setXrayMode: (active: boolean) => {
      setXray(active);
    },
    setTickCount: (count: number) => hud.setTickCount(count),
    overlay: overlay,
    taskQueue: taskQueue,
    scenery: { forestRing, skirt, scene },
  });

  // --- Sim state ---

  let autoTick = wasmReady; // Auto-tick ON by default when sim is ready
  let tickAccumulator = 0;
  let tickSpeed = 1; // 1x, 2x, 5x
  let prevWeatherState = 0; // 0=Clear, 1=Rain, 2=Drought
  let droughtStress = 0; // 0-1 smooth drought foliage yellowing
  let gustTimer = 8 + Math.random() * 12; // seconds until next wind gust
  let gustStrength = 0; // 0-1 current gust intensity (decays after gust)
  let windAngle = Math.random() * Math.PI * 2; // slowly drifting wind direction
  let ambientSoundTimer = 0; // seconds until next ambient fauna sound
  let growthSoundCooldown = 0; // ticks until next growth sound can play
  let prevPlantCount = 0; // for detecting growth bursts
  let prevWaterCount = 0; // for detecting water expansion (channel filling)
  let dustPuffTimer = 0; // seconds until next gnome footstep dust puff
  let prevGnomeState = -1; // for detecting gnome state transitions
  let owlHootTimer = 30 + Math.random() * 30; // seconds until next owl hoot
  let gardenWhisperTimer = 20 + Math.random() * 20; // seconds until next garden whisper
  let activePollinators = 0; // cross-block pollinator count for sunbeam pollen effect
  let leafDripTimer = 0; // remaining seconds of post-rain leaf dripping
  let leafDripInterval = 0; // accumulator for drip emit rate
  let prevShootingStarSlot = -1; // for detecting shooting star events (shader fires every ~45s)
  let nextAgeMilestone = 1000; // next tick count to celebrate
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
    orbit.resetIdle();
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
  renderer.domElement.addEventListener('mouseleave', () => {
    isDragging = false;
    hoverTooltip.style.display = 'none';
  });

  // --- Plant hover tooltip ---
  const hoverTooltip = document.createElement('div');
  hoverTooltip.style.cssText =
    'position:fixed;pointer-events:none;padding:3px 8px;background:rgba(30,25,20,0.85);' +
    'color:#e8dcc8;font-size:12px;border-radius:4px;display:none;z-index:100;' +
    'font-family:system-ui,sans-serif;white-space:nowrap;';
  document.body.appendChild(hoverTooltip);

  const MATERIAL_NAMES: Record<number, string> = {
    [Material.Trunk]: 'Trunk', [Material.Branch]: 'Branch',
    [Material.Leaf]: 'Leaf', [Material.Root]: 'Root',
    [Material.Seed]: 'Seed', [Material.DeadWood]: 'Dead Wood',
  };
  const PLANT_MATERIALS = new Set([Material.Trunk, Material.Branch, Material.Leaf, Material.Root, Material.Seed]);

  renderer.domElement.addEventListener('mousemove', (e) => {
    if (isDragging) {
      hoverTooltip.style.display = 'none';
      return;
    }
    const hit = raycastVoxel(e.clientX, e.clientY, orbit.camera, terrainGroup, false);
    if (!hit) { hoverTooltip.style.display = 'none'; return; }

    const grid = getGridView();
    if (!grid) { hoverTooltip.style.display = 'none'; return; }

    const idx = (hit.x + hit.y * GRID_X + hit.z * GRID_X * GRID_Y) * VOXEL_BYTES;
    const mat = grid[idx];

    if (!PLANT_MATERIALS.has(mat)) {
      hoverTooltip.style.display = 'none';
      return;
    }

    const speciesId = grid[idx + 3]; // byte 3 = species_id for plant voxels
    const speciesName = SPECIES_NAMES[speciesId] ?? `Species ${speciesId}`;
    const matName = MATERIAL_NAMES[mat] ?? '';

    hoverTooltip.textContent = `${speciesName} ${matName}`.trim();
    hoverTooltip.style.display = 'block';
    hoverTooltip.style.left = `${e.clientX + 12}px`;
    hoverTooltip.style.top = `${e.clientY - 20}px`;
  });

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
        // Toggle x-ray: greyscale, hide leaves/fauna, show lens picker
        setXray(!xrayActive);
        questLog.recordDepthChange();
        if (xrayActive && !_xrayTipShown) {
          hud.addEvent('X-ray mode — pick a lens to inspect your garden');
          _xrayTipShown = true;
        }
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

  // --- Welcome message ---
  // Delayed so the garden is visible before text appears
  setTimeout(() => {
    hud.addEvent('Welcome to your garden — the spring is flowing');
  }, 1500);
  setTimeout(() => {
    hud.addEvent('Click anywhere to zone an area for planting');
  }, 4000);

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
        const currentTick = Number(getTick());
        hud.setTickCount(currentTick);
        // Garden age milestones
        if (currentTick >= nextAgeMilestone) {
          const messages: Record<number, string> = {
            1000: 'Your garden has lived 1,000 moments \u2014 roots are taking hold',
            5000: '5,000 moments \u2014 this is a living ecosystem now',
            10000: '10,000 moments \u2014 your garden tells its own story',
            25000: '25,000 moments \u2014 ancient trees, deep roots, a world complete',
          };
          const msg = messages[nextAgeMilestone];
          if (msg) {
            hud.addEvent(msg);
            playDiscovery();
          }
          // Schedule next milestone
          if (nextAgeMilestone < 5000) nextAgeMilestone = 5000;
          else if (nextAgeMilestone < 10000) nextAgeMilestone = 10000;
          else if (nextAgeMilestone < 25000) nextAgeMilestone = 25000;
          else nextAgeMilestone = nextAgeMilestone + 25000;
        }
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
        fallingLeaves.setTreeSpecies(Array.from(stats.speciesIds));
        // Update species unlocks from sim-side ecological milestones
        const milestones = getMilestones();
        if (milestones) hud.updateMilestones(milestones);
        // Weather transition events
        const newWeather = getWeatherState();
        if (newWeather !== prevWeatherState) {
          if (newWeather === 1) { hud.addEvent('Rain begins \u2014 the garden drinks deeply'); playRainStart(); }
          else if (newWeather === 2) { hud.addEvent('Drought \u2014 water runs low, roots dig deep'); playDroughtStart(); }
          else if (prevWeatherState === 1) { hud.addEvent('The rain passes \u2014 skies clear'); leafDripTimer = 30 + Math.random() * 30; }
          else if (prevWeatherState === 2) hud.addEvent('Drought breaks \u2014 the soil can breathe');
          prevWeatherState = newWeather;
        }
      }
      remeshDirty();
    }

    // Sync gnome from sim (single source of truth for position + state)
    if (isInitialized()) {
      const gnomeSim = getGnomeState();
      if (gnomeSim) {
        const taskCompleted = gardener.syncFromSim(gnomeSim, dt, elapsed);
        if (taskCompleted) {
          // Dequeue from JS taskQueue to keep ghost overlay in sync
          const task = taskQueue.dequeue();
          if (task) {
            particles.emit(task.x + 0.5, task.z + 0.5, task.y + 0.5);
          }
          remeshDirty();
        }

        // Gnome state transition sounds
        if (gnomeSim.state !== prevGnomeState) {
          playGnomeSound(gnomeSim.state);
          prevGnomeState = gnomeSim.state;
        }

        // Gnome footstep dust puffs when walking/wandering/working
        dustPuffTimer -= dt;
        if (dustPuffTimer <= 0) {
          if (gnomeSim.state === 1 || gnomeSim.state === 5) {
            // Walking/wandering: light footstep dust
            particles.emitDustPuff(gnomeSim.x + 0.5, gnomeSim.z + 0.5, gnomeSim.y + 0.5);
            dustPuffTimer = 0.25 + Math.random() * 0.15;
          } else if (gnomeSim.state === 2) {
            // Working: heavier soil burst at feet
            particles.emitDustPuff(gnomeSim.x + 0.5, gnomeSim.z + 0.5, gnomeSim.y + 0.5);
            particles.emitDustPuff(gnomeSim.x + 0.5, gnomeSim.z + 0.5, gnomeSim.y + 0.5);
            dustPuffTimer = 0.15 + Math.random() * 0.1; // more frequent when digging
          }
        }

        // Squirrel trust milestone messages
        if (gnomeSim.squirrelTrust > _prevSquirrelTrust) {
          for (const [threshold, msg] of TRUST_MILESTONES) {
            if (_prevSquirrelTrust < threshold && gnomeSim.squirrelTrust >= threshold) {
              hud.addEvent(msg);
              if (threshold >= 180) playDiscovery();
            }
          }
          _prevSquirrelTrust = gnomeSim.squirrelTrust;
        }
      }
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
          hud.showSaveIndicator();
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

    // Drought visual: warmer, hazier atmosphere + foliage stress
    if (isInitialized()) {
      const ws = getWeatherState();
      if (ws === 2) {
        if (scene.fog instanceof THREE.FogExp2) {
          // Push fog toward warm amber and slightly thicker
          scene.fog.color.lerp(new THREE.Color(0.65, 0.50, 0.35), 0.02);
          scene.fog.density = Math.min(scene.fog.density * 1.001, 0.004);
        }
        // Foliage stress: gradually yellow during drought
        droughtStress = Math.min(droughtStress + dt * 0.1, 1.0);
      } else {
        // Recovery: foliage greens up when drought ends
        droughtStress = Math.max(droughtStress - dt * 0.2, 0.0);
      }
      foliage.setDroughtStress(droughtStress);
    }

    // Animate foliage wind sway + day-night color tinting
    foliage.update(elapsed);
    foliage.setDayTint(dayCycle.getTime());

    // Terrain day-night tint (reuse foliage tint values — same color temperature)
    {
      const t = dayCycle.getTime();
      let r = 1, g = 1, b = 1;
      if (t < 0.2) {
        const f = t / 0.2;
        r = 0.7 + f * 0.3; g = 0.72 + f * 0.25; b = 0.85 - f * 0.1;
      } else if (t < 0.35) {
        const f = (t - 0.2) / 0.15;
        r = 1.0; g = 0.97 - f * 0.02; b = 0.75 + f * 0.25;
      } else if (t < 0.65) {
        r = 1.0; g = 0.98; b = 1.0;
      } else if (t < 0.8) {
        const f = (t - 0.65) / 0.15;
        r = 1.0 + f * 0.05; g = 0.95 - f * 0.08; b = 0.95 - f * 0.2;
      } else {
        const f = (t - 0.8) / 0.2;
        r = 1.05 - f * 0.4; g = 0.87 - f * 0.17; b = 0.75 + f * 0.1;
      }
      setTerrainDayTint(r, g, b);
      updateWaterDayTint(r, g, b);
      // Star reflections on water: match sky night amount
      const nightAmount = t >= 0.8 ? (t - 0.8) / 0.15
        : t <= 0.15 ? 1.0
        : t <= 0.3 ? 1.0 - (t - 0.15) / 0.15
        : 0;
      updateWaterNight(Math.max(0, Math.min(1, nightAmount)));

      // Bloom boost: sunrise (0.22-0.28), golden hour (0.65-0.80), sunset (0.78-0.85)
      {
        let bloom = 0.25; // base
        // Golden hour: strongest bloom
        if (t >= 0.65 && t < 0.80) {
          const f = (t - 0.65) / 0.15;
          bloom += Math.sin(f * Math.PI) * 0.20;
        }
        // Sunrise flash: brief vivid pulse as sun crests horizon
        if (t >= 0.22 && t < 0.28) {
          const f = (t - 0.22) / 0.06;
          bloom += Math.sin(f * Math.PI) * 0.15;
        }
        // Sunset glow: warm pulse as sun drops below
        if (t >= 0.78 && t < 0.85) {
          const f = (t - 0.78) / 0.07;
          bloom += Math.sin(f * Math.PI) * 0.12;
        }
        postProcessing.setBloomStrength(bloom);
      }

      // Heat shimmer: active during drought or hot midday (0.4-0.6)
      {
        let shimmer = 0;
        if (prevWeatherState === 2) {
          // Drought: steady shimmer
          shimmer = 0.003;
        } else if (t >= 0.4 && t <= 0.6) {
          // Hot midday: gentle shimmer peaks at noon (0.5)
          const noon = 1.0 - Math.abs(t - 0.5) / 0.1;
          shimmer = Math.max(0, noon) * 0.0015;
        }
        postProcessing.setHeatShimmer(shimmer, elapsed);
      }

      // Fog color follows time of day (unless drought overrides)
      if (scene.fog instanceof THREE.FogExp2 && prevWeatherState !== 2) {
        // Target fog color by time of day
        const fogR = r * 0.8;
        const fogG = g * 0.82;
        const fogB = b * 0.85;
        scene.fog.color.lerp(new THREE.Color(fogR, fogG, fogB), 0.02);
      }
    }

    // Update fauna positions and animation
    fauna.update(elapsed);

    // Fireflies + cricket sounds: active during dusk/night
    const dayTime = dayCycle.getTime();
    fireflies.setActive(dayTime);
    fireflies.update(dt, elapsed);
    // Firefly water reflections + moth flutter: nighttime light-drawn effects
    {
      const litFlies = fireflies.getActivePositions();
      if (litFlies.length > 0) {
        // Water reflections
        if (prevWaterCount > 0 && Math.random() < dt * 0.8) {
          const pick = litFlies[Math.floor(Math.random() * litFlies.length)];
          if (pick.z <= GROUND_LEVEL + 6) {
            particles.emitFireflyReflection(pick.x, pick.y);
          }
        }
        // Moths drawn to firefly light
        if (Math.random() < dt * 0.3) {
          const pick = litFlies[Math.floor(Math.random() * litFlies.length)];
          particles.emitMothFlutter(pick.x, pick.z, pick.y); // sim x,z → Three.js x,y; sim y → Three.js z
        }
      }
    }
    dew.setActive(dayTime);
    dew.update(dt, elapsed);
    // Dew drop tinkle: occasional audio complement to visual dew sparkles
    if (dayTime >= 0.15 && dayTime <= 0.35 && Math.random() < dt * 0.4) {
      playDewDrop();
    }
    mist.setActive(dayTime);
    mist.setWaterInfluence(prevWaterCount);
    mist.update(dt);
    dustMotes.setActive(dayTime);
    dustMotes.update(dt, elapsed);
    gnats.setActive(dayTime);
    gnats.setFoliageCount(foliage.count);
    gnats.update(dt, elapsed);
    setNightAmbient(dayTime);
    setFrogChorus(prevWaterCount, dayTime);
    setWaterBabble(prevWaterCount);
    setGardenDrone(foliage.count, dayTime);
    {
      const fc = getFaunaCount();
      setGardenVitality(foliage.count, fc);
      // Ecosystem health → warmth glow: plants, fauna, and species diversity
      const plantHealth = Math.min(1, foliage.count / 1000);
      const faunaHealth = Math.min(1, fc / 8);
      const ecoWarmth = (plantHealth * 0.5 + faunaHealth * 0.5);
      postProcessing.setEcoWarmth(ecoWarmth);
    }

    // Owl hoot during deep night
    const isDeepNight = dayTime >= 0.80 || dayTime < 0.10;
    if (isDeepNight) {
      owlHootTimer -= dt;
      if (owlHootTimer <= 0) {
        playOwlHoot();
        owlHootTimer = 30 + Math.random() * 40;
      }
    }

    // Garden whisper: soft harmonic murmur when ecosystem is diverse (6+ species)
    if (foliage.count > 500) {
      gardenWhisperTimer -= dt;
      if (gardenWhisperTimer <= 0) {
        playGardenWhisper();
        gardenWhisperTimer = 25 + Math.random() * 35;
      }
    }

    // Shooting star sound: matches shader event timing (every ~45s, night only)
    const starSlot = Math.floor(elapsed / 45);
    if (starSlot !== prevShootingStarSlot) {
      prevShootingStarSlot = starSlot;
      if (isDeepNight) playShootingStar();
    }

    // Falling leaves: ambient canopy motion
    fallingLeaves.setPlantCount(foliage.count);
    fallingLeaves.setWind(foliage.getWindStrength());
    fallingLeaves.update(dt, elapsed);

    // Ambient fauna sounds: periodic bird chirps and bee buzzes when present
    ambientSoundTimer -= dt;
    if (ambientSoundTimer <= 0 && isInitialized()) {
      // Dawn chorus: birds sing much more frequently around dawn (0.2-0.3)
      const dayTime = dayCycle.getTime();
      const isDawn = dayTime >= 0.2 && dayTime <= 0.3;
      const faunaCount = getFaunaCount();
      // More fauna = more frequent sounds (thriving garden sounds different)
      const faunaFactor = Math.max(0.3, 1.0 - faunaCount * 0.05); // 0.3-1.0
      ambientSoundTimer = isDawn
        ? (2 + Math.random() * 4) * faunaFactor   // 0.6-6s at dawn
        : (8 + Math.random() * 12) * faunaFactor;  // 2.4-20s normally
      if (faunaCount > 0) {
        const fView = getFaunaView();
        if (fView) {
          // Pick a random fauna and play its sound
          const idx = Math.floor(Math.random() * faunaCount);
          const f = readFauna(fView, idx);
          if (f.type === 2 || isDawn) {
            // Dawn chorus: varied bird songs; otherwise standard chirp
            const r = Math.random();
            if (isDawn) {
              if (r < 0.35) playBirdCall();
              else if (r < 0.65) playBirdWarble();
              else playRobinSong();
              // Layer 1-2 distant birds for full chorus depth
              playDistantBird();
              if (Math.random() < 0.5) playDistantBird();
            } else {
              playBirdCall();
            }
          } else if (f.type <= 1) {
            playBuzz(); // Bee/butterfly (not at dawn)
          } else if (f.type === FaunaType.Squirrel) {
            playSquirrelChitter();
          }
        }
      }
    }

    // Continuous pollinator hum: scales with active bee/butterfly count
    if (isInitialized()) {
      const fc = getFaunaCount();
      const fv = getFaunaView();
      let pollinators = 0;
      let beetles = 0;
      if (fv) {
        for (let i = 0; i < fc; i++) {
          const f = readFauna(fv, i);
          if (f.type === FaunaType.Bee || f.type === FaunaType.Butterfly) {
            pollinators++;
            // Bee waggle dance: golden figure-8 particles when actively pollinating
            if (f.type === FaunaType.Bee && f.state === FaunaState.Acting
                && Math.random() < dt * 1.2) {
              particles.emitBeeWaggle(f.x, f.y); // sim x,y → world x,z
            }
            // Butterfly pollen trail: soft yellow motes drift from wings while active
            if (f.type === FaunaType.Butterfly
                && (f.state === FaunaState.Seeking || f.state === FaunaState.Acting)
                && Math.random() < dt * 0.6) {
              particles.emitButterflyPollen(f.x, f.y);
            }
          } else if (f.type === FaunaType.Beetle) {
            beetles++;
            // Beetle trail shimmer: faint iridescent particles showing decomposition path
            if ((f.state === FaunaState.Seeking || f.state === FaunaState.Acting)
                && Math.random() < dt * 0.4) {
              particles.emitBeetleTrail(f.x, f.y);
            }
          }
          // Worm soil disturbance: tiny earthy puffs when active underground
          if (f.type === FaunaType.Worm && (f.state === FaunaState.Seeking || f.state === FaunaState.Acting)
              && Math.random() < dt * 0.5) {
            particles.emitWormTrail(f.x, f.y); // sim x,y → world x,z
          }
          // Bird nesting: tiny twig particles tumble from perched birds
          if (f.type === FaunaType.Bird && f.state === FaunaState.Idle
              && Math.random() < dt * 0.3) {
            particles.emitBirdNesting(f.x, f.y);
          }
          // Squirrel footprint dust: tiny puffs when scurrying
          if (f.type === FaunaType.Squirrel && f.state === FaunaState.Seeking
              && Math.random() < dt * 0.8) {
            particles.emitSquirrelPrints(f.x, f.y);
          }
          // Flying fauna over water → ripple (low probability per frame to keep it subtle)
          if ((f.type === FaunaType.Bee || f.type === FaunaType.Butterfly || f.type === FaunaType.Bird)
              && prevWaterCount > 0 && Math.random() < dt * 0.15) {
            // Check if fauna is near water surface level
            const faunaWorldY = f.z; // sim z → height
            if (faunaWorldY >= GROUND_LEVEL && faunaWorldY <= GROUND_LEVEL + 5) {
              particles.emitWaterRipple(f.x, GROUND_LEVEL + 0.2, f.y);
            }
          }
        }
      }
      activePollinators = pollinators;
      setPollinatorHum(pollinators);
      setBeetleClick(beetles, dayTime);
    }

    // Dawn soil steam: warm wisps rise from bare ground as morning sun heats it
    const dawnSteam = dayTime >= 0.22 && dayTime <= 0.32;
    if (dawnSteam && Math.random() < dt * 3) {
      // Random ground position in the garden
      const sx = 10 + Math.random() * (GRID_X - 20);
      const sz = 10 + Math.random() * (GRID_Y - 20);
      particles.emitSoilSteam(sx, sz);
    }

    // Sunbeam shafts: golden particles stream through canopy during bright daylight
    // Pollinators add visible pollen — more golden, denser, slightly driftier
    if (dayTime >= 0.3 && dayTime <= 0.65 && foliage.count > 500) {
      const pollenAmount = Math.min(1, activePollinators / 4); // 0-1 based on pollinator count
      const beamRate = 1.5 + pollenAmount * 2.0; // up to 3.5/sec with heavy pollen
      if (Math.random() < dt * beamRate) {
        const sx = 15 + Math.random() * (GRID_X - 30);
        const sz = 15 + Math.random() * (GRID_Y - 30);
        particles.emitSunbeam(sx, sz, pollenAmount);
      }
    }

    // Camera rustle: fast panning scatters leaf fragments from nearby canopy
    const panSpeed = orbit.getPanSpeed();
    if (panSpeed > 5 && foliage.count > 50 && Math.random() < dt * panSpeed * 0.1) {
      const c = orbit.getCenter();
      particles.emitCameraRustle(c.x, c.y, c.z);
    }

    // Animate growth particles
    particles.update(dt);

    // Rain + weather-driven wind: sync with sim weather state
    if (isInitialized()) {
      const weatherState = getWeatherState();
      rain.setActive(weatherState === 1); // 1 = Rain
      setRaining(weatherState === 1);    // rain audio
      updateWaterRain(rain.getIntensity());

      // Post-rain leaf drip: water drops fall from foliage for 30-60s after rain stops
      if (leafDripTimer > 0) {
        leafDripTimer -= dt;
        leafDripInterval += dt;
        // Drip rate tapers off: starts at ~6/sec, ends at ~1/sec
        const dripRate = 0.15 + 0.35 * (1 - leafDripTimer / 60);
        while (leafDripInterval >= dripRate) {
          leafDripInterval -= dripRate;
          particles.emitLeafDrip();
        }
      }

      // Rain puddle shimmer: reflective ground sparkles form during rain
      if (weatherState === 1 && Math.random() < dt * 2) {
        const px = 10 + Math.random() * (GRID_X - 20);
        const pz = 10 + Math.random() * (GRID_Y - 20);
        particles.emitPuddleShimmer(px, pz);
      }

      // Drought dust devils: occasional spiral dust particles
      if (weatherState === 2 && Math.random() < dt * 0.3) {
        const dx = 10 + Math.random() * (GRID_X - 20);
        const dz = 10 + Math.random() * (GRID_Y - 20);
        particles.emitDustDevil(dx, dz);
      }

      // Wind strength varies with weather: gusty in rain, still in drought
      const baseWind = weatherState === 1 ? 0.7 : weatherState === 2 ? 0.12 : 0.35;

      // Dynamic wind gusts — periodic pulses that spike wind strength
      gustTimer -= dt;
      if (gustTimer <= 0) {
        gustStrength = 0.3 + Math.random() * 0.3; // 0.3-0.6 gust peak
        // Next gust sooner in rain (windy), later in drought (still)
        gustTimer = weatherState === 1 ? 6 + Math.random() * 8
          : weatherState === 2 ? 20 + Math.random() * 15
          : 10 + Math.random() * 15;
        playWindGust();
        fallingLeaves.emitGustBurst();
        particles.emitPetalBurst();
        // Wind chime: dense foliage (300+) gardens ring softly on gusts (~40% chance)
        if (foliage.count > 300 && Math.random() < 0.4) playWindChime();
      }
      // Gust decays smoothly over ~3s
      gustStrength *= Math.max(0, 1 - dt * 0.5);
      if (gustStrength < 0.01) gustStrength = 0;

      // Wind direction drifts slowly (full rotation every ~2 min)
      windAngle += dt * 0.05;

      // Wind streak particles during gusts — shows air direction
      if (gustStrength > 0.1 && Math.random() < dt * gustStrength * 4) {
        particles.emitWindStreak(windAngle, gustStrength);
      }

      const targetWind = Math.min(1, baseWind + gustStrength);
      const currentWind = foliage.getWindStrength();
      const newWind = currentWind + (targetWind - currentWind) * 0.05;
      foliage.setWindStrength(newWind);
      setWindAmbient(newWind);
      setLeafRustle(foliage.count, newWind);
    }
    rain.update(dt);

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
