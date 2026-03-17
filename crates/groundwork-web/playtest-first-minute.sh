#!/usr/bin/env bash
#
# playtest-first-minute.sh — Granular first-minute playtest
#
# Captures a screenshot every ~5 seconds of gameplay (50 ticks)
# to build a frame-by-frame understanding of the new player experience.
#
# Output: artifacts/playtest-first-minute/*.png
#

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

OUTPUT_DIR="$SCRIPT_DIR/artifacts/playtest-first-minute"
mkdir -p "$OUTPUT_DIR"

# Find browser
BROWSER_PATH=""
for dir in "$HOME/Library/Caches/ms-playwright" "$HOME/.cache/ms-playwright"; do
  if [ -d "$dir" ]; then
    found=$(find "$dir" -name "Chromium" -o -name "chromium" -o -name "chrome" 2>/dev/null | head -1)
    if [ -n "$found" ] && [ -x "$found" ]; then
      BROWSER_PATH="$found"
      break
    fi
  fi
done

if [ -z "$BROWSER_PATH" ]; then
  echo "No browser found"
  exit 1
fi

echo "Browser: $BROWSER_PATH"
echo "Output: $OUTPUT_DIR"

# Run the capture script
BROWSER_PATH="$BROWSER_PATH" OUTPUT_DIR="$OUTPUT_DIR" WEB_DIR="$SCRIPT_DIR" \
  node --input-type=module <<'NODESCRIPT'
import { chromium } from 'playwright-core';
import { spawn } from 'child_process';
import fs from 'fs';

const BROWSER_PATH = process.env.BROWSER_PATH;
const OUTPUT_DIR = process.env.OUTPUT_DIR;
const WEB_DIR = process.env.WEB_DIR;
const PORT = 5179;

