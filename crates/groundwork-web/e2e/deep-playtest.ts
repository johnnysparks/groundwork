/**
 * Deep playtest — comprehensive visual evaluation of the web renderer.
 *
 * Captures screenshots covering:
 * - Initial view, multiple angles
 * - X-ray underground view with root glow
 * - Close-up of trees and foliage
 * - Water rendering
 * - HUD and tool palette
 * - Different times of day
 */

import { test, expect } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const SCREENSHOT_DIR = process.env.SCREENSHOT_DIR
  || path.resolve(__dirname, '../../../artifacts/screenshots/deep-playtest');

test.describe('Deep Playtest', () => {
  test.beforeAll(async () => {
    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
  });

  test('comprehensive visual evaluation', async ({ page }) => {
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle' });
    await page.waitForSelector('#loading.hidden', { timeout: 30000 });

    const hasAPI = await page.evaluate(() => !!(window as any).agentAPI);
    expect(hasAPI).toBeTruthy();

    async function screenshot(label: string) {
      await page.waitForTimeout(400);
      const filepath = path.join(SCREENSHOT_DIR, `${label}.png`);
      await page.screenshot({ path: filepath, type: 'png' });
      console.log(`  📸 ${label}.png`);
    }

    async function execAction(action: any) {
      await page.evaluate(async (a: any) => {
        const api = (window as any).agentAPI;
        if (api) await api.executeAction(a);
      }, action);
      await page.waitForTimeout(100);
    }

    // === 0. BUILD PHASE — plant a diverse garden for ecological drama ===
    // The default garden has a dense starter garden (21 seeds). Add more seeds
    // in diverse conditions to create competition, succession, and synergy.

    // Plant near water (good conditions)
    await execAction({ type: 'Fill', tool: 'seed', x1: 32, y1: 38, z1: 50, x2: 36, y2: 42, z2: 50 });
    // Plant far from water (will stress for water)
    await execAction({ type: 'Fill', tool: 'seed', x1: 55, y1: 55, z1: 50, x2: 60, y2: 60, z2: 50 });
    // Dense cluster for competition
    await execAction({ type: 'Fill', tool: 'seed', x1: 42, y1: 30, z1: 50, x2: 48, y2: 36, z2: 50 });
    // Dig an irrigation channel from the spring toward the dry area
    await execAction({ type: 'Fill', tool: 'dig', x1: 45, y1: 45, z1: 49, x2: 55, y2: 55, z2: 49 });

    // Grow for 300 ticks — enough for seeds to germinate and trees to start competing
    await execAction({ type: 'Tick', n: 300 });

    // === 1. INITIAL STATE — default view ===
    await screenshot('01-initial-default');

    // === 2. ORBIT — different angles ===
    await execAction({ type: 'CameraOrbit', theta_deg: 0, phi_deg: 55 });
    await screenshot('02-front-view');

    await execAction({ type: 'CameraOrbit', theta_deg: 180, phi_deg: 55 });
    await screenshot('03-back-view');

    // === 3. CLOSE-UP — zoom into the largest tree ===
    await execAction({ type: 'CameraOrbit', theta_deg: 30, phi_deg: 50 });
    await execAction({ type: 'CameraZoom', level: 2.0 });
    await screenshot('04-closeup-tree');

    // === 4. ZOOM OUT — full garden view ===
    await execAction({ type: 'CameraOrbit', theta_deg: 45, phi_deg: 70 });
    await execAction({ type: 'CameraZoom', level: 0.4 });
    await screenshot('05-full-garden-zoomed-out');

    // === 5. X-RAY MODE — underground roots ===
    // Toggle x-ray via keyboard
    await page.keyboard.press('q');
    await page.waitForTimeout(200);
    await execAction({ type: 'CameraOrbit', theta_deg: 45, phi_deg: 55 });
    await execAction({ type: 'CameraZoom', level: 1.0 });
    await screenshot('06-xray-underground');

    // X-ray from different angle
    await execAction({ type: 'CameraOrbit', theta_deg: 120, phi_deg: 45 });
    await screenshot('07-xray-side-angle');

    // X-ray close-up on roots
    await execAction({ type: 'CameraOrbit', theta_deg: 60, phi_deg: 40 });
    await execAction({ type: 'CameraZoom', level: 1.8 });
    await screenshot('08-xray-closeup-roots');

    // === 6. TURN OFF X-RAY ===
    await page.keyboard.press('q');
    await page.waitForTimeout(200);

    // === 7. SIDE VIEW — cross-section feel ===
    await execAction({ type: 'CameraOrbit', theta_deg: 90, phi_deg: 30 });
    await execAction({ type: 'CameraZoom', level: 1.0 });
    await screenshot('09-low-angle-side');

    // === 8. TOP-DOWN — planning view ===
    await execAction({ type: 'CameraOrbit', theta_deg: 0, phi_deg: 85 });
    await execAction({ type: 'CameraZoom', level: 0.5 });
    await screenshot('10-topdown-planning');

    // === 9. TIME OF DAY — cycle through lighting ===
    // Dawn
    await page.keyboard.press('[');
    await page.keyboard.press('[');
    await page.keyboard.press('[');
    await page.keyboard.press('[');
    await page.keyboard.press('[');
    await page.waitForTimeout(300);
    await execAction({ type: 'CameraOrbit', theta_deg: 45, phi_deg: 55 });
    await execAction({ type: 'CameraZoom', level: 0.8 });
    await screenshot('11-dawn-lighting');

    // Golden hour (step forward)
    for (let i = 0; i < 10; i++) await page.keyboard.press(']');
    await page.waitForTimeout(300);
    await screenshot('12-golden-hour');

    // Blue hour
    for (let i = 0; i < 5; i++) await page.keyboard.press(']');
    await page.waitForTimeout(300);
    await screenshot('13-blue-hour');

    // Noon
    for (let i = 0; i < 10; i++) await page.keyboard.press(']');
    await page.waitForTimeout(300);
    await screenshot('14-noon');

    // === 10. HUD FOCUS — show tool palette and species picker ===
    // Reset to golden hour, good angle
    for (let i = 0; i < 5; i++) await page.keyboard.press('[');
    await page.waitForTimeout(200);
    await execAction({ type: 'CameraOrbit', theta_deg: 45, phi_deg: 65 });
    await execAction({ type: 'CameraZoom', level: 0.7 });
    // Select seed tool to show species panel
    await page.keyboard.press('2');
    await page.waitForTimeout(500);
    await screenshot('15-hud-species-picker');

    // === 11. FOREST RING + SKIRT — decorative surroundings ===
    await execAction({ type: 'CameraOrbit', theta_deg: 45, phi_deg: 45 });
    await execAction({ type: 'CameraZoom', level: 0.35 });
    await screenshot('16-forest-ring-surroundings');

    // === 12. MATURE GARDEN — tick forward to grow competition ===
    // Fast-forward 500 ticks for trees to mature and roots to overlap
    await execAction({ type: 'Tick', n: 500 });

    // Above-ground: mature garden overview
    await execAction({ type: 'CameraOrbit', theta_deg: 45, phi_deg: 60 });
    await execAction({ type: 'CameraZoom', level: 0.7 });
    await screenshot('17-mature-garden');

    // X-ray: root competition should show red borders where species overlap
    await page.keyboard.press('q');
    await page.waitForTimeout(200);
    await execAction({ type: 'CameraOrbit', theta_deg: 45, phi_deg: 55 });
    await execAction({ type: 'CameraZoom', level: 1.0 });
    await screenshot('18-mature-xray-roots');

    // Close-up on root competition zone
    await execAction({ type: 'CameraOrbit', theta_deg: 60, phi_deg: 40 });
    await execAction({ type: 'CameraZoom', level: 1.8 });
    await screenshot('19-mature-xray-closeup');

    // === 13. IRRIGATION LENS — moisture heatmap shows water distribution ===
    // Cycle to irrigation lens (Shift+Q cycles lenses: off→roots→irrigation)
    // Currently on roots lens, press Shift+Q to go to irrigation
    await page.keyboard.press('Shift+q');
    await page.waitForTimeout(300);
    await execAction({ type: 'CameraOrbit', theta_deg: 45, phi_deg: 55 });
    await execAction({ type: 'CameraZoom', level: 0.8 });
    await screenshot('20-irrigation-heatmap');

    // Irrigation close-up on the dug channel area
    await execAction({ type: 'CameraOrbit', theta_deg: 30, phi_deg: 45 });
    await execAction({ type: 'CameraZoom', level: 1.5 });
    await screenshot('21-irrigation-closeup');

    // Turn off x-ray
    await page.keyboard.press('q');
    await page.waitForTimeout(200);

    console.log(`\nDone: 21 screenshots saved to ${SCREENSHOT_DIR}`);
  });
});
