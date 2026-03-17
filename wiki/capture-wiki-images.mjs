/**
 * Wiki image capture — generates species thumbnails from the model viewer.
 *
 * Uses the model viewer scene (?scene=viewer&species=X) to capture clean
 * screenshots of each species at every growth stage. No WASM required —
 * the model viewer uses pre-built mock grids.
 *
 * Usage:
 *   cd crates/groundwork-web
 *   node ../../wiki/capture-wiki-images.mjs
 *
 * Prerequisites:
 *   - npm install (in crates/groundwork-web)
 *   - Playwright chromium installed: npx playwright install chromium
 *   - Vite dev server running: npm run dev (or script starts one)
 */

import { spawn } from 'child_process';
import { createRequire } from 'module';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const WEB_DIR = path.resolve(__dirname, '../crates/groundwork-web');

// Resolve playwright-core from the web directory's node_modules
const webRequire = createRequire(path.join(WEB_DIR, 'package.json'));
const { chromium } = webRequire('playwright-core');
const IMAGE_DIR = path.resolve(__dirname, 'images');
const VITE_PORT = 5175;

// --- Model viewer grid layout (must match mockGrid.ts createModelViewerGrid) ---
// All species use the same 160×50×80 grid with gl=30 to avoid renderer init-order bugs.
const LAYOUT = {
  tree:        { gw: 160, gd: 50, gl: 30, numStages: 4 },
  shrub:       { gw: 160, gd: 50, gl: 30, numStages: 3 },
  flower:      { gw: 160, gd: 50, gl: 30, numStages: 3 },
  groundcover: { gw: 160, gd: 50, gl: 30, numStages: 3 },
};

// Zoom per stage — small species need much more zoom since they're tiny on the 160-wide grid
const ZOOM = {
  tree:        [3.5, 2.5, 1.8, 1.4],
  shrub:       [5.0, 4.0, 3.5],
  flower:      [6.0, 5.0, 4.0],
  groundcover: [6.0, 5.0, 4.0],
};

// How far above ground to aim the camera for each stage
const FOCUS_Z = {
  tree:        [2, 8, 14, 18],
  shrub:       [1, 2, 3],
  flower:      [1, 2, 3],
  groundcover: [0, 0, 1],
};

// Species to photograph (stage names + tick labels match existing wiki filenames)
const SPECIES = [
  { name: 'oak',        type: 'tree',        stages: ['seedling','sapling','young','mature'], ticks: [30,100,300,600] },
  { name: 'birch',      type: 'tree',        stages: ['seedling','sapling','young','mature'], ticks: [30,100,300,600] },
  { name: 'willow',     type: 'tree',        stages: ['seedling','sapling','young','mature'], ticks: [30,100,300,600] },
  { name: 'pine',       type: 'tree',        stages: ['seedling','sapling','young','mature'], ticks: [30,100,300,600] },
  { name: 'fern',       type: 'shrub',       stages: ['seedling','sapling','young'], ticks: [20,60,150] },
  { name: 'berry-bush', type: 'shrub',       stages: ['seedling','sapling','young'], ticks: [20,60,150] },
  { name: 'holly',      type: 'shrub',       stages: ['seedling','sapling','young'], ticks: [20,60,150] },
  { name: 'wildflower', type: 'flower',      stages: ['seedling','sapling','young'], ticks: [15,40,80] },
  { name: 'daisy',      type: 'flower',      stages: ['seedling','sapling','young'], ticks: [15,40,80] },
  { name: 'moss',       type: 'groundcover', stages: ['seedling','sapling','young'], ticks: [15,40,80] },
  { name: 'grass',      type: 'groundcover', stages: ['seedling','sapling','young'], ticks: [15,40,80] },
  { name: 'clover',     type: 'groundcover', stages: ['seedling','sapling','young'], ticks: [15,40,80] },
];

