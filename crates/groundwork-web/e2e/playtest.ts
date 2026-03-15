/**
 * Playwright screenshot harness for the player agent.
 *
 * Replays a groundwork-player trace through the actual Three.js web renderer
 * in a headless browser, capturing PNG screenshots at every Screenshot action.
 *
 * Usage:
 *   npx playwright test e2e/playtest.ts
 *   # or via npm script:
 *   npm run playtest -- --trace path/to/trace.json
 *
 * Prerequisites:
 *   - WASM must be built: npm run wasm
 *   - Playwright installed: npx playwright install chromium
 *
 * The harness:
 *   1. Starts the Vite dev server
 *   2. Opens the page in headless Chromium
 *   3. Waits for WASM sim to initialize
 *   4. Reads the trace JSON (from groundwork-player)
 *   5. Replays each action via window.agentAPI
 *   6. At Screenshot actions, captures the canvas via page.screenshot()
 *   7. Saves PNGs to artifacts/screenshots/ with the label as filename
 *
 * Screenshots are the actual rendered Three.js output — what a player sees.
 * These get attached to PRs for visual evaluation of the garden.
 */

import { test, expect } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Default trace path — can be overridden via TRACE_PATH env var
const TRACE_PATH = process.env.TRACE_PATH
  || path.resolve(__dirname, '../../../crates/groundwork-player/artifacts/traces/claude_session.json');

const SCREENSHOT_DIR = process.env.SCREENSHOT_DIR
  || path.resolve(__dirname, '../../../artifacts/screenshots');

/** Parse the trace action from the serde-serialized JSON format.
 *  Rust serde serializes enums as { "VariantName": { fields } } or just "VariantName" for unit variants.
 */
function parseAction(raw: any): { type: string; [key: string]: any } | null {
  if (typeof raw === 'string') {
    // Unit variant like "Status" or "CameraReset"
    return { type: raw };
  }
  if (typeof raw === 'object') {
    const keys = Object.keys(raw);
    if (keys.length === 1) {
      const type = keys[0];
      const fields = raw[type];
      if (typeof fields === 'object' && fields !== null) {
        return { type, ...fields };
      }
      return { type, value: fields };
    }
    // Might already be in { type: ..., ... } format
    if (raw.type) return raw;
  }
  return null;
}

test.describe('Player Agent Playtest', () => {
  test.beforeAll(async () => {
    // Ensure screenshot directory exists
    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
  });

  test('replay trace and capture screenshots', async ({ page }) => {
    // Navigate to the app
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle' });

    // Wait for the app to initialize (loading screen hidden)
    await page.waitForSelector('#loading.hidden', { timeout: 30000 });

    // Wait for WASM to be ready
    const wasmReady = await page.evaluate(() => {
      return (window as any).agentAPI?.isReady() ?? false;
    });

    if (!wasmReady) {
      console.warn('WASM not ready — running against mock data');
    }

    // Check if agent API is available
    const hasAPI = await page.evaluate(() => !!(window as any).agentAPI);
    expect(hasAPI).toBeTruthy();

    // Load the trace
    if (!fs.existsSync(TRACE_PATH)) {
      console.log(`No trace file at ${TRACE_PATH} — running default screenshot sequence`);
      await captureDefaultSequence(page);
      return;
    }

    const traceData = JSON.parse(fs.readFileSync(TRACE_PATH, 'utf-8'));
    const steps = traceData.steps || traceData;

    console.log(`Replaying ${steps.length} steps from trace...`);

    let screenshotCount = 0;
    for (let i = 0; i < steps.length; i++) {
      const step = steps[i];
      const action = parseAction(step.action);
      if (!action) continue;

      if (action.type === 'Screenshot') {
        // Capture the actual rendered frame
        const label = action.label || `screenshot-${screenshotCount}`;
        const filename = `${label}.png`;
        const filepath = path.join(SCREENSHOT_DIR, filename);

        // Wait for render to settle
        await page.waitForTimeout(200);

        await page.screenshot({
          path: filepath,
          type: 'png',
          fullPage: false,
        });

        console.log(`  📸 ${filename} (step ${i}, tick ${step.observation?.tick ?? '?'})`);
        screenshotCount++;
      } else {
        // Execute the action via agentAPI
        await page.evaluate(async (act) => {
          const api = (window as any).agentAPI;
          if (api) await api.executeAction(act);
        }, action);

        // Brief pause for actions that change the scene
        if (['Tick', 'Place', 'Fill'].includes(action.type)) {
          await page.waitForTimeout(100);
        }
      }
    }

    if (screenshotCount === 0) {
      console.log('Trace had no Screenshot actions — running default screenshot sequence');
      await captureDefaultSequence(page);
      return;
    }

    console.log(`\nDone: ${screenshotCount} screenshots saved to ${SCREENSHOT_DIR}`);
  });
});

/** Fallback: if no trace file, capture a default set of screenshots */
async function captureDefaultSequence(page: any) {
  const actions = [
    // Capture initial state
    { label: '01-initial', camera: { theta: 45, phi: 60 } },
    // Place some water and seeds, tick, screenshot
    { label: '02-planted', actions: [
      { type: 'Fill', tool: 'water', x1: 35, y1: 35, z1: 45, x2: 45, y2: 45, z2: 45 },
      { type: 'Place', tool: 'seed', x: 36, y: 40, z: 50, species: 'oak' },
      { type: 'Place', tool: 'seed', x: 44, y: 40, z: 50, species: 'fern' },
      { type: 'Place', tool: 'seed', x: 40, y: 36, z: 50, species: 'wildflower' },
      { type: 'Place', tool: 'seed', x: 40, y: 44, z: 50, species: 'moss' },
      { type: 'Tick', n: 5 },
    ]},
    // Tick to grow
    { label: '03-early-growth', actions: [{ type: 'Tick', n: 100 }] },
    // Different angle
    { label: '04-side-view', camera: { theta: 90, phi: 40, zoom: 1.5 } },
    // More growth
    { label: '05-mature', actions: [{ type: 'Tick', n: 200 }] },
    // Top-down
    { label: '06-topdown', camera: { theta: 45, phi: 80, zoom: 0.6 } },
  ];

  fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });

  for (const step of actions) {
    // Execute any pre-screenshot actions
    if ('actions' in step && step.actions) {
      for (const act of step.actions) {
        await page.evaluate(async (a: any) => {
          const api = (window as any).agentAPI;
          if (api) await api.executeAction(a);
        }, act);
        await page.waitForTimeout(50);
      }
    }

    // Set camera if specified
    if ('camera' in step && step.camera) {
      await page.evaluate(async (cam: any) => {
        const api = (window as any).agentAPI;
        if (api) {
          await api.executeAction({ type: 'CameraOrbit', theta_deg: cam.theta, phi_deg: cam.phi });
          if (cam.zoom) await api.executeAction({ type: 'CameraZoom', level: cam.zoom });
        }
      }, step.camera);
    }

    // Wait for render
    await page.waitForTimeout(300);

    // Screenshot
    const filepath = path.join(SCREENSHOT_DIR, `${step.label}.png`);
    await page.screenshot({ path: filepath, type: 'png' });
    console.log(`  📸 ${step.label}.png`);
  }
}
