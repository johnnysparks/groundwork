/**
 * playtest-density.mjs — Test the density-not-species experience.
 *
 * Simulates the actual new player flow: paint density zones (species=255),
 * watch what the sim decides to grow. Captures frames showing emergence.
 *
 * Usage: BROWSER_PATH=... node scripts/playtest-density.mjs
 */

import { chromium } from 'playwright-core';
import { spawn } from 'child_process';
import fs from 'fs';
import path from 'path';

const BROWSER_PATH = process.env.BROWSER_PATH;
const WEB_DIR = process.env.WEB_DIR || path.dirname(new URL(import.meta.url).pathname).replace('/scripts', '');
const OUTPUT_DIR = path.join(WEB_DIR, 'artifacts', 'playtest-density');
const PORT = 5180;

if (!BROWSER_PATH) {
  // Try playwright cache
  const home = process.env.HOME;
  const dirs = [
    `${home}/Library/Caches/ms-playwright`,
    `${home}/.cache/ms-playwright`,
  ];
  for (const dir of dirs) {
    if (!fs.existsSync(dir)) continue;
    const found = fs.readdirSync(dir, { recursive: true })
      .find(f => f.endsWith('Chromium') || f.endsWith('chrome'));
    if (found) {
      process.env.BROWSER_PATH = path.join(dir, found);
      break;
    }
  }
}

fs.mkdirSync(OUTPUT_DIR, { recursive: true });

// Start vite if needed
let viteProcess = null;
try {
  await fetch(`http://localhost:${PORT}`);
} catch {
  console.log('Starting Vite...');
  viteProcess = spawn('npx', ['vite', '--port', String(PORT)], {
    cwd: WEB_DIR, stdio: ['pipe', 'pipe', 'pipe'],
  });
  await new Promise((resolve, reject) => {
    const t = setTimeout(() => reject(new Error('Vite timeout')), 30000);
    viteProcess.stdout.on('data', d => {
      if (d.toString().includes('ready in')) { clearTimeout(t); setTimeout(resolve, 1000); }
    });
    viteProcess.on('error', reject);
  });
}

const browser = await chromium.launch({
  executablePath: process.env.BROWSER_PATH,
  headless: true,
  args: ['--use-gl=angle', '--use-angle=swiftshader', '--no-sandbox',
         '--disable-setuid-sandbox', '--disable-dev-shm-usage'],
});
const page = await (await browser.newContext({ viewport: { width: 1920, height: 1080 } })).newPage();
page.on('console', m => { if (m.type() === 'error') console.log(`  [err] ${m.text()}`); });

await page.goto(`http://localhost:${PORT}`, { waitUntil: 'networkidle', timeout: 30000 });
try { await page.waitForSelector('#loading.hidden', { timeout: 15000 }); } catch {}

const ready = await page.evaluate(() => window.agentAPI?.isReady?.() ?? false);
if (!ready) {
  console.log('WASM not ready — aborting');
  await browser.close();
  if (viteProcess) viteProcess.kill();
  process.exit(1);
}

console.log('WASM ready. Starting density painting playtest...\n');

const exec = (action) => page.evaluate(async a => {
  const api = window.agentAPI;
  if (api) return await api.executeAction(a);
}, action);

const snap = async (name, description) => {
  await page.evaluate(() => { window.agentAPI?.hideUI?.(); });
  await page.evaluate(async () => {
    for (let i = 0; i < 5; i++) await new Promise(r => requestAnimationFrame(r));
  });
  await page.waitForTimeout(150);
  const fp = path.join(OUTPUT_DIR, `${name}.png`);
  await page.screenshot({ path: fp });
  const kb = (fs.statSync(fp).size / 1024).toFixed(0);
  console.log(`  ${name} (${kb}KB) — ${description}`);
};

const getStats = () => page.evaluate(() => {
  const api = window.agentAPI;
  return {
    tick: api?.getTick?.() ?? 0,
    fauna: api?.getFaunaCount?.() ?? 0,
  };
});

// Set golden hour for warm visuals
await page.evaluate(t => window.agentAPI?.setTimeOfDay(t), 0.75);

// Water basin
await exec({ type: 'Fill', tool: 'water', x1: 25, y1: 25, z1: 50, x2: 55, y2: 55, z2: 50 });

// === Phase 1: Fresh garden ===
await exec({ type: 'CameraOrbit', theta_deg: 45, phi_deg: 60 });
await snap('01-empty', 'Empty garden with water basin');

// === Phase 2: Density paint near water (species=255) ===
// Simulate the actual player flow: click-to-sow with species=255
for (let x = 30; x <= 50; x += 3) {
  for (let y = 30; y <= 50; y += 3) {
    await exec({ type: 'Place', tool: 'seed', x, y, z: 55 });
    // Note: species omitted = undefined → agent API should treat as 255
  }
}
await snap('02-density-painted', 'Density zones painted — seeds placed with species=255');

// === Phase 3: Early germination ===
await exec({ type: 'Tick', n: 50 });
let stats = await getStats();
await snap('03-germinating', `50 ticks: seeds germinating (tick=${stats.tick})`);

// === Phase 4: First emergence ===
await exec({ type: 'Tick', n: 50 });
stats = await getStats();
await snap('04-first-growth', `100 ticks: first species emerging (tick=${stats.tick})`);

// === Phase 5: Garden taking shape ===
await exec({ type: 'Tick', n: 100 });
stats = await getStats();
await snap('05-taking-shape', `200 ticks: garden shape forming (tick=${stats.tick}, fauna=${stats.fauna})`);

// Close-up
await exec({ type: 'CameraZoom', level: 2.0 });
await snap('05-closeup', 'Close-up: what species grew?');

// === Phase 6: Established garden ===
await exec({ type: 'CameraZoom', level: 1.0 });
await exec({ type: 'Tick', n: 200 });
stats = await getStats();
await snap('06-established', `400 ticks: established garden (tick=${stats.tick}, fauna=${stats.fauna})`);

// X-ray underground
await exec({ type: 'CameraCutaway', z: 1 });
await page.waitForTimeout(200);
await snap('06-xray', 'Underground view: root diversity');
await exec({ type: 'CameraCutaway', z: 0 });

// === Phase 7: Mature garden ===
await exec({ type: 'Tick', n: 200 });
stats = await getStats();
await snap('07-mature', `600 ticks: mature garden (tick=${stats.tick}, fauna=${stats.fauna})`);

// Multiple angles
await exec({ type: 'CameraOrbit', theta_deg: 120, phi_deg: 40 });
await snap('07-side', 'Side view of mature garden');

await exec({ type: 'CameraOrbit', theta_deg: 45, phi_deg: 85 });
await snap('07-topdown', 'Top-down: species distribution');

console.log(`\n=== Density playtest complete ===`);
console.log(`Final: tick=${stats.tick}, fauna=${stats.fauna}`);

await browser.close();
if (viteProcess) viteProcess.kill();