// Start vite
let viteProcess = null;
try {
  const resp = await fetch(`http://localhost:${PORT}`);
  if (resp.ok) console.log('Vite already running');
} catch {
  console.log('Starting Vite...');
  viteProcess = spawn('npx', ['vite', '--port', String(PORT)], {
    cwd: WEB_DIR, stdio: ['pipe', 'pipe', 'pipe'], env: { ...process.env },
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
  executablePath: BROWSER_PATH,
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

console.log('WASM ready. Starting first-minute playtest...\n');

const exec = (action) => page.evaluate(async a => {
  const api = window.agentAPI;
  if (api) return await api.executeAction(a);
}, action);

const snap = async (name, description) => {
  // Wait for render
  await page.evaluate(async () => {
    for (let i = 0; i < 3; i++) await new Promise(r => requestAnimationFrame(r));
  });
  await page.waitForTimeout(200);
  const fp = `${OUTPUT_DIR}/${name}.png`;
  await page.screenshot({ path: fp });
  const kb = (fs.statSync(fp).size / 1024).toFixed(0);
  console.log(`  ${name} (${kb}KB) — ${description}`);
};

// Set noon lighting for consistent visuals
await exec({ type: 'Tick', n: 1 });

// ─── THE FIRST MINUTE ───────────────────────────────────────────

// t=0: Fresh garden — what does the player see first?
await snap('t000_fresh_start', 'First frame: empty garden, quest panel visible');

// Player reads quest: "Look around" — pan the camera
await exec({ type: 'CameraOrbit', theta_deg: 30, phi_deg: 55 });
await snap('t000_look_around', 'Player pans camera to explore');

// t=5s: Player finds the water spring (quest 2)
await exec({ type: 'Tick', n: 50 });
await snap('t050_five_seconds', '5 seconds in: empty garden, looking for spring');

// Player places water near the spring
await exec({ type: 'CameraOrbit', theta_deg: 45, phi_deg: 60 });
const cx = 40, cy = 40, gl = await page.evaluate(() => window.agentAPI?.getGroundLevel?.() ?? 50);
await exec({ type: 'Fill', tool: 'water', x1: cx-3, y1: cy-3, z1: gl+1, x2: cx+3, y2: cy+3, z2: gl+3 });
await exec({ type: 'Tick', n: 10 });
await snap('t060_water_placed', '6 seconds: player waters the garden');

// t=10s: Player plants first seed (oak)
await exec({ type: 'Place', tool: 'seed', x: cx, y: cy, z: gl+4, species: 'oak' });
await exec({ type: 'Tick', n: 10 });
await snap('t070_seed_planted', '7 seconds: first oak seed planted');

// Plant a few more species for variety
await exec({ type: 'Place', tool: 'seed', x: cx-4, y: cy+3, z: gl+4, species: 'birch' });
await exec({ type: 'Place', tool: 'seed', x: cx+3, y: cy-2, z: gl+4, species: 'wildflower' });
await exec({ type: 'Place', tool: 'seed', x: cx-2, y: cy-3, z: gl+4, species: 'clover' });
await exec({ type: 'Place', tool: 'seed', x: cx+4, y: cy+4, z: gl+4, species: 'grass' });
await exec({ type: 'Tick', n: 10 });
await snap('t080_seeds_in', '8 seconds: 5 seeds planted, garden has seeds visible');

// t=10s: Seeds settling
await exec({ type: 'Tick', n: 20 });
await snap('t100_ten_seconds', '10 seconds: seeds should be visible as mounds');

// t=15s: First trunks should appear
await exec({ type: 'Tick', n: 50 });
await snap('t150_fifteen_seconds', '15 seconds: first trunks emerging?');

// t=20s: Sapling stage — FIRST LEAVES?
await exec({ type: 'Tick', n: 50 });
await snap('t200_twenty_seconds', '20 seconds: saplings should show first leaves');

// Close-up to see detail
await exec({ type: 'CameraZoom', level: 2.5 });
await exec({ type: 'CameraPan', x: cx, y: cy, z: gl+3 });
await snap('t200_closeup', '20 seconds close-up: can we see green?');

// t=25s: Growth particles, early canopy
await exec({ type: 'CameraZoom', level: 1.0 });
await exec({ type: 'Tick', n: 50 });
await snap('t250_twentyfive', '25 seconds: young trees growing');

// t=30s: Half-minute mark
await exec({ type: 'Tick', n: 50 });
await snap('t300_thirty_seconds', '30 seconds: half-minute — garden should look alive');

// Close-up at 30s
await exec({ type: 'CameraZoom', level: 2.0 });
await snap('t300_closeup', '30 seconds close-up: species variety visible?');

// t=35s
await exec({ type: 'CameraZoom', level: 1.0 });
await exec({ type: 'Tick', n: 50 });
await snap('t350_thirtyfive', '35 seconds: canopy developing');

// t=40s
await exec({ type: 'Tick', n: 50 });
await snap('t400_forty_seconds', '40 seconds: trees should be recognizable');

// t=45s — fauna should start appearing
await exec({ type: 'Tick', n: 50 });
const faunaCount = await page.evaluate(() => window.agentAPI?.getFaunaCount?.() ?? 0);
await snap('t450_fortyfive', `45 seconds: fauna count = ${faunaCount}`);

// t=50s
await exec({ type: 'Tick', n: 50 });
await snap('t500_fifty_seconds', '50 seconds: approaching one minute');

// t=55s
await exec({ type: 'Tick', n: 50 });
await snap('t550_fiftyfive', '55 seconds: garden should be lush');

// t=60s — THE ONE MINUTE MARK
await exec({ type: 'Tick', n: 50 });
const finalFauna = await page.evaluate(() => window.agentAPI?.getFaunaCount?.() ?? 0);
const finalTick = await page.evaluate(() => window.agentAPI?.getTick?.() ?? 0);
await snap('t600_one_minute', `60 seconds (tick ${finalTick}): fauna=${finalFauna} — does this garden make you want to keep playing?`);

// Final close-up
await exec({ type: 'CameraZoom', level: 2.5 });
await snap('t600_closeup', 'One minute close-up: detail and life');

// Side view
await exec({ type: 'CameraZoom', level: 1.0 });
await exec({ type: 'CameraOrbit', theta_deg: 90, phi_deg: 45 });
await snap('t600_side', 'One minute side view: tree silhouettes');

// Top-down
await exec({ type: 'CameraOrbit', theta_deg: 45, phi_deg: 85 });
await snap('t600_topdown', 'One minute top-down: garden layout');

console.log('\n=== First minute playtest complete ===');
console.log(`Final state: tick=${finalTick}, fauna=${finalFauna}`);

await browser.close();
if (viteProcess) viteProcess.kill();
NODESCRIPT

echo ""
echo "Screenshots in: $OUTPUT_DIR"
ls -la "$OUTPUT_DIR"/*.png 2>/dev/null | tail -20
