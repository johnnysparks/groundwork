#!/usr/bin/env bash
#
# screenshot.sh — Environment-adaptive Three.js screenshot capture.
#
# Sniffs the current environment for the fastest path to headless browser
# screenshots. Installs nothing unless absolutely necessary.
#
# Usage:
#   ./screenshot.sh                  # Capture default 7-angle sequence
#   ./screenshot.sh --quick          # Single hero shot only (fastest)
#   ./screenshot.sh --angles "45,60 120,35 225,55"  # Custom angles (theta,phi pairs)
#
# Exit codes:
#   0 = screenshots captured successfully
#   1 = fatal error (no viable path found)
#
# Output: PNG files in artifacts/screenshots/
#
# --- Environment detection priority (fastest first) ---
#
# 1. CACHED PLAYWRIGHT BROWSER — check ~/.cache/ms-playwright/ for any
#    chromium install. This is the most common case in CI and Claude Code
#    environments. Zero install time.
#
# 2. SYSTEM CHROMIUM — check for chromium-browser, chromium, google-chrome,
#    google-chrome-stable on PATH or in common install locations.
#    Zero install time.
#
# 3. PUPPETEER CHROMIUM — check node_modules for puppeteer's bundled
#    chromium (some environments pre-install puppeteer globally).
#    Zero install time.
#
# 4. INSTALL CHROMIUM — as a last resort, try `npx playwright install chromium`
#    or `apt-get install -y chromium-browser`. Only if nothing else works.
#
# For all paths, playwright-core (already a devDep) drives the browser.
# The app's mock mode renders the full Three.js scene without WASM.
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
SCREENSHOT_DIR="${SCREENSHOT_DIR:-$PROJECT_ROOT/artifacts/screenshots}"
VITE_PORT="${VITE_PORT:-5174}"
QUICK=false
CUSTOM_ANGLES=""

# Parse args
while [[ $# -gt 0 ]]; do
  case "$1" in
    --quick) QUICK=true; shift ;;
    --angles) CUSTOM_ANGLES="$2"; shift 2 ;;
    --port) VITE_PORT="$2"; shift 2 ;;
    --out) SCREENSHOT_DIR="$2"; shift 2 ;;
    *) echo "Unknown arg: $1"; exit 1 ;;
  esac
done

# --- Step 1: Find Node.js ---

NODE=""
if command -v node &>/dev/null; then
  NODE="$(command -v node)"
elif [ -x /opt/node22/bin/node ]; then
  NODE="/opt/node22/bin/node"
  export PATH="/opt/node22/bin:$PATH"
fi

if [ -z "$NODE" ]; then
  echo "FATAL: Node.js not found. Cannot capture screenshots."
  exit 1
fi
echo "node: $($NODE --version) at $NODE"

# --- Step 2: Ensure npm deps (three, vite, playwright-core) ---

if [ ! -d "$SCRIPT_DIR/node_modules/three" ]; then
  echo "Installing npm dependencies..."
  (cd "$SCRIPT_DIR" && npm install --prefer-offline --no-audit --no-fund 2>&1 | tail -3)
fi

# --- Step 3: Find a browser ---

BROWSER_PATH=""
BROWSER_SOURCE=""

