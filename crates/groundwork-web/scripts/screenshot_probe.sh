#!/usr/bin/env bash
#
# screenshot_probe.sh — Probe the environment and recommend the fastest screenshot path.
#
# Run this BEFORE attempting screenshots. It checks what's available and tells you
# exactly what to do next. No installs, no side effects.
#
# Usage:
#   cd crates/groundwork-web && ./scripts/screenshot_probe.sh
#

set -uo pipefail

WEB_DIR="$(cd "$(dirname "$0")/.." && pwd)"

echo "=== Screenshot Environment Probe ==="
echo ""

# Track what's available
HAS_NODE=false
HAS_DEPS=false
HAS_BROWSER=false
HAS_DISPLAY=false
HAS_WASM=false
BROWSER_PATH=""
RECOMMENDED_PATH=""
MISSING=()

# --- Node.js ---
if command -v node &>/dev/null; then
  HAS_NODE=true
  echo "node:      $(node --version) ✓"
elif [ -x /opt/node22/bin/node ]; then
  HAS_NODE=true
  echo "node:      $(/opt/node22/bin/node --version) at /opt/node22/bin/node ✓"
  echo "           (add to PATH: export PATH=/opt/node22/bin:\$PATH)"
else
  echo "node:      NOT FOUND ✗"
  MISSING+=("node")
fi

# --- npm deps ---
if [ -d "$WEB_DIR/node_modules/three" ] && [ -d "$WEB_DIR/node_modules/playwright-core" ]; then
  HAS_DEPS=true
  echo "npm deps:  installed ✓"
else
  echo "npm deps:  NOT INSTALLED ✗"
  MISSING+=("npm-deps")
fi

# --- Browser (check in order of likelihood) ---
find_browser() {
  # Playwright cache (most common in CI/Claude Code)
  for dir in "$HOME/.cache/ms-playwright" "/root/.cache/ms-playwright"; do
    if [ -d "$dir" ]; then
      local found
      found=$(find "$dir" -name "chrome" -path "*/chrome-linux/*" -type f 2>/dev/null | head -1)
      if [ -n "$found" ] && [ -x "$found" ]; then
        BROWSER_PATH="$found"
        echo "browser:   $found (playwright cache) ✓"
        return 0
      fi
    fi
  done

  # System chromium
  for cmd in chromium-browser chromium google-chrome-stable google-chrome; do
    if command -v "$cmd" &>/dev/null; then
      BROWSER_PATH="$(command -v "$cmd")"
      echo "browser:   $BROWSER_PATH (system) ✓"
      return 0
    fi
  done

  # Puppeteer bundled
  local pup
  pup=$(find "$WEB_DIR/node_modules" /usr/lib/node_modules /usr/local/lib/node_modules \
    -path "*/puppeteer*/.local-chromium/*/chrome" -type f 2>/dev/null | head -1)
  if [ -n "$pup" ] && [ -x "$pup" ]; then
    BROWSER_PATH="$pup"
    echo "browser:   $pup (puppeteer) ✓"
    return 0
  fi

  return 1
}

if find_browser; then
  HAS_BROWSER=true
else
  echo "browser:   NOT FOUND ✗"
  MISSING+=("browser")
fi

# --- Display server ---
if [ -n "${DISPLAY:-}" ]; then
  HAS_DISPLAY=true
  echo "display:   DISPLAY=$DISPLAY ✓"
elif command -v xvfb-run &>/dev/null; then
  HAS_DISPLAY=true
  echo "display:   xvfb-run available ✓"
elif command -v Xvfb &>/dev/null; then
  HAS_DISPLAY=true
  echo "display:   Xvfb available (manual start needed) ✓"
else
  echo "display:   NO X SERVER ✗"
  echo "           (headless chrome may still work without it)"
  # Don't add to MISSING — might work anyway
fi

# --- WASM build ---
if [ -f "$WEB_DIR/wasm/groundwork_sim.js" ]; then
  HAS_WASM=true
  echo "wasm:      built ✓ (real sim available)"
else
  echo "wasm:      not built (mock mode — still captures full renderer)"
fi

# --- Recommendation ---
echo ""
echo "=== Recommended Path ==="
echo ""

if [ ${#MISSING[@]} -eq 0 ]; then
  echo "READY TO GO. Run:"
  echo ""
  echo "  npm run playtest"
  echo ""
  echo "Or for a quick single shot:"
  echo ""
  echo "  ./screenshot.sh --quick"
  echo ""
  if $HAS_WASM; then
    echo "Mode: WASM sim (real ecological simulation)"
  else
    echo "Mode: Mock data (full renderer, static scene)"
  fi
elif [[ " ${MISSING[*]} " == *" npm-deps "* ]] && [ ${#MISSING[@]} -eq 1 ]; then
  echo "Just need deps. Run:"
  echo ""
  echo "  npm install && npm run playtest"
  echo ""
elif [[ " ${MISSING[*]} " == *" browser "* ]] && [ ${#MISSING[@]} -eq 1 ]; then
  echo "Just need a browser. Run:"
  echo ""
  echo "  npm run playtest:install && npm run playtest"
  echo ""
elif [[ " ${MISSING[*]} " == *" browser "* ]] && [[ " ${MISSING[*]} " == *" npm-deps "* ]]; then
  echo "Need deps + browser. Run:"
  echo ""
  echo "  npm install && npm run playtest:install && npm run playtest"
  echo ""
elif [[ " ${MISSING[*]} " == *" node "* ]]; then
  echo "BLOCKED: No Node.js found. Cannot capture screenshots."
  echo "Install Node.js 18+ first."
fi

echo "---"
echo "Screenshots output to: artifacts/screenshots/"
echo "Attach to PR when done."