// Interaction scenes (still require WASM sim)
const INTERACTION_SCENES = [
  {
    name: 'nitrogen-handshake',
    description: 'Clover boosting oak growth',
    setup: [
      { type: 'Fill', tool: 'water', x1: 35, y1: 35, z1: 50, x2: 45, y2: 45, z2: 50 },
      { type: 'Place', tool: 'seed', x: 40, y: 40, z: 55, species: 'oak' },
      { type: 'Place', tool: 'seed', x: 42, y: 42, z: 55, species: 'clover' },
      { type: 'Place', tool: 'seed', x: 38, y: 42, z: 55, species: 'clover' },
      { type: 'Place', tool: 'seed', x: 42, y: 38, z: 55, species: 'clover' },
    ],
    ticks: 400,
    camera: { theta: 30, phi: 50, zoom: 1.3 },
  },
  {
    name: 'canopy-layers',
    description: 'Oak canopy with fern understory and moss ground',
    setup: [
      { type: 'Fill', tool: 'water', x1: 30, y1: 30, z1: 50, x2: 50, y2: 50, z2: 50 },
      { type: 'Place', tool: 'seed', x: 40, y: 40, z: 55, species: 'oak' },
      { type: 'Place', tool: 'seed', x: 38, y: 38, z: 55, species: 'fern' },
      { type: 'Place', tool: 'seed', x: 42, y: 38, z: 55, species: 'fern' },
      { type: 'Place', tool: 'seed', x: 38, y: 42, z: 55, species: 'moss' },
      { type: 'Place', tool: 'seed', x: 42, y: 42, z: 55, species: 'moss' },
    ],
    ticks: 500,
    camera: { theta: 40, phi: 45, zoom: 1.2 },
  },
  {
    name: 'pine-territory',
    description: 'Pine with acidified soil zone',
    setup: [
      { type: 'Fill', tool: 'water', x1: 35, y1: 35, z1: 50, x2: 45, y2: 45, z2: 50 },
      { type: 'Place', tool: 'seed', x: 40, y: 40, z: 55, species: 'pine' },
      { type: 'Place', tool: 'seed', x: 44, y: 40, z: 55, species: 'oak' },
      { type: 'Place', tool: 'seed', x: 36, y: 40, z: 55, species: 'fern' },
    ],
    ticks: 500,
    camera: { theta: 45, phi: 55, zoom: 1.2 },
  },
  {
    name: 'competition',
    description: 'Crowded oaks thinning naturally',
    setup: [
      { type: 'Fill', tool: 'water', x1: 36, y1: 36, z1: 50, x2: 44, y2: 44, z2: 50 },
      ...Array.from({ length: 9 }, (_, i) => ({
        type: 'Place', tool: 'seed',
        x: 37 + (i % 3) * 3, y: 37 + Math.floor(i / 3) * 3, z: 55, species: 'oak',
      })),
    ],
    ticks: 400,
    camera: { theta: 45, phi: 60, zoom: 1.0 },
  },
  {
    name: 'xray-roots',
    description: 'Underground root systems in x-ray mode',
    setup: [
      { type: 'Fill', tool: 'water', x1: 30, y1: 35, z1: 50, x2: 50, y2: 45, z2: 50 },
      { type: 'Place', tool: 'seed', x: 35, y: 40, z: 55, species: 'oak' },
      { type: 'Place', tool: 'seed', x: 45, y: 40, z: 55, species: 'pine' },
    ],
    ticks: 400,
    xray: true,
    camera: { theta: 45, phi: 55, zoom: 1.0 },
  },
];

// --- Helpers ---

function stageX(type, stageIndex) {
  const l = LAYOUT[type];
  const spacing = Math.floor(l.gw / (l.numStages + 1));
  return spacing * (stageIndex + 1);
}

