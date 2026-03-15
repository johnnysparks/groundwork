# Screenshot TTS (Time-to-Screenshot)

Use this when a session needs **visual evidence of the Three.js scene** on a PR.

## Flow

```
1) Probe  →  2) Capture  →  3) Attach to PR
```

### 1. Environment probe (no installs)

```bash
cd crates/groundwork-web && ./scripts/screenshot_probe.sh
```

Copy the output into your notes. Follow its recommended path. Do not install `wasm-pack` unless real-sim visuals are strictly required for this task.

### 2. Capture — fastest path first

**Primary (one command, ~50s):**
```bash
cd crates/groundwork-web && xvfb-run -a npm run playtest
```
This starts Vite automatically, launches headless Chromium, captures 6 screenshots from multiple camera angles, and saves them to `artifacts/screenshots/`.

If `xvfb-run` is unavailable and `$DISPLAY` is set, drop it:
```bash
npm run playtest
```

**Quick single shot (~15s):**
```bash
cd crates/groundwork-web && ./screenshot.sh --quick
```

**Custom angles (theta,phi pairs):**
```bash
./screenshot.sh --angles "45,60 120,35 225,55"
```

### 3. Minimal installs (only if probe says needed)

Playwright is **not** in the base `npm install` — it's only needed for screenshots. This keeps `npm install` fast for dev/build/doc work.

| Probe says | Run |
|---|---|
| npm deps missing | `npm install` |
| Playwright missing | `npm run playtest:install` (installs Playwright + browser) |
| Both missing | `npm install && npm run playtest:install` |
| Node missing | **Blocked.** Install Node 18+ first. |

Avoid `wasm-pack` by default. The app screenshots in mock mode — same renderer, static scene.

### 4. When real simulation visuals are mandatory

Only if the task specifically requires live growth, fauna, or ecological interactions:

```bash
cd crates/groundwork-web && npm run wasm && xvfb-run -a npm run playtest
```

### 5. PR requirements

In the PR body or a comment:
- State the mode: `WASM sim` or `Mock data`
- Upload at least 1 screenshot PNG from `artifacts/screenshots/`
- Include one observation about what's visible in the scene (e.g. "water pond visible with translucent shader, oak trunk and foliage rendering, golden hour lighting active")
- Include the command you ran

### 6. Done condition

At least one screenshot is attached to the PR. A reviewer can visually evaluate the scene without running code.

---

## Why this flow works

- The web client has a **mock fallback** when WASM isn't built — full Three.js renderer, static scene. Screenshot capture works without `wasm-pack`.
- `npm run playtest` is a one-command path: Vite dev server + Playwright browser automation + PNG output.
- `playwright.config.ts` auto-detects any available Chromium (Playwright cache, system, puppeteer) via `findChromium()`. Version mismatches don't block it.
- The Playwright browser binary is usually the only missing piece — `npm run playtest:install` fixes it in ~60s.

## How the detection works (for debugging)

The probe checks four places for a browser, in order:

1. **Playwright cache** (`~/.cache/ms-playwright/`) — most CI and Claude Code environments have this from a previous `npx playwright install`. Zero install time.
2. **System chromium** — `chromium-browser`, `chromium`, `google-chrome` on PATH. Zero install time.
3. **Puppeteer bundle** — some environments pre-install puppeteer with a bundled chromium.
4. **Install** — `npx playwright install chromium` downloads ~130MB. Last resort.

Display server: `xvfb-run` (preferred) > existing `$DISPLAY` > manual `Xvfb :99` > hope headless works without it.

## Common failures

| Symptom | Fix |
|---|---|
| `No tests found` | Check `@types/node` is installed: `npm install` |
| `Executable doesn't exist` | Playwright can't find browser — run `./scripts/screenshot_probe.sh` to see what's available |
| `Xvfb failed to start` | Stale lock: `rm -f /tmp/.X*-lock` |
| Port in use | `pkill -f vite` or set `VITE_PORT=5175` |
| All screenshots identical | Camera snap not working — check `agent-api.ts` calls `_orbitCamera.snap()` |
| Blank/black screenshots | SwiftShader issue — verify chrome args include `--use-gl=angle --use-angle=swiftshader` |
