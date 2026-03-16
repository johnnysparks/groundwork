/**
 * Wiki image capture — generates species thumbnails and gameplay screenshots.
 *
 * Uses the agentAPI to plant individual species, advance growth, and capture
 * close-up screenshots at each growth stage. Output goes to wiki/images/.
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

import { chromium } from 'playwright-core';
import { spawn } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const WEB_DIR = path.resolve(__dirname, '../crates/groundwork-web');
const IMAGE_DIR = path.resolve(__dirname, 'images');
const VITE_PORT = 5175; // different port to avoid conflicts

// Species to photograph
const SPECIES = [
  { name: 'oak',        id: 0,  type: 'tree',        ticks: [30, 100, 300, 600] },
  { name: 'birch',      id: 1,  type: 'tree',        ticks: [30, 100, 300, 600] },
  { name: 'willow',     id: 2,  type: 'tree',        ticks: [30, 100, 300, 600] },
  { name: 'pine',       id: 3,  type: 'tree',        ticks: [30, 100, 300, 600] },
  { name: 'fern',       id: 4,  type: 'shrub',       ticks: [20, 60, 150] },
  { name: 'berry-bush', id: 5,  type: 'shrub',       ticks: [20, 60, 150] },
  { name: 'holly',      id: 6,  type: 'shrub',       ticks: [20, 60, 150] },
  { name: 'wildflower', id: 7,  type: 'flower',      ticks: [15, 40, 80] },
  { name: 'daisy',      id: 8,  type: 'flower',      ticks: [15, 40, 80] },
  { name: 'moss',       id: 9,  type: 'groundcover', ticks: [15, 40, 80] },
  { name: 'grass',      id: 10, type: 'groundcover', ticks: [15, 40, 80] },
  { name: 'clover',     id: 11, type: 'groundcover', ticks: [15, 40, 80] },
];

// Interaction scenes to photograph
const SCENES = [
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
    camera: { theta: 30, phi: 50, zoom: 2.5 },
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
    camera: { theta: 40, phi: 45, zoom: 2.0 },
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
    camera: { theta: 45, phi: 55, zoom: 2.0 },
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
    camera: { theta: 45, phi: 60, zoom: 1.8 },
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
    camera: { theta: 45, phi: 55, zoom: 1.5 },
  },
];

// --- Helpers ---

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
  // Fallback: try system chrome
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

// --- Main ---

async function main() {
  fs.mkdirSync(IMAGE_DIR, { recursive: true });
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

    const exec = async (page, action) => {
      await page.evaluate(async a => {
        const api = window.agentAPI;
        if (api) await api.executeAction(a);
      }, action);
    };

    const snap = async (page, filepath) => {
      await page.evaluate(async () => {
        for (let i = 0; i < 10; i++) await new Promise(r => requestAnimationFrame(r));
      });
      await page.waitForTimeout(100);
      await page.screenshot({ path: filepath, type: 'png' });
      const size = (fs.statSync(filepath).size / 1024).toFixed(0);
      console.log(`  -> ${path.relative(IMAGE_DIR, filepath)} (${size}KB)`);
    };

    // --- Species thumbnails ---
    console.log('\n=== Species Thumbnails ===\n');
    for (const sp of SPECIES) {
      console.log(`${sp.name} (${sp.type}):`);
      const page = await context.newPage();
      await page.goto(`http://localhost:${VITE_PORT}`, { waitUntil: 'networkidle', timeout: 30000 });
      try { await page.waitForSelector('#loading.hidden', { timeout: 15000 }); } catch {}
      await page.waitForTimeout(500);

      const wasmReady = await page.evaluate(() => window.agentAPI?.isReady?.() ?? false);
      if (!wasmReady) {
        console.log('  WASM not ready, skipping');
        await page.close();
        continue;
      }

      // Water near center
      await exec(page, { type: 'Fill', tool: 'water', x1: 35, y1: 35, z1: 50, x2: 45, y2: 45, z2: 50 });
      // Plant the species
      await exec(page, { type: 'Place', tool: 'seed', x: 40, y: 40, z: 55, species: sp.name });
      // Set to noon for consistent lighting
      await page.evaluate(() => window.agentAPI?.setTimeOfDay(0.5));

      let prevTick = 0;
      for (let i = 0; i < sp.ticks.length; i++) {
        const targetTick = sp.ticks[i];
        const advance = targetTick - prevTick;
        if (advance > 0) {
          await exec(page, { type: 'Tick', n: advance });
        }
        prevTick = targetTick;

        // Close-up camera for thumbnails
        await exec(page, { type: 'CameraOrbit', theta_deg: 35, phi_deg: 50 });
        await exec(page, { type: 'CameraZoom', level: 3.0 });

        const stage = i === 0 ? 'seedling' : i === 1 ? 'sapling' : i === 2 ? 'young' : 'mature';
        const filepath = path.join(IMAGE_DIR, 'species', `${sp.name}-${stage}-t${targetTick}.png`);
        await snap(page, filepath);
      }

      // X-ray view for root system
      await exec(page, { type: 'CameraCutaway', z: 1 });
      await exec(page, { type: 'CameraOrbit', theta_deg: 45, phi_deg: 55 });
      await exec(page, { type: 'CameraZoom', level: 2.5 });
      const rootPath = path.join(IMAGE_DIR, 'species', `${sp.name}-roots.png`);
      await snap(page, rootPath);

      await page.close();
    }

    // --- Interaction scenes ---
    console.log('\n=== Interaction Scenes ===\n');
    for (const scene of SCENES) {
      console.log(`${scene.name}: ${scene.description}`);
      const page = await context.newPage();
      await page.goto(`http://localhost:${VITE_PORT}`, { waitUntil: 'networkidle', timeout: 30000 });
      try { await page.waitForSelector('#loading.hidden', { timeout: 15000 }); } catch {}
      await page.waitForTimeout(500);

      const wasmReady = await page.evaluate(() => window.agentAPI?.isReady?.() ?? false);
      if (!wasmReady) {
        console.log('  WASM not ready, skipping');
        await page.close();
        continue;
      }

      for (const action of scene.setup) {
        await exec(page, action);
      }

      await page.evaluate(() => window.agentAPI?.setTimeOfDay(0.75)); // golden hour
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
    console.log('Run with WASM build for full sim screenshots: cd crates/groundwork-web && npm run wasm');

  } finally {
    await browser.close();
    if (viteProcess) viteProcess.kill('SIGTERM');
  }
}

main().catch(e => { console.error(e); process.exit(1); });
