import { test, expect } from '@playwright/test';

test('debug: seed moisture + health verification', async ({ page }) => {
  await page.goto('http://localhost:5173', { waitUntil: 'networkidle' });
  await page.waitForSelector('#loading.hidden', { timeout: 30000 });

  const hasAPI = await page.evaluate(() => !!(window as any).agentAPI);
  expect(hasAPI).toBeTruthy();

  // Check soil moisture near the pond (y~16, pond center)
  const moisture = await page.evaluate(() => {
    const api = (window as any).agentAPI;
    const info = api.getGridInfo();
    // getGridView reads raw bytes
    // Scan soil at ground level around the pond
    // pond at (40, 16)
    const results: Array<{x: number, y: number, z: number, mat: number, water: number}> = [];
    // We can't call getGridView directly from here — it's in bridge.ts
    // But getMaterialCounts scans the whole grid. Let's check specific positions.
    return info;
  });
  console.log('Grid info:', JSON.stringify(moisture));

  // Place seeds NEAR the pond where soil is wet (y=22, close to pond y=16)
  await page.evaluate(async () => {
    const api = (window as any).agentAPI;
    // Place seeds very close to the pond
    await api.executeAction({ type: 'Fill', tool: 'seed', x1: 38, y1: 22, z1: 50, x2: 42, y2: 26, z2: 50 });
  });

  const afterPlace = await page.evaluate(() => {
    const api = (window as any).agentAPI;
    return api.getMaterialCounts();
  });
  console.log('After place near pond:', JSON.stringify(afterPlace));

  // Tick to grow — 200 ticks should be enough
  await page.evaluate(async () => {
    const api = (window as any).agentAPI;
    await api.executeAction({ type: 'Tick', n: 200 });
  });

  const afterGrow = await page.evaluate(() => {
    const api = (window as any).agentAPI;
    return {
      tick: api.getTick(),
      materials: api.getMaterialCounts(),
      trees: api.getTreeEntityHealth(),
      health: api.getHealthHistogram(),
    };
  });
  console.log('After 200 ticks:', JSON.stringify(afterGrow));

  // If still no growth, try sampling soil water near the seed positions
  const soilWater = await page.evaluate(() => {
    const api = (window as any).agentAPI;
    // Sample raw grid bytes at seed positions to check soil moisture
    const grid = (window as any).agentAPI.sampleLeafBytes ? null : null;
    // Can't easily sample soil from agentAPI. Let's check getMaterialCounts.
    return api.getMaterialCounts();
  });

  if (afterGrow.materials.leaf === 0) {
    console.log('NO GROWTH - seeds did not germinate. Soil may be too dry near seed positions.');
    // Try an even wetter location — right at the pond edge
    await page.evaluate(async () => {
      const api = (window as any).agentAPI;
      await api.executeAction({ type: 'Fill', tool: 'seed', x1: 38, y1: 18, z1: 50, x2: 42, y2: 20, z2: 50 });
      await api.executeAction({ type: 'Tick', n: 200 });
    });

    const afterWet = await page.evaluate(() => {
      const api = (window as any).agentAPI;
      return {
        tick: api.getTick(),
        materials: api.getMaterialCounts(),
        trees: api.getTreeEntityHealth(),
        health: api.getHealthHistogram(),
      };
    });
    console.log('After wet position + 200 ticks:', JSON.stringify(afterWet));

    // Sample leaves if any
    if (afterWet.materials.leaf > 0) {
      const samples = await page.evaluate(() => {
        const api = (window as any).agentAPI;
        return api.sampleLeafBytes();
      });
      console.log('Leaf samples:', JSON.stringify(samples.slice(0, 5)));
    }
  } else {
    // We have leaves! Check health
    const samples = await page.evaluate(() => {
      const api = (window as any).agentAPI;
      return api.sampleLeafBytes();
    });
    console.log('Leaf samples:', JSON.stringify(samples.slice(0, 5)));
  }
});
