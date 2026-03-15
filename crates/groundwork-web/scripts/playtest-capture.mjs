/**
 * Playtest capture — plants a garden through the web renderer via agentAPI,
 * captures screenshots at each stage to evaluate the visual experience.
 */
import { chromium } from 'playwright-core';
import { spawn } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const WEB_DIR = path.resolve(__dirname, '..');
const SCREENSHOT_DIR = process.env.SCREENSHOT_DIR || path.resolve(__dirname, '../../../artifacts/screenshots/playtest');

// Find browser
function findBrowser() {
  const paths = [
    '/root/.cache/ms-playwright/chromium-1194/chrome-linux/chrome',
    '/root/.cache/ms-playwright/chromium-1208/chrome-linux/chrome',
  ];
  for (const p of paths) {
    if (fs.existsSync(p)) return p;
  }
  throw new Error('No browser found');
}

async function startVite(port) {
  try {
    const resp = await fetch(`http://localhost:${port}`);
    if (resp.ok || resp.status === 404) {
      console.log(`Vite already running on port ${port}`);
      return null;
    }
  } catch {}

  console.log('Starting Vite dev server...');
  const vite = spawn('npx', ['vite', '--port', String(port)], {
    cwd: WEB_DIR, stdio: ['pipe', 'pipe', 'pipe'], env: { ...process.env },
  });
  await new Promise((resolve, reject) => {
    const t = setTimeout(() => reject(new Error('Vite timeout')), 30000);
    vite.stdout.on('data', d => {
      const s = d.toString();
      if (s.includes('Local:') || s.includes('ready in')) { clearTimeout(t); setTimeout(resolve, 1000); }
    });
    vite.stderr.on('data', d => process.stderr.write(d));
    vite.on('error', e => { clearTimeout(t); reject(e); });
  });
  return vite;
}

