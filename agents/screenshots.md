# Screenshot Capture — Agent Instructions

> **Goal:** Capture Three.js rendered PNGs from headless Chromium as fast as possible.
> Every environment is slightly different. Your job is to sniff what's available and pick the shortest path.

## Why This Matters

Visual evaluation is non-negotiable. The renderer is the game — if we can't see it, we can't ship it. Screenshots go on PRs, in feedback, in handoffs. Time-to-screenshot (TTS) matters because every minute spent fighting tooling is a minute not evaluating the actual scene.

## Mental Model

The screenshot pipeline has exactly three links:

```
  Vite dev server  →  Headless browser + WebGL  →  page.screenshot() → PNG
```

Vite is always available (it's a devDep). The browser is the variable. Your job is finding one.

## Environment Detection — Decision Tree

Run these checks **in order**. Stop at the first hit.

### 1. Playwright cache (TTS: ~15s, zero install)

```bash
find ~/.cache/ms-playwright /root/.cache/ms-playwright -name "chrome" -path "*/chrome-linux/*" -type f 2>/dev/null | head -1
```

**Why check first:** Claude Code environments, CI runners, and dev machines that have ever run `npx playwright install` will have a chromium binary cached here. This is the most common case. It doesn't need to match the installed `@playwright/test` version — `playwright-core` (our devDep) can drive any chromium.

**Gotcha:** The version in the cache may not match what `@playwright/test` expects. The `npx playwright test` runner will refuse it. That's why we use `playwright-core` directly (the `chromium.launch({ executablePath })` API), not the test runner. Version mismatch doesn't matter for `launch()`.

### 2. System chromium (TTS: ~15s, zero install)

```bash
which chromium-browser || which chromium || which google-chrome-stable || which google-chrome
# Also check: /usr/bin/chromium-browser, /snap/bin/chromium
```

**Why:** Some environments have system chromium from apt/snap/brew. Works identically to Playwright's — just pass the path to `executablePath`.

### 3. Puppeteer's bundled chromium (TTS: ~15s, zero install)

```bash
find /usr/lib/node_modules /usr/local/lib/node_modules -path "*/puppeteer*/.local-chromium/*/chrome" 2>/dev/null | head -1
```

**Why:** Some environments pre-install puppeteer globally. Its bundled chromium works fine with playwright-core.

### 4. Install chromium (TTS: ~60-120s)

Only if 1-3 all fail. Try in order:

```bash
npx playwright install chromium          # Downloads ~130MB, installs to cache
# OR if network to playwright CDN fails:
apt-get install -y chromium-browser      # Uses OS package manager
```

**Gotcha:** Network may be restricted (DNS failures, firewalls). If `playwright install` fails, try apt. If both fail, you cannot capture screenshots — say so clearly in your handoff rather than making excuses.

## Display Server

Headless Chromium with SwiftShader still needs an X server for WebGL context creation.

```bash
# Preferred: xvfb-run wraps the entire command
which xvfb-run && echo "use: xvfb-run -a <command>"

# Fallback: start Xvfb manually
which Xvfb && Xvfb :99 -screen 0 1920x1080x24 & export DISPLAY=:99

# Already have a display? (e.g. desktop environment, existing Xvfb)
echo $DISPLAY  # if set, you're fine
```

If none of these work, try running anyway — some newer headless Chrome versions don't need X. Worst case it fails with a clear error.

## The Capture Itself

Once you have a browser path, the capture is straightforward. Use `screenshot.sh` in `crates/groundwork-web/`:

```bash
cd crates/groundwork-web
./screenshot.sh --quick              # Single hero shot (~15s)
./screenshot.sh                       # 7 angles (~25s)
./screenshot.sh --angles "45,60 120,35 225,55"  # Custom theta,phi pairs
```

The script handles Vite startup, browser launch, and cleanup. It outputs to `artifacts/screenshots/`.

### What the script does (so you can adapt if it breaks)

1. Starts Vite on port 5174 (or reuses if already running)
2. Launches chromium via `playwright-core` with `--use-gl=angle --use-angle=swiftshader --no-sandbox`
3. Navigates to the app — WASM will fail (that's fine), mock grid loads automatically
4. Uses `window.agentAPI.executeAction()` to control camera angles
5. Calls `page.screenshot()` at each angle
6. Saves PNGs to `artifacts/screenshots/`

### If the script fails

The most common failures and fixes:

| Symptom | Cause | Fix |
|---------|-------|-----|
| `ERR_MODULE_NOT_FOUND: playwright-core` | node_modules not installed | `npm install` first |
| `Xvfb failed to start` | Stale X lock file | `rm -f /tmp/.X*-lock` then retry |
| Port in use | Stale Vite from previous run | `pkill -f vite` or use `--port 5175` |
| Black/blank screenshots | SwiftShader not working | Check chrome args include `--use-gl=angle --use-angle=swiftshader` |
| All screenshots identical | Camera not snapping | Agent API bug — check `snap()` is called after camera commands |
| `Cannot find chrome` | None of the 4 paths found a browser | See detection tree above |

## Mock Mode vs WASM

The app has two rendering paths:

- **WASM mode**: Real Rust sim compiled to WASM, full ecological simulation, growth, fauna
- **Mock mode**: JS-generated voxel grid with a static oak tree, water pond, soil layers

Both use the **exact same rendering pipeline** — meshing, materials, lighting, shaders, post-processing. Mock mode is what you'll get in most agent environments (no wasm-pack installed). The screenshots are real rendered frames of the actual production renderer.

If WASM *is* available (someone ran `npm run wasm`), the screenshots will show a live sim with growth and fauna. Better, but not required.

## What to Do With Screenshots

- **PR comments:** Upload PNGs directly to GitHub PR comments/descriptions
- **Handoffs:** Reference the files in `artifacts/screenshots/` — they're committed to the branch
- **Feedback:** "Here's what I see" beats "I couldn't get screenshots working" every time
- **Evaluation:** Look at the renders critically. Is the terrain readable? Is lighting warm? Are there obvious rendering bugs? A screenshot is only useful if you actually evaluate it.

## Quick Reference

```bash
# Full sequence from zero (assumes node + npm available)
cd crates/groundwork-web
npm install                    # once
./screenshot.sh --quick        # fastest path to one screenshot
ls artifacts/screenshots/      # verify output
```

The goal is a PNG on the PR. Everything else is implementation detail.