async function findBrowser() {
  const cacheDir = process.platform === 'darwin'
    ? path.join(process.env.HOME, 'Library/Caches/ms-playwright')
    : path.join(process.env.HOME, '.cache/ms-playwright');

  if (fs.existsSync(cacheDir)) {
    const pattern = process.platform === 'darwin' ? 'Chromium.app/Contents/MacOS/Chromium' : 'chrome-linux/chrome';
    const entries = fs.readdirSync(cacheDir, { recursive: true, encoding: 'utf8' });
    for (const entry of entries) {
      if (entry.endsWith(pattern.split('/').pop())) {
        const full = path.join(cacheDir, entry);
        if (fs.existsSync(full)) return full;
      }
    }
  }
  for (const cmd of ['/usr/bin/chromium', '/usr/bin/google-chrome']) {
    if (fs.existsSync(cmd)) return cmd;
  }
  return null;
}

async function startVite() {
  try {
    const resp = await fetch(`http://localhost:${VITE_PORT}`);
    if (resp.ok || resp.status === 404) return null;
  } catch {}
  console.log('Starting Vite dev server...');
  const vite = spawn('npx', ['vite', '--port', String(VITE_PORT)], {
    cwd: WEB_DIR, stdio: ['pipe', 'pipe', 'pipe'],
  });
  await new Promise((resolve, reject) => {
    const t = setTimeout(() => reject(new Error('Vite timeout')), 30000);
    vite.stdout.on('data', d => {
      if (d.toString().includes('ready in') || d.toString().includes('Local:')) {
        clearTimeout(t); setTimeout(resolve, 500);
      }
    });
    vite.stderr.on('data', d => process.stderr.write(d));
    vite.on('error', e => { clearTimeout(t); reject(e); });
  });
  return vite;
}

async function waitForAPI(page) {
  await page.waitForFunction(() => !!window.agentAPI, { timeout: 15000 });
  await page.waitForTimeout(500);
}

async function exec(page, action) {
  await page.evaluate(async a => {
    const api = window.agentAPI;
    if (api) await api.executeAction(a);
  }, action);
}

async function cleanScene(page) {
  await page.evaluate(() => {
    const api = window.agentAPI;
    if (!api) return;
    api.hideUI();
    api.setSceneryVisible(false);
    api.setFogEnabled(false);
    api.setTimeOfDay(0.45); // slightly before noon — warm light, soft shadows
  });
}

async function snap(page, filepath) {
  await page.evaluate(async () => {
    for (let i = 0; i < 15; i++) await new Promise(r => requestAnimationFrame(r));
  });
  await page.waitForTimeout(200);
  await page.screenshot({ path: filepath, type: 'png' });
  const size = (fs.statSync(filepath).size / 1024).toFixed(0);
  console.log(`  -> ${path.relative(IMAGE_DIR, filepath)} (${size}KB)`);
}

// --- Main ---

