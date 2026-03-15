import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  timeout: 120_000, // 2 minutes per test (sim + screenshots take time)
  use: {
    baseURL: 'http://localhost:5173',
    // Headless by default; set HEADED=1 to see the browser
    headless: !process.env.HEADED,
    // Large viewport for good screenshots
    viewport: { width: 1920, height: 1080 },
    // WebGL needs a real GPU context
    launchOptions: {
      args: ['--use-gl=angle', '--use-angle=swiftshader'],
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
