import { defineConfig } from '@playwright/test';
import { execSync } from 'child_process';

/**
 * Find the best available Chromium binary.
 * Priority: CHROMIUM_PATH env var > Playwright cache (any version) > system browser.
 *
 * This works around version-mismatch errors when the cached Playwright browser
 * doesn't match the installed @playwright/test version (common in CI/agent
 * environments where the browser was installed by a previous session).
 *
 * Returns undefined to let Playwright use its default when:
 * - The exact matching version IS installed (no workaround needed)
 * - We're on an OS where the search paths don't apply
 */
function findChromium(): string | undefined {
  // Explicit override always wins
  if (process.env.CHROMIUM_PATH) return process.env.CHROMIUM_PATH;

  const platform = process.platform;
  const home = process.env.HOME || '';
  const cacheDir = platform === 'darwin'
    ? home + '/Library/Caches/ms-playwright'
    : home + '/.cache/ms-playwright';

  const searchDirs = [cacheDir];
  // CI/agent environments often run as root
  if (platform === 'linux' && home !== '/root') {
    searchDirs.push('/root/.cache/ms-playwright');
  }

  for (const base of searchDirs) {
    try {
      if (platform === 'linux') {
        const result = execSync(
          `find "${base}" -name "chrome" -path "*/chrome-linux/*" -type f 2>/dev/null | head -1`,
          { encoding: 'utf-8' },
        ).trim();
        if (result) return result;
      } else if (platform === 'darwin') {
        const result = execSync(
          `find "${base}" -name "Chromium" -path "*/Chromium.app/Contents/MacOS/*" -type f 2>/dev/null | head -1`,
          { encoding: 'utf-8' },
        ).trim();
        if (result) return result;
      }
      // Windows: let Playwright handle it (browser is in AppData, path is complex)
    } catch {}
  }

  // System browser (Linux/macOS only — Windows users should use Playwright's default)
  if (platform !== 'win32') {
    for (const cmd of ['chromium-browser', 'chromium', 'google-chrome-stable', 'google-chrome']) {
      try {
        const result = execSync(`which ${cmd} 2>/dev/null`, { encoding: 'utf-8' }).trim();
        if (result) return result;
      } catch {}
    }
  }

  return undefined; // let Playwright try its default
}

const chromiumPath = findChromium();

export default defineConfig({
  testDir: './e2e',
  testMatch: '**/*.ts',
  timeout: 120_000, // 2 minutes per test (sim + screenshots take time)
  use: {
    baseURL: 'http://localhost:5173',
    // Headless by default; set HEADED=1 to see the browser
    headless: !process.env.HEADED,
    // Large viewport for good screenshots
    viewport: { width: 1920, height: 1080 },
    // --no-sandbox for CI/container environments (no-op on macOS/Windows)
    // SwiftShader software GL for headless; real GPU used in headed mode
    launchOptions: {
      executablePath: chromiumPath,
      args: [
        '--no-sandbox', '--disable-setuid-sandbox',
        '--disable-dev-shm-usage',
        ...(process.env.HEADED ? [] : ['--use-gl=angle', '--use-angle=swiftshader']),
      ],
    },
  },
  // Start the dev server automatically
  webServer: {
    command: 'npm run dev',
    port: 5173,
    reuseExistingServer: true,
    timeout: 30_000,
  },
});