async function main() {
  fs.mkdirSync(path.join(IMAGE_DIR, 'species'), { recursive: true });
  fs.mkdirSync(path.join(IMAGE_DIR, 'interactions'), { recursive: true });
  fs.mkdirSync(path.join(IMAGE_DIR, 'fauna'), { recursive: true });

  const browserPath = await findBrowser();
  if (!browserPath) {
    console.error('No browser found. Run: npx playwright install chromium');
    process.exit(1);
  }
  console.log(`Browser: ${browserPath}`);

  const viteProcess = await startVite();
  const browser = await chromium.launch({
    executablePath: browserPath,
    headless: true,
    args: ['--use-gl=angle', '--use-angle=swiftshader', '--no-sandbox', '--disable-setuid-sandbox'],
  });

  try {
    const context = await browser.newContext({ viewport: { width: 800, height: 600 } });

    // === Species thumbnails (model viewer — no WASM needed) ===
    console.log('\n=== Species Thumbnails (Model Viewer) ===\n');

    for (const sp of SPECIES) {
      console.log(`${sp.name} (${sp.type}):`);
      const layout = LAYOUT[sp.type];
      const zooms = ZOOM[sp.type];
      const focusZ = FOCUS_Z[sp.type];

      const page = await context.newPage();
      const url = `http://localhost:${VITE_PORT}?scene=viewer&species=${sp.name}`;
      await page.goto(url, { waitUntil: 'networkidle', timeout: 30000 });
      try { await page.waitForSelector('#loading.hidden', { timeout: 15000 }); } catch {}

      await waitForAPI(page);
      await cleanScene(page);

      // Capture each growth stage
      for (let i = 0; i < sp.stages.length; i++) {
        const sx = stageX(sp.type, i);
        const cy = Math.floor(layout.gd / 2);
        const fz = layout.gl + focusZ[i];

        // Reset camera, set orbit angle, pan to stage, zoom in
        await exec(page, { type: 'CameraOrbit', theta_deg: 35, phi_deg: 45 });
        await exec(page, { type: 'CameraPan', x: sx, y: cy, z: fz });
        await exec(page, { type: 'CameraZoom', level: zooms[i] });

        const filepath = path.join(IMAGE_DIR, 'species', `${sp.name}-${sp.stages[i]}-t${sp.ticks[i]}.png`);
        await snap(page, filepath);
      }

      // Roots view: pan to the most mature stage, enable x-ray
      const lastIdx = sp.stages.length - 1;
      const rootX = stageX(sp.type, lastIdx);
      const rootCy = Math.floor(layout.gd / 2);

      await exec(page, { type: 'CameraCutaway', z: 1 });
      await exec(page, { type: 'CameraOrbit', theta_deg: 45, phi_deg: 55 });
      await exec(page, { type: 'CameraPan', x: rootX, y: rootCy, z: layout.gl - 5 });
      await exec(page, { type: 'CameraZoom', level: zooms[lastIdx] });

      const rootPath = path.join(IMAGE_DIR, 'species', `${sp.name}-roots.png`);
      await snap(page, rootPath);

      await page.close();
    }

    // === Interaction scenes (requires WASM) ===
    console.log('\n=== Interaction Scenes (WASM sim) ===\n');

    for (const scene of INTERACTION_SCENES) {
      console.log(`${scene.name}: ${scene.description}`);
      const page = await context.newPage();
      await page.goto(`http://localhost:${VITE_PORT}`, { waitUntil: 'networkidle', timeout: 30000 });
      try { await page.waitForSelector('#loading.hidden', { timeout: 15000 }); } catch {}
      await page.waitForTimeout(500);

      const wasmReady = await page.evaluate(() => window.agentAPI?.isReady?.() ?? false);
      if (!wasmReady) {
        console.log('  WASM not ready, skipping (run npm run wasm first)');
        await page.close();
        continue;
      }

      for (const action of scene.setup) {
        await exec(page, action);
      }

      await page.evaluate(() => {
        window.agentAPI.hideUI();
        window.agentAPI.setSceneryVisible(false);
        window.agentAPI.setFogEnabled(false);
        window.agentAPI.setTimeOfDay(0.75); // golden hour
      });

      await exec(page, { type: 'Tick', n: scene.ticks });

      if (scene.xray) {
        await exec(page, { type: 'CameraCutaway', z: 1 });
        await page.waitForTimeout(200);
      }

      await exec(page, { type: 'CameraOrbit', theta_deg: scene.camera.theta, phi_deg: scene.camera.phi });
      await exec(page, { type: 'CameraZoom', level: scene.camera.zoom });

      const filepath = path.join(IMAGE_DIR, 'interactions', `${scene.name}.png`);
      await snap(page, filepath);

      await page.close();
    }

    console.log(`\nDone! Images in: ${IMAGE_DIR}`);

  } finally {
    await browser.close();
    if (viteProcess) viteProcess.kill('SIGTERM');
  }
}

main().catch(e => { console.error(e); process.exit(1); });
