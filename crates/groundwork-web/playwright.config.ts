import { defineConfig } from '@playwright/test';
import { execSync } from 'child_process';

/**
 * Find the best available Chromium binary.
 * Priority: CHROMIUM_PATH env var > Playwright cache (any version) > system chromium.
 * This avoids version-mismatch errors when the cached browser doesn't match
 * the installed @playwright/test version.
 */
function findChromium(): string | undefined {
  if (process.env.CHROMIUM_PATH) return process.env.CHROMIUM_PATH;

  // Search Playwright cache for any chromium
  for (const base of [process.env.HOME + '/.cache/ms-playwright', '/root/.cache/ms-playwright']) {
    try {
      const result = execSync(
        `find ${base} -name "chrome" -path "*/chrome-linux/*" -type f 2>/dev/null | head -1`,
        { encoding: 'utf-8' },
      ).trim();
      if (result) return result;
    } catch {}
  }

  // System chromium
  for (const cmd of ['chromium-browser', 'chromium', 'google-chrome-stable', 'google-chrome']) {
    try {
      const result = execSync(`which ${cmd} 2>/dev/null`, { encoding: 'utf-8' }).trim();
      if (result) return result;
    } catch {}
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
    // WebGL via software renderer; --no-sandbox for CI/container environments
    launchOptions: {
      executablePath: chromiumPath,
      args: [
        '--use-gl=angle', '--use-angle=swiftshader',
        '--no-sandbox', '--disable-setuid-sandbox',
        '--disable-dev-shm-usage',
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