async function main() {
  fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
  const port = 5175;
  const viteProcess = await startVite(port);
  let browser;

  try {
    browser = await chromium.launch({
      executablePath: findBrowser(),
      headless: true,
      args: [
        '--use-gl=angle', '--use-angle=swiftshader',
        '--no-sandbox', '--disable-setuid-sandbox',
        '--disable-dev-shm-usage', '--disable-gpu-sandbox',
      ],
    });

    const page = await (await browser.newContext({ viewport: { width: 1920, height: 1080 } })).newPage();
    page.on('console', m => {
      if (m.type() === 'error') console.log(`  [browser error] ${m.text()}`);
    });

    await page.goto(`http://localhost:${port}`, { waitUntil: 'networkidle', timeout: 30000 });
    try { await page.waitForSelector('#loading.hidden', { timeout: 15000 }); } catch {}

    const hasAPI = await page.evaluate(() => !!window.agentAPI);
    const wasmReady = await page.evaluate(() => window.agentAPI?.isReady?.() ?? false);
    console.log(`agentAPI: ${hasAPI}, WASM: ${wasmReady}`);

    // Wait for first render
    await page.evaluate(async () => {
      for (let i = 0; i < 10; i++) await new Promise(r => requestAnimationFrame(r));
    });
    await page.waitForTimeout(500);

    const exec = (action) => page.evaluate(async a => {
      const api = window.agentAPI;
      if (api) return await api.executeAction(a);
      return { action: 'no-api', tick: 0 };
    }, action);

    const snap = async (name) => {
      await page.evaluate(async () => {
        for (let i = 0; i < 5; i++) await new Promise(r => requestAnimationFrame(r));
      });
      await page.waitForTimeout(200);
      const fp = path.join(SCREENSHOT_DIR, `${name}.png`);
      await page.screenshot({ path: fp, type: 'png' });
      const size = (fs.statSync(fp).size / 1024).toFixed(0);
      console.log(`  📸 ${name}.png (${size}KB)`);
    };

    const counts = async () => {
      return page.evaluate(() => window.agentAPI?.getMaterialCounts?.() ?? {});
    };

    // ===== PLAYTEST SEQUENCE =====

    // 1. Initial state
    console.log('\n=== Phase 1: Initial state ===');
    await exec({ type: 'CameraOrbit', theta_deg: 45, phi_deg: 60 });
    await snap('01-initial-view');
    console.log('  Materials:', JSON.stringify(await counts()));

    // 2. Create water features
    console.log('\n=== Phase 2: Adding water ===');
    await exec({ type: 'Fill', tool: 'water', x1: 35, y1: 35, z1: 45, x2: 45, y2: 45, z2: 45 });
    await exec({ type: 'Fill', tool: 'water', x1: 55, y1: 55, z1: 45, x2: 62, y2: 62, z2: 45 });
    await exec({ type: 'Tick', n: 5 });
    await snap('02-water-placed');

    // 3. Plant diverse species
    console.log('\n=== Phase 3: Planting garden ===');
    // Trees
    await exec({ type: 'Place', tool: 'seed', x: 38, y: 38, z: 50, species: 'oak' });
    await exec({ type: 'Place', tool: 'seed', x: 32, y: 32, z: 50, species: 'birch' });
    await exec({ type: 'Place', tool: 'seed', x: 58, y: 58, z: 50, species: 'willow' });
    await exec({ type: 'Place', tool: 'seed', x: 45, y: 30, z: 50, species: 'pine' });
    // Shrubs
    await exec({ type: 'Place', tool: 'seed', x: 36, y: 42, z: 50, species: 'fern' });
    await exec({ type: 'Place', tool: 'seed', x: 60, y: 56, z: 50, species: 'berry-bush' });
    await exec({ type: 'Place', tool: 'seed', x: 42, y: 36, z: 50, species: 'holly' });
    // Flowers
    await exec({ type: 'Place', tool: 'seed', x: 37, y: 37, z: 50, species: 'wildflower' });
    await exec({ type: 'Place', tool: 'seed', x: 43, y: 43, z: 50, species: 'daisy' });
    await exec({ type: 'Place', tool: 'seed', x: 56, y: 60, z: 50, species: 'wildflower' });
    // Groundcover
    await exec({ type: 'Place', tool: 'seed', x: 37, y: 39, z: 50, species: 'clover' });
    await exec({ type: 'Place', tool: 'seed', x: 39, y: 37, z: 50, species: 'moss' });
    await exec({ type: 'Place', tool: 'seed', x: 59, y: 57, z: 50, species: 'grass' });

    await exec({ type: 'Tick', n: 5 });
    await snap('03-seeds-planted');
    console.log('  Materials:', JSON.stringify(await counts()));

    // 4. Early growth (tick 50)
    console.log('\n=== Phase 4: Early growth ===');
    await exec({ type: 'Tick', n: 50 });
    await snap('04-early-growth-hero');
    await exec({ type: 'CameraOrbit', theta_deg: 120, phi_deg: 40 });
    await snap('04-early-growth-side');
    console.log('  Materials:', JSON.stringify(await counts()));

    // 5. Mid growth (tick 150)
    console.log('\n=== Phase 5: Mid growth ===');
    await exec({ type: 'Tick', n: 100 });
    await exec({ type: 'CameraOrbit', theta_deg: 45, phi_deg: 60 });
    await snap('05-mid-growth-hero');
    await exec({ type: 'CameraOrbit', theta_deg: 45, phi_deg: 80 });
    await exec({ type: 'CameraZoom', level: 0.7 });
    await snap('05-mid-growth-topdown');
    console.log('  Materials:', JSON.stringify(await counts()));

    // 6. Mature garden (tick 350)
    console.log('\n=== Phase 6: Mature garden ===');
    await exec({ type: 'Tick', n: 200 });
    await exec({ type: 'CameraOrbit', theta_deg: 45, phi_deg: 60 });
    await exec({ type: 'CameraZoom', level: 1.0 });
    await snap('06-mature-hero');
    await exec({ type: 'CameraOrbit', theta_deg: 200, phi_deg: 25 });
    await snap('06-mature-low-angle');
    await exec({ type: 'CameraOrbit', theta_deg: 90, phi_deg: 50 });
    await exec({ type: 'CameraZoom', level: 1.8 });
    await snap('06-mature-closeup');
    console.log('  Materials:', JSON.stringify(await counts()));

    // 7. Late garden (tick 500+)
    console.log('\n=== Phase 7: Late garden ===');
    await exec({ type: 'Tick', n: 200 });
    await exec({ type: 'CameraOrbit', theta_deg: 45, phi_deg: 60 });
    await exec({ type: 'CameraZoom', level: 1.0 });
    await snap('07-late-hero');
    await exec({ type: 'CameraOrbit', theta_deg: 315, phi_deg: 55 });
    await snap('07-late-opposite');
    console.log('  Materials:', JSON.stringify(await counts()));

    console.log(`\nDone: screenshots → ${SCREENSHOT_DIR}`);
  } finally {
    if (browser) await browser.close();
    if (viteProcess) viteProcess.kill('SIGTERM');
  }
}

main().catch(e => { console.error(e); process.exit(1); });