find_browser() {
  # 0. Explicit override via env var
  if [ -n "${BROWSER_PATH:-}" ] && [ -x "$BROWSER_PATH" ]; then
    BROWSER_SOURCE="env"
    return 0
  fi

  # 3a. Playwright cache (any version)
  # macOS: ~/Library/Caches/ms-playwright, Linux: ~/.cache/ms-playwright
  local pw_cache_dirs=()
  if [ "$(uname)" = "Darwin" ]; then
    pw_cache_dirs+=("$HOME/Library/Caches/ms-playwright")
  else
    pw_cache_dirs+=("$HOME/.cache/ms-playwright")
    # CI/agent environments often run as root
    if [ "$HOME" != "/root" ] && [ -d "/root/.cache/ms-playwright" ]; then
      pw_cache_dirs+=("/root/.cache/ms-playwright")
    fi
  fi

  for pw_dir in "${pw_cache_dirs[@]}"; do
    if [ -d "$pw_dir" ]; then
      local found=""
      if [ "$(uname)" = "Darwin" ]; then
        found=$(find "$pw_dir" -name "Chromium" -path "*/Chromium.app/Contents/MacOS/*" -type f 2>/dev/null | head -1)
      else
        found=$(find "$pw_dir" -name "chrome" -path "*/chrome-linux/*" -type f 2>/dev/null | head -1)
      fi
      if [ -n "$found" ] && [ -x "$found" ]; then
        BROWSER_PATH="$found"
        BROWSER_SOURCE="playwright-cache"
        return 0
      fi
    fi
  done

  # 3b. System chromium/chrome
  for cmd in chromium-browser chromium google-chrome-stable google-chrome; do
    if command -v "$cmd" &>/dev/null; then
      BROWSER_PATH="$(command -v "$cmd")"
      BROWSER_SOURCE="system"
      return 0
    fi
  done
  # Common non-PATH locations
  for p in /usr/bin/chromium-browser /usr/bin/chromium /usr/bin/google-chrome /snap/bin/chromium; do
    if [ -x "$p" ]; then
      BROWSER_PATH="$p"
      BROWSER_SOURCE="system-path"
      return 0
    fi
  done

  # 3c. Puppeteer's bundled chromium
  local puppeteer_chrome
  puppeteer_chrome=$(find "$SCRIPT_DIR/node_modules" /usr/lib/node_modules /usr/local/lib/node_modules \
    -path "*/puppeteer*/.local-chromium/*/chrome" -type f 2>/dev/null | head -1)
  if [ -n "$puppeteer_chrome" ] && [ -x "$puppeteer_chrome" ]; then
    BROWSER_PATH="$puppeteer_chrome"
    BROWSER_SOURCE="puppeteer"
    return 0
  fi

  return 1
}

if ! find_browser; then
  echo "No browser found. Attempting minimal install..."

  # Try playwright install (uses existing playwright-core dep, just downloads the binary)
  if npx playwright install chromium 2>/dev/null; then
    find_browser || true
  fi

  # Try apt as last resort
  if [ -z "$BROWSER_PATH" ] && command -v apt-get &>/dev/null; then
    echo "Trying apt-get install chromium-browser..."
    apt-get install -y --no-install-recommends chromium-browser 2>/dev/null || true
    find_browser || true
  fi

  if [ -z "$BROWSER_PATH" ]; then
    echo "FATAL: Could not find or install a browser."
    echo "Manual fix: install chromium or run 'npx playwright install chromium'"
    exit 1
  fi
fi

echo "browser: $BROWSER_PATH ($BROWSER_SOURCE)"

# --- Step 4: Check for Xvfb (needed for software GL) ---

XVFB_CMD=""
if command -v xvfb-run &>/dev/null; then
  XVFB_CMD="xvfb-run -a -s '-screen 0 1920x1080x24'"
  echo "display: xvfb-run"
elif [ -n "${DISPLAY:-}" ]; then
  echo "display: $DISPLAY (existing)"
else
  # Try to start Xvfb manually
  if command -v Xvfb &>/dev/null; then
    export DISPLAY=:99
    Xvfb :99 -screen 0 1920x1080x24 &>/dev/null &
    XVFB_PID=$!
    sleep 1
    echo "display: Xvfb :99 (started, pid $XVFB_PID)"
  else
    echo "WARNING: No display server found. Headless Chrome may still work."
    echo "display: none"
  fi
fi

# --- Step 5: Run the capture ---

mkdir -p "$SCREENSHOT_DIR"

echo ""
echo "=== Capturing screenshots ==="
echo "  browser: $BROWSER_PATH"
echo "  output:  $SCREENSHOT_DIR"
echo "  quick:   $QUICK"
echo ""

