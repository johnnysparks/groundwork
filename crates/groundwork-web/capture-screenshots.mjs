/**
 * Direct screenshot capture script — no Playwright test runner needed.
 * Uses playwright-core to launch Chromium, navigate to the Vite dev server,
 * and capture PNG screenshots of the Three.js scene in mock data mode.
 */
import { chromium } from 'playwright-core';
import { spawn } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SCREENSHOT_DIR = path.resolve(__dirname, '../../artifacts/screenshots');
const PORT = 5174;

async function startViteServer() {
  console.log('Starting Vite dev server...');
  const vite = spawn('npx', ['vite', '--port', String(PORT), '--host', '0.0.0.0'], {
    cwd: __dirname,
    stdio: ['pipe', 'pipe', 'pipe'],
    env: { ...process.env },
  });

  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error('Vite server start timeout')), 30000);

    vite.stdout.on('data', (data) => {
      const msg = data.toString();
      process.stdout.write(`  [vite] ${msg}`);
      if (msg.includes('Local:') || msg.includes('ready in')) {
        clearTimeout(timeout);
        setTimeout(() => resolve(vite), 1000);
      }
    });

    vite.stderr.on('data', (data) => {
      process.stderr.write(`  [vite-err] ${data.toString()}`);
    });

    vite.on('error', (err) => {
      clearTimeout(timeout);
      reject(err);
    });
  });
}

/** Helper: execute an agent action and wait for a render frame */
async function executeAction(page, action) {
  await page.evaluate(async (a) => {
    const api = window.agentAPI;
    if (api) await api.executeAction(a);
  }, action);
}

/** Helper: wait for N render frames to complete */
async function waitFrames(page, n = 3) {
  await page.evaluate(async (count) => {
    for (let i = 0; i < count; i++) {
      await new Promise(r => requestAnimationFrame(r));
    }
  }, n);
  // Plus a small real-time buffer for GPU
  await page.waitForTimeout(100);
}

async function main() {
  fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });

  const viteProcess = await startViteServer();

  let browser;
  try {
    const browserPath = '/root/.cache/ms-playwright/chromium-1194/chrome-linux/chrome';
    if (!fs.existsSync(browserPath)) {
      throw new Error(`Chromium not found at ${browserPath}`);
    }

    console.log(`\nLaunching Chromium...`);
    browser = await chromium.launch({
      executablePath: browserPath,
      headless: true,
      args: [
        '--use-gl=angle',
        '--use-angle=swiftshader',
        '--no-sandbox',
        '--disable-setuid-sandbox',
        '--disable-dev-shm-usage',
        '--disable-gpu-sandbox',
      ],
    });

    const context = await browser.newContext({
      viewport: { width: 1920, height: 1080 },
    });
    const page = await context.newPage();

    page.on('console', (msg) => {
      if (msg.type() === 'error') {
        console.log(`  [browser-error] ${msg.text()}`);
      }
    });

    console.log(`Navigating to http://localhost:${PORT}...`);
    await page.goto(`http://localhost:${PORT}`, { waitUntil: 'networkidle', timeout: 30000 });

    // Wait for loading screen to hide
    console.log('Waiting for app initialization...');
    try {
      await page.waitForSelector('#loading.hidden', { timeout: 15000 });
    } catch {
      console.log('  Loading screen not found, continuing...');
    }

    const hasAPI = await page.evaluate(() => !!window.agentAPI);
    const wasmReady = await page.evaluate(() => window.agentAPI?.isReady?.() ?? false);
    console.log(`  agentAPI: ${hasAPI}, WASM: ${wasmReady}`);

    // Let Three.js finish initial render
    await waitFrames(page, 5);
    await page.waitForTimeout(1000);

    console.log('\nCapturing screenshots...');
    const screenshots = [];

    async function capture(name) {
      await waitFrames(page, 3);
      const filepath = path.join(SCREENSHOT_DIR, `${name}.png`);
      await page.screenshot({ path: filepath, type: 'png' });
      screenshots.push(filepath);
      console.log(`  ${name}.png`);
    }

    // 1. Default diorama view — the mock garden from the starting angle
    await capture('01-initial-view');

    // 2. Side view — lower elevation, rotated 90 degrees
    await executeAction(page, { type: 'CameraOrbit', theta_deg: 120, phi_deg: 35 });
    await capture('02-side-view');

    // 3. Top-down view — high elevation, looking down
    await executeAction(page, { type: 'CameraOrbit', theta_deg: 45, phi_deg: 80 });
    await capture('03-top-down');

    // 4. Close-up of the tree — zoom in
    await executeAction(page, { type: 'CameraOrbit', theta_deg: 30, phi_deg: 50 });
    await executeAction(page, { type: 'CameraZoom', level: 2.0 });
    await capture('04-close-up');

    // 5. Low angle dramatic view
    await executeAction(page, { type: 'CameraOrbit', theta_deg: 200, phi_deg: 25 });
    await executeAction(page, { type: 'CameraZoom', level: 1.0 });
    await capture('05-low-angle');

    // 6. Opposite corner view
    await executeAction(page, { type: 'CameraOrbit', theta_deg: 225, phi_deg: 55 });
    await capture('06-opposite-corner');

    // 7. Reset to starting angle for a clean "hero" shot
    await executeAction(page, { type: 'CameraReset' });
    await capture('07-hero-shot');

    console.log(`\nDone! ${screenshots.length} screenshots saved to ${SCREENSHOT_DIR}`);
    screenshots.forEach(s => console.log(`  ${path.basename(s)}`));

  } catch (err) {
    console.error('Error:', err);
    process.exit(1);
  } finally {
    if (browser) await browser.close();
    viteProcess.kill('SIGTERM');
  }
}

main().catch(err => {
  console.error('Fatal:', err);
  process.exit(1);
});