# Build the node command — inline to avoid needing a separate .mjs file
# that might not be in sync. This is the single source of truth.
CAPTURE_SCRIPT=$(cat <<'NODESCRIPT'
import { chromium } from 'playwright-core';
import { spawn } from 'child_process';
import fs from 'fs';
import path from 'path';

const BROWSER_PATH = process.env.BROWSER_PATH;
const SCREENSHOT_DIR = process.env.SCREENSHOT_DIR;
const VITE_PORT = parseInt(process.env.VITE_PORT || '5174');
const QUICK = process.env.QUICK === 'true';
const CUSTOM_ANGLES = process.env.CUSTOM_ANGLES || '';
const WEB_DIR = process.env.WEB_DIR;

// --- Start Vite ---
async function startVite() {
  // Check if something is already serving on the port
  try {
    const resp = await fetch(`http://localhost:${VITE_PORT}`);
    if (resp.ok || resp.status === 404) {
      console.log(`Vite already running on port ${VITE_PORT}`);
      return null; // no process to manage
    }
  } catch {}

  console.log('Starting Vite dev server...');
  const vite = spawn('npx', ['vite', '--port', String(VITE_PORT)], {
    cwd: WEB_DIR, stdio: ['pipe', 'pipe', 'pipe'], env: { ...process.env },
  });
  await new Promise((resolve, reject) => {
    const t = setTimeout(() => reject(new Error('Vite timeout')), 30000);
    vite.stdout.on('data', d => {
      const s = d.toString();
      if (s.includes('Local:') || s.includes('ready in')) { clearTimeout(t); setTimeout(resolve, 500); }
    });
    vite.stderr.on('data', d => process.stderr.write(d));
    vite.on('error', e => { clearTimeout(t); reject(e); });
  });
  // Detect actual port (Vite may pick a different one if ours is taken)
  return vite;
}

// --- Main ---
async function main() {
  fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
  const viteProcess = await startVite();
  let browser;
  try {
    browser = await chromium.launch({
      executablePath: BROWSER_PATH,
      headless: true,
      args: [
        '--use-gl=angle', '--use-angle=swiftshader',
        '--no-sandbox', '--disable-setuid-sandbox',
        '--disable-dev-shm-usage', '--disable-gpu-sandbox',
      ],
    });
    const page = await (await browser.newContext({ viewport: { width: 1920, height: 1080 } })).newPage();
    page.on('console', m => { if (m.type() === 'error') console.log(`  [browser] ${m.text()}`); });

    await page.goto(`http://localhost:${VITE_PORT}`, { waitUntil: 'networkidle', timeout: 30000 });
    try { await page.waitForSelector('#loading.hidden', { timeout: 15000 }); } catch {}

    const hasAPI = await page.evaluate(() => !!window.agentAPI);
    const wasmReady = await page.evaluate(() => window.agentAPI?.isReady?.() ?? false);
    console.log(`agentAPI: ${hasAPI}, WASM: ${wasmReady}`);

    // Wait for first render
    await page.evaluate(async () => {
      for (let i = 0; i < 5; i++) await new Promise(r => requestAnimationFrame(r));
    });
    await page.waitForTimeout(500);

    const exec = (action) => page.evaluate(async a => {
      const api = window.agentAPI; if (api) await api.executeAction(a);
    }, action);

    // Advance sim so fauna can spawn and growth can progress
    if (wasmReady) {
      // Plant some flowers for pollinators before ticking
      await exec({ type: 'Place', tool: 'seed', x: 42, y: 42, z: 50, species: 'wildflower' });
      await exec({ type: 'Place', tool: 'seed', x: 38, y: 42, z: 50, species: 'wildflower' });
      await exec({ type: 'Place', tool: 'seed', x: 40, y: 38, z: 50, species: 'daisy' });
      await exec({ type: 'Place', tool: 'seed', x: 42, y: 38, z: 50, species: 'clover' });
      await exec({ type: 'Place', tool: 'seed', x: 38, y: 38, z: 50, species: 'moss' });
      console.log('Advancing sim 200 ticks for fauna/growth...');
      await exec({ type: 'Tick', n: 200 });
      await page.waitForTimeout(200);
      const info = await page.evaluate(() => {
        const api = window.agentAPI;
        if (!api) return 'no api';
        return `tick=${api.getTick()}, fauna=${api.getFaunaCount()}`;
      });
      console.log(`  ${info}`);
    }

    const snap = async (name) => {
      await page.evaluate(async () => {
        for (let i = 0; i < 3; i++) await new Promise(r => requestAnimationFrame(r));
      });
      await page.waitForTimeout(100);
      const fp = path.join(SCREENSHOT_DIR, `${name}.png`);
      await page.screenshot({ path: fp, type: 'png' });
      console.log(`  ${name}.png (${(fs.statSync(fp).size / 1024).toFixed(0)}KB)`);
    };

    // Build angle list
    let angles;
    if (CUSTOM_ANGLES) {
      angles = CUSTOM_ANGLES.split(/\s+/).map((pair, i) => {
        const [t, p] = pair.split(',').map(Number);
        return { name: `angle-${String(i+1).padStart(2,'0')}`, theta: t, phi: p };
      });
    } else if (QUICK) {
      angles = [{ name: 'hero', theta: 45, phi: 60 }];
    } else {
      // Standard tour: surface views, lighting, x-ray
      const shots = [
        // --- Surface (golden hour default) ---
        { name: '01-hero',           theta: 45,  phi: 60 },
        { name: '02-side',           theta: 120, phi: 35 },
        { name: '03-close-up',       theta: 30,  phi: 50, zoom: 2.0 },
        { name: '04-wide',           theta: 45,  phi: 85, zoom: 0.3 },
        // --- Day cycle: 4 times of day from same angle ---
        { name: '05-dawn',           theta: 45,  phi: 60, time: 0.25 },
        { name: '06-noon',           theta: 45,  phi: 60, time: 0.5 },
        { name: '07-golden',         theta: 45,  phi: 60, time: 0.75 },
        { name: '08-blue-hour',      theta: 45,  phi: 60, time: 0.0 },
        // --- X-ray underground (reset to golden hour) ---
        { name: '09-xray-hero',      theta: 45,  phi: 60, xray: true, time: 0.75 },
        { name: '10-xray-side',      theta: 120, phi: 35, xray: true },
        { name: '11-xray-close',     theta: 30,  phi: 50, zoom: 2.0, xray: true },
      ];
      angles = shots;
    }

    let xrayOn = false;
    for (const a of angles) {
      // Toggle x-ray mode if needed
      const wantXray = !!a.xray;
      if (wantXray !== xrayOn) {
        await exec({ type: 'CameraCutaway', z: wantXray ? 1 : 0 });
        xrayOn = wantXray;
        await page.waitForTimeout(200); // let transparency settle
      }
      // Set time of day if specified
      if (a.time !== undefined) {
        await page.evaluate((t) => { window.agentAPI?.setTimeOfDay(t); }, a.time);
        // Need a frame for lighting to update
        await page.evaluate(async () => {
          for (let i = 0; i < 5; i++) await new Promise(r => requestAnimationFrame(r));
        });
        await page.waitForTimeout(100);
      }
      await exec({ type: 'CameraOrbit', theta_deg: a.theta, phi_deg: a.phi });
      if (a.zoom) await exec({ type: 'CameraZoom', level: a.zoom });
      else if (angles.indexOf(a) > 0) await exec({ type: 'CameraZoom', level: 1.0 });
      await snap(a.name);
    }
    // Disable x-ray if left on
    if (xrayOn) await exec({ type: 'CameraCutaway', z: 0 });

    console.log(`\nDone: ${angles.length} screenshots → ${SCREENSHOT_DIR}`);
  } finally {
    if (browser) await browser.close();
    if (viteProcess) viteProcess.kill('SIGTERM');
  }
}
main().catch(e => { console.error(e); process.exit(1); });
NODESCRIPT
)

# Write the inline script to a temp file IN the project dir (for node_modules resolution)
TMPSCRIPT="$SCRIPT_DIR/.capture-tmp-$$.mjs"
echo "$CAPTURE_SCRIPT" > "$TMPSCRIPT"

export BROWSER_PATH SCREENSHOT_DIR VITE_PORT QUICK CUSTOM_ANGLES
export WEB_DIR="$SCRIPT_DIR"

RUNCMD="$NODE $TMPSCRIPT"

if [ -n "$XVFB_CMD" ]; then
  eval "$XVFB_CMD $RUNCMD"
else
  eval "$RUNCMD"
fi

EXIT=$?
rm -f "$TMPSCRIPT"

# Cleanup Xvfb if we started it
if [ -n "${XVFB_PID:-}" ]; then
  kill "$XVFB_PID" 2>/dev/null || true
fi

if [ $EXIT -eq 0 ]; then
  echo ""
  echo "Screenshots ready:"
  ls -la "$SCREENSHOT_DIR"/*.png 2>/dev/null
fi

exit $EXIT
