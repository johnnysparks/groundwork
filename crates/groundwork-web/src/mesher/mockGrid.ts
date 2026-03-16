/**
 * OAK GROWTH SHOWCASE — demo grid displaying oak at each growth stage.
 *
 * DESIGN ETHOS:
 *
 * A seedling is a promise. A sapling is determination.
 * A young tree is ambition. A mature tree is presence.
 * An old-growth tree is a world unto itself.
 *
 * An ancient oak fills 1/6th of the map. Its crown creates shade
 * that shapes what can grow beneath it. Many generations of
 * flowers and pollinators have lived and died under its canopy.
 *
 * Trees are not decorations. They ARE the landscape.
 *
 * LAYOUT:
 * Five oak specimens left→right (X axis), seedling to old-growth.
 * Single row centered in Y. Grid sized to fit with crown clearance.
 */

import { GRID_X, GRID_Y, GRID_Z, GROUND_LEVEL, VOXEL_BYTES, Material, setGridDimensions } from '../bridge';

// ═══════════════════════════════════════════════════════════════════════
// Voxel placement helpers
// ═══════════════════════════════════════════════════════════════════════

function setVoxel(grid: Uint8Array, x: number, y: number, z: number, mat: number): void {
  if (x < 0 || x >= GRID_X || y < 0 || y >= GRID_Y || z < 0 || z >= GRID_Z) return;
  const idx = (x + y * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
  grid[idx] = mat;
}

function getVoxel(grid: Uint8Array, x: number, y: number, z: number): number {
  if (x < 0 || x >= GRID_X || y < 0 || y >= GRID_Y || z < 0 || z >= GRID_Z) return Material.Air;
  return grid[(x + y * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES];
}

/** Place a cylindrical trunk column */
function placeTrunk(
  grid: Uint8Array, cx: number, cy: number,
  zBase: number, height: number, radius: number,
): void {
  for (let h = 0; h < height; h++) {
    const z = zBase + h;
    if (radius <= 0) {
      setVoxel(grid, cx, cy, z, Material.Trunk);
    } else {
      for (let dx = -radius; dx <= radius; dx++) {
        for (let dy = -radius; dy <= radius; dy++) {
          if (dx * dx + dy * dy <= radius * radius) {
            setVoxel(grid, cx + dx, cy + dy, z, Material.Trunk);
          }
        }
      }
    }
  }
}

/**
 * Place a tapering trunk (for old-growth trees with buttressed bases).
 * Radius interpolates from baseR at bottom to topR at top.
 */
function placeTaperingTrunk(
  grid: Uint8Array, cx: number, cy: number,
  zBase: number, height: number, baseR: number, topR: number,
): void {
  for (let h = 0; h < height; h++) {
    const z = zBase + h;
    const frac = h / Math.max(1, height - 1);
    const r = Math.round(baseR + (topR - baseR) * frac);
    if (r <= 0) {
      setVoxel(grid, cx, cy, z, Material.Trunk);
    } else {
      for (let dx = -r; dx <= r; dx++) {
        for (let dy = -r; dy <= r; dy++) {
          if (dx * dx + dy * dy <= r * r) {
            setVoxel(grid, cx + dx, cy + dy, z, Material.Trunk);
          }
        }
      }
    }
  }
}

/** Place a disc of leaf voxels (only overwrites air) */
function placeLeafDisc(grid: Uint8Array, cx: number, cy: number, z: number, radius: number): void {
  if (z < 0 || z >= GRID_Z) return;
  for (let dx = -radius; dx <= radius; dx++) {
    for (let dy = -radius; dy <= radius; dy++) {
      if (dx * dx + dy * dy <= radius * radius) {
        const gx = cx + dx, gy = cy + dy;
        if (gx >= 0 && gx < GRID_X && gy >= 0 && gy < GRID_Y) {
          if (getVoxel(grid, gx, gy, z) === Material.Air) {
            setVoxel(grid, gx, gy, z, Material.Leaf);
          }
        }
      }
    }
  }
}

/** Place a multi-layer crown of leaf voxels */
function placeCrown(
  grid: Uint8Array, cx: number, cy: number,
  zBase: number, maxR: number, height: number,
  shape: 'round' | 'narrow' | 'wide' | 'conical',
): void {
  for (let dz = 0; dz < height; dz++) {
    const z = zBase + dz;
    const frac = dz / Math.max(1, height - 1);
    let r: number;
    switch (shape) {
      case 'round':
        // Egg-shaped: peak radius at ~40% height, tapers both ends
        r = Math.max(1, Math.round(maxR * Math.sin((frac * 0.8 + 0.15) * Math.PI)));
        break;
      case 'narrow':
        r = Math.max(1, Math.round(maxR * 0.55));
        break;
      case 'wide':
        r = frac < 0.6
          ? maxR
          : Math.max(1, Math.round(maxR * (1 - (frac - 0.6) / 0.4 * 0.7)));
        break;
      case 'conical':
        r = Math.max(1, Math.round(maxR * (1 - frac * 0.9)));
        break;
    }
    placeLeafDisc(grid, cx, cy, z, r);
  }
}

/** Place branch arms extending outward from trunk at a given height */
function placeBranches(
  grid: Uint8Array, cx: number, cy: number,
  z: number, reach: number, dirSeed: number,
): void {
  const dirs: [number, number][] = [[1, 0], [0, 1], [-1, 0], [0, -1]];
  const d1 = dirs[dirSeed % 4];
  const d2 = dirs[(dirSeed + 2) % 4]; // opposite direction
  for (let r = 1; r <= reach; r++) {
    setVoxel(grid, cx + d1[0] * r, cy + d1[1] * r, z, Material.Branch);
    setVoxel(grid, cx + d2[0] * r, cy + d2[1] * r, z, Material.Branch);
  }
}

/** Simple deterministic hash for per-direction variation */
function dirHash(seed: number, i: number): number {
  return ((seed * 73856093 + i * 19349663) >>> 0) & 0xffff;
}

/**
 * Place thick major limbs radiating outward in 8 directions.
 * Each limb has a different reach based on a deterministic seed,
 * creating an asymmetric branching structure.
 */
function placeLimbs(
  grid: Uint8Array, cx: number, cy: number,
  zStart: number, reach: number, thickness: number, seed: number,
): void {
  const dirs: [number, number][] = [
    [1, 0], [0, 1], [-1, 0], [0, -1],
    [1, 1], [-1, 1], [1, -1], [-1, -1],
  ];
  for (let i = 0; i < dirs.length; i++) {
    const [dx, dy] = dirs[i];
    // Vary reach 50–130% per direction; skip some entirely
    const h = dirHash(seed, i);
    const reachMul = 0.5 + (h & 0xff) / 255.0 * 0.8;
    const thisReach = Math.max(2, Math.floor(reach * reachMul));
    // 15% chance to skip a limb (creates gaps in the canopy)
    if ((h >> 8) < 0x26) continue;

    for (let r = 1; r <= thisReach; r++) {
      const gx = cx + dx * r;
      const gy = cy + dy * r;
      const z = zStart + Math.floor(r / 3);
      setVoxel(grid, gx, gy, z, Material.Branch);
      if (thickness > 1 && r <= thisReach * 0.7) {
        setVoxel(grid, gx, gy, z + 1, Material.Branch);
      }
      if (thickness > 2 && r <= thisReach * 0.4) {
        setVoxel(grid, gx, gy, z - 1, Material.Branch);
      }
    }
  }
}

/**
 * Place a multi-lobe spreading crown with per-direction variation.
 * Each lobe has randomized reach, radius, and height based on a seed,
 * creating the irregular, spreading silhouette of a real oak.
 */
function placeOakCrown(
  grid: Uint8Array, cx: number, cy: number,
  zBase: number, mainR: number, height: number,
  limbReach: number, seed: number,
): void {
  // Central crown — slightly off-center for asymmetry
  const cOff = (dirHash(seed, 99) & 3) - 1; // -1, 0, or 1
  const centralR = Math.max(3, Math.floor(mainR * 0.5));
  placeCrown(grid, cx + cOff, cy, zBase, centralR, height, 'wide');

  // 8 lobes with per-direction variation
  const dirs: [number, number][] = [
    [1, 0], [0, 1], [-1, 0], [0, -1],
    [1, 1], [-1, 1], [1, -1], [-1, -1],
  ];

  for (let i = 0; i < dirs.length; i++) {
    const [dx, dy] = dirs[i];
    const h = dirHash(seed, i + 50);
    const isDiag = i >= 4;

    // 20% chance to skip a lobe (creates interesting gaps)
    if ((h >> 12) < 0x3) continue;

    // Vary reach 60–140% per direction
    const reachMul = 0.6 + (h & 0xff) / 255.0 * 0.8;
    const baseReach = isDiag ? limbReach * 0.65 : limbReach;
    const lobeReach = Math.floor(baseReach * reachMul);

    // Vary radius 60–130%
    const rMul = 0.6 + ((h >> 4) & 0xff) / 255.0 * 0.7;
    const baseR = isDiag ? mainR * 0.3 : mainR * 0.45;
    const lobeR = Math.max(2, Math.floor(baseR * rMul));

    // Vary height
    const hMul = 0.6 + ((h >> 8) & 0xf) / 15.0 * 0.6;
    const baseH = isDiag ? height * 0.5 : height * 0.65;
    const lobeH = Math.max(2, Math.floor(baseH * hMul));

    const lx = cx + dx * lobeReach;
    const ly = cy + dy * lobeReach;
    const lz = zBase + Math.floor(lobeReach / 3) + (isDiag ? 1 : 0);

    placeCrown(grid, lx, ly, lz, lobeR, lobeH, 'wide');
  }
}

/** Place a single root tendril — a line of root voxels through soil */
function placeRootLine(
  grid: Uint8Array, x0: number, y0: number, z0: number,
  x1: number, y1: number, z1: number, thickness: number,
): void {
  // Bresenham-ish 3D line with thickness
  const dx = x1 - x0, dy = y1 - y0, dz = z1 - z0;
  const steps = Math.max(Math.abs(dx), Math.abs(dy), Math.abs(dz), 1);
  for (let s = 0; s <= steps; s++) {
    const frac = s / steps;
    const x = Math.round(x0 + dx * frac);
    const y = Math.round(y0 + dy * frac);
    const z = Math.round(z0 + dz * frac);
    // Thickness as a disc perpendicular to travel direction
    const r = thickness;
    for (let tdx = -r; tdx <= r; tdx++) {
      for (let tdy = -r; tdy <= r; tdy++) {
        if (tdx * tdx + tdy * tdy <= r * r) {
          const gx = x + tdx, gy = y + tdy;
          if (gx >= 0 && gx < GRID_X && gy >= 0 && gy < GRID_Y && z >= 0 && z < GRID_Z) {
            const mat = getVoxel(grid, gx, gy, z);
            if (mat === Material.Soil || mat === Material.Stone) {
              setVoxel(grid, gx, gy, z, Material.Root);
            }
          }
        }
      }
    }
  }
}

/**
 * Place a branching root network — the underground mirror of the crown.
 *
 * Structure:
 * - Taproot: thick central root descending straight down, tapering
 * - Major laterals: 8 thick roots radiating outward, descending as they spread
 * - Secondary branches: fork off laterals at intervals, thinner and shorter
 * - Fine roots: single-voxel tendrils at the tips
 *
 * Like an inverted crown — each stage adds more complexity and spread.
 */
function placeOakRoots(
  grid: Uint8Array, cx: number, cy: number,
  zSurface: number, depth: number, spread: number,
  tapRadius: number, seed: number,
): void {
  // --- Taproot: straight down from center, tapering ---
  for (let dz = 1; dz <= depth; dz++) {
    const z = zSurface - dz;
    if (z < 0) break;
    const frac = dz / depth;
    const r = Math.max(0, Math.round(tapRadius * (1 - frac * 0.8)));
    if (r === 0) {
      setVoxel(grid, cx, cy, z, Material.Root);
    } else {
      for (let dx = -r; dx <= r; dx++) {
        for (let dy = -r; dy <= r; dy++) {
          if (dx * dx + dy * dy <= r * r) {
            const gx = cx + dx, gy = cy + dy;
            if (gx >= 0 && gx < GRID_X && gy >= 0 && gy < GRID_Y) {
              if (getVoxel(grid, gx, gy, z) === Material.Soil) {
                setVoxel(grid, gx, gy, z, Material.Root);
              }
            }
          }
        }
      }
    }
  }

  // --- Major lateral roots: radiate outward in 8 directions ---
  const dirs: [number, number][] = [
    [1, 0], [0, 1], [-1, 0], [0, -1],
    [1, 1], [-1, 1], [1, -1], [-1, -1],
  ];

  for (let i = 0; i < dirs.length; i++) {
    const [dx, dy] = dirs[i];
    const h = dirHash(seed, i + 100);
    const isDiag = i >= 4;

    // 15% chance to skip a root (creates asymmetry)
    if ((h >> 12) < 0x2) continue;

    // Per-direction reach variation 50–140%
    const reachMul = 0.5 + (h & 0xff) / 255.0 * 0.9;
    const baseSpread = isDiag ? spread * 0.6 : spread;
    const thisSpread = Math.max(3, Math.floor(baseSpread * reachMul));

    // Descent angle varies per root (how deep it goes relative to spread)
    const descentRate = 0.3 + ((h >> 4) & 0xf) / 15.0 * 0.5; // 0.3–0.8

    // Thickness: thick near trunk, thins to 0 at tip
    const baseThick = Math.min(tapRadius, 2);

    // Draw the main lateral root as a descending line
    const endX = cx + dx * thisSpread;
    const endY = cy + dy * thisSpread;
    const endZ = Math.max(1, Math.round(zSurface - 2 - thisSpread * descentRate));

    // Draw in segments so we can taper thickness
    const segments = 4;
    for (let seg = 0; seg < segments; seg++) {
      const f0 = seg / segments;
      const f1 = (seg + 1) / segments;
      const sx = Math.round(cx + dx * thisSpread * f0);
      const sy = Math.round(cy + dy * thisSpread * f0);
      const sz = Math.round(zSurface - 2 - thisSpread * descentRate * f0);
      const ex = Math.round(cx + dx * thisSpread * f1);
      const ey = Math.round(cy + dy * thisSpread * f1);
      const ez = Math.round(zSurface - 2 - thisSpread * descentRate * f1);
      const thick = Math.max(0, Math.round(baseThick * (1 - f0 * 0.8)));
      placeRootLine(grid, sx, sy, sz, ex, ey, ez, thick);
    }

    // --- Secondary branches: fork off the lateral at intervals ---
    const numForks = Math.max(1, Math.floor(thisSpread / 6));
    for (let f = 0; f < numForks; f++) {
      const fh = dirHash(seed, i * 17 + f + 200);
      const forkFrac = 0.3 + f / numForks * 0.5; // fork at 30-80% along lateral
      const forkX = Math.round(cx + dx * thisSpread * forkFrac);
      const forkY = Math.round(cy + dy * thisSpread * forkFrac);
      const forkZ = Math.round(zSurface - 2 - thisSpread * descentRate * forkFrac);

      // Fork direction: perpendicular to parent + some downward
      // Rotate 90 degrees: (dx,dy) → (-dy,dx) or (dy,-dx)
      const forkDir = (fh & 1) ? [-dy, dx] : [dy, -dx];
      const forkLen = Math.max(2, Math.floor(thisSpread * 0.4 * (0.5 + (fh & 0xf) / 15.0 * 0.5)));
      const forkDescent = 0.4 + ((fh >> 4) & 0xf) / 15.0 * 0.4;

      const fEndX = forkX + forkDir[0] * forkLen;
      const fEndY = forkY + forkDir[1] * forkLen;
      const fEndZ = Math.max(1, Math.round(forkZ - forkLen * forkDescent));

      placeRootLine(grid, forkX, forkY, forkZ, fEndX, fEndY, fEndZ, 0);

      // Fine tip tendril at end of fork — descends a bit more
      const tipH = dirHash(seed, i * 31 + f + 300);
      const tipLen = Math.max(1, Math.floor(forkLen * 0.3));
      const tipDx = ((tipH & 3) - 1); // slight wander
      const tipDy = (((tipH >> 2) & 3) - 1);
      placeRootLine(grid, fEndX, fEndY, fEndZ,
        fEndX + tipDx * tipLen, fEndY + tipDy * tipLen,
        Math.max(1, fEndZ - tipLen), 0);
    }

    // Fine tip at end of main lateral
    const tipH = dirHash(seed, i + 400);
    const tipLen = Math.max(2, Math.floor(thisSpread * 0.15));
    placeRootLine(grid, endX, endY, endZ,
      endX + dx * tipLen, endY + dy * tipLen,
      Math.max(1, endZ - tipLen), 0);
  }

  // --- Surface buttresses: flared roots visible at ground level ---
  // Only for larger trees (spread > 8)
  if (spread > 8) {
    for (let i = 0; i < dirs.length; i++) {
      const [dx, dy] = dirs[i];
      const h = dirHash(seed, i + 500);
      if ((h >> 12) < 0x3) continue; // skip some buttresses
      const extend = Math.max(1, Math.floor(tapRadius * 1.5 * (0.5 + (h & 0xf) / 15.0 * 0.5)));
      for (let r = 0; r <= extend; r++) {
        const gx = cx + dx * (tapRadius + r);
        const gy = cy + dy * (tapRadius + r);
        setVoxel(grid, gx, gy, zSurface + 1, Material.Root);
        setVoxel(grid, gx, gy, zSurface, Material.Root);
        if (r < extend * 0.6) {
          setVoxel(grid, gx, gy, zSurface - 1, Material.Root);
        }
      }
    }
  }
}

// ═══════════════════════════════════════════════════════════════════════
// Layout — oak only, 5 growth stages in a single row
// ═══════════════════════════════════════════════════════════════════════

// Grid sized for one row of oaks: old-growth crown R28 needs ~70 Y,
// stage spacing based on crown sizes at each stage.
const DEMO_X = 220;
const DEMO_Y = 70;
const DEMO_Z = 110;
const DEMO_GROUND = 40;

// Stage columns — spacing grows with crown radius.
const S0 = 10;   // Seedling (1 voxel)
const S1 = 30;   // Sapling (crown R5)
const S2 = 60;   // Young (crown R12)
const S3 = 110;  // Mature (crown R20)
const S4 = 180;  // Old Growth (crown R28)

// All oaks centered in Y
const OAK_Y = 35;

const Z = DEMO_GROUND + 1; // surface level (first air voxel)

// ═══════════════════════════════════════════════════════════════════════
// Grid generation
// ═══════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════
// Scene registry — each scene is a named grid factory
// ═══════════════════════════════════════════════════════════════════════

export interface SceneDef {
  id: string;
  name: string;
  description: string;
  /** null = use WASM simulation */
  createGrid: (() => Uint8Array) | null;
}

export const SCENES: SceneDef[] = [
  { id: 'sim', name: 'Simulation', description: 'Live WASM simulation (requires wasm build)', createGrid: null },
  { id: 'oak', name: 'Oak Stages', description: 'Five oak growth stages — seedling to old-growth', createGrid: () => createPlantDemoGrid() },
  { id: 'garden', name: 'Dense Garden', description: 'A mixed garden with all species', createGrid: () => createDenseGardenGrid() },
];

/** Get the scene ID from the URL, or a sensible default */
export function getSceneId(wasmAvailable: boolean): string {
  const params = new URLSearchParams(window.location.search);
  const id = params.get('scene');
  if (id && SCENES.some(s => s.id === id)) return id;
  return wasmAvailable ? 'sim' : 'oak';
}

/** Navigate to a different scene (reloads the page) */
export function switchScene(id: string): void {
  const params = new URLSearchParams(window.location.search);
  params.set('scene', id);
  window.location.search = params.toString();
}

export function createPlantDemoGrid(): Uint8Array {
  setGridDimensions(DEMO_X, DEMO_Y, DEMO_Z, DEMO_GROUND);

  const size = GRID_X * GRID_Y * GRID_Z * VOXEL_BYTES;
  const grid = new Uint8Array(size);

  // --- Terrain: flat soil surface, stone base ---
  for (let z = 0; z < GRID_Z; z++) {
    for (let y = 0; y < GRID_Y; y++) {
      for (let x = 0; x < GRID_X; x++) {
        const idx = (x + y * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
        if (z < 15) {
          grid[idx] = Material.Stone;
        } else if (z <= GROUND_LEVEL) {
          grid[idx] = Material.Soil;
        }
      }
    }
  }

  // ═══════════════════════════════════════════════════════════════════
  // OAK — the king. Round crown, thick trunk, deep roots.
  //
  // The oak is the anchor of the temperate garden. Patient and massive.
  // An ancient oak planted in early game should fill up 1/6th of the map,
  // dominating the scene and the landscape. Many generations of flowers
  // and pollinators will have lived and died in its shade.
  //
  // The reference: a gnarled Japanese oak, trunk wider than a person,
  // crown spreading 20+ meters, roots gripping the earth like fingers.
  // In our glen, the old-growth oak IS the glen.
  // ═══════════════════════════════════════════════════════════════════

  // Seedling: a promise — single stem, single taproot
  setVoxel(grid, S0, OAK_Y, Z, Material.Trunk);
  setVoxel(grid, S0, OAK_Y, Z + 1, Material.Leaf);
  setVoxel(grid, S0, OAK_Y, GROUND_LEVEL, Material.Root);
  setVoxel(grid, S0, OAK_Y, GROUND_LEVEL - 1, Material.Root);
  setVoxel(grid, S0, OAK_Y, GROUND_LEVEL - 2, Material.Root);

  // Sapling: determination — short trunk, first lateral roots emerging
  placeTrunk(grid, S1, OAK_Y, Z, 10, 1);
  placeBranches(grid, S1, OAK_Y, Z + 7, 3, 0);
  placeBranches(grid, S1, OAK_Y, Z + 8, 2, 1);
  placeCrown(grid, S1, OAK_Y, Z + 6, 5, 5, 'wide');
  // depth=8, spread=6, tapR=1 — small root ball with a few laterals
  placeOakRoots(grid, S1, OAK_Y, GROUND_LEVEL, 8, 6, 1, S1);

  // Young tree: ambition — root system spreading to match crown reach
  placeTrunk(grid, S2, OAK_Y, Z, 20, 2);
  placeLimbs(grid, S2, OAK_Y, Z + 12, 8, 1, S2);
  placeLimbs(grid, S2, OAK_Y, Z + 16, 6, 1, S2 + 1);
  placeOakCrown(grid, S2, OAK_Y, Z + 12, 12, 8, 8, S2);
  // depth=16, spread=14, tapR=2 — developing lateral network
  placeOakRoots(grid, S2, OAK_Y, GROUND_LEVEL, 16, 14, 2, S2);

  // Mature: presence — massive root network, buttresses gripping the earth
  placeTaperingTrunk(grid, S3, OAK_Y, Z, 30, 4, 3);
  placeLimbs(grid, S3, OAK_Y, Z + 16, 14, 2, S3);
  placeLimbs(grid, S3, OAK_Y, Z + 22, 10, 2, S3 + 1);
  placeLimbs(grid, S3, OAK_Y, Z + 26, 8, 1, S3 + 2);
  placeOakCrown(grid, S3, OAK_Y, Z + 16, 20, 12, 14, S3);
  // depth=24, spread=22, tapR=3 — thick laterals with secondary branches
  placeOakRoots(grid, S3, OAK_Y, GROUND_LEVEL, 24, 22, 3, S3);

  // Old Growth: a world unto itself — underground as impressive as above.
  // Root network spreads as wide as the crown. Thick laterals fork into
  // secondary and tertiary branches, creating a cave-like network through
  // the soil. Surface buttresses grip the earth like gnarled fingers.
  placeTaperingTrunk(grid, S4, OAK_Y, Z, 35, 6, 3);
  placeLimbs(grid, S4, OAK_Y, Z + 16, 20, 3, S4);
  placeLimbs(grid, S4, OAK_Y, Z + 22, 16, 2, S4 + 1);
  placeLimbs(grid, S4, OAK_Y, Z + 28, 12, 2, S4 + 2);
  placeLimbs(grid, S4, OAK_Y, Z + 32, 8, 1, S4 + 3);
  placeOakCrown(grid, S4, OAK_Y, Z + 18, 28, 14, 18, S4);
  // depth=30, spread=28, tapR=5 — enormous spreading root cave network
  placeOakRoots(grid, S4, OAK_Y, GROUND_LEVEL, 30, 28, 5, S4);

  return grid;
}

// ═══════════════════════════════════════════════════════════════════════
// Dense Garden — a mixed garden with multiple species
// ═══════════════════════════════════════════════════════════════════════

function createDenseGardenGrid(): Uint8Array {
  // Use a moderate grid — fits nicely without massive dimensions
  setGridDimensions(120, 120, 60, 30);

  const GX = 120, GY = 120, GZ = 60, GL = 30;
  const size = GX * GY * GZ * VOXEL_BYTES;
  const grid = new Uint8Array(size);
  const SZ = GL + 1; // surface Z

  function set(x: number, y: number, z: number, mat: number): void {
    if (x < 0 || x >= GX || y < 0 || y >= GY || z < 0 || z >= GZ) return;
    grid[(x + y * GX + z * GX * GY) * VOXEL_BYTES] = mat;
  }

  function get(x: number, y: number, z: number): number {
    if (x < 0 || x >= GX || y < 0 || y >= GY || z < 0 || z >= GZ) return Material.Air;
    return grid[(x + y * GX + z * GX * GY) * VOXEL_BYTES];
  }

  // --- Terrain ---
  for (let z = 0; z < GZ; z++) {
    for (let y = 0; y < GY; y++) {
      for (let x = 0; x < GX; x++) {
        if (z < 10) set(x, y, z, Material.Stone);
        else if (z <= GL) set(x, y, z, Material.Soil);
      }
    }
  }

  // --- Water feature: small pond ---
  for (let dx = -5; dx <= 5; dx++) {
    for (let dy = -5; dy <= 5; dy++) {
      if (dx * dx + dy * dy <= 25) {
        set(60 + dx, 60 + dy, GL, Material.Water);
        set(60 + dx, 60 + dy, GL + 1, Material.Water);
      }
    }
  }

  // --- Mature oak (center-left) ---
  placeTaperingTrunk(grid, 35, 60, SZ, 25, 3, 2);
  placeLimbs(grid, 35, 60, SZ + 14, 10, 2, 35);
  placeLimbs(grid, 35, 60, SZ + 18, 8, 1, 36);
  placeOakCrown(grid, 35, 60, SZ + 12, 14, 10, 10, 35);
  placeOakRoots(grid, 35, 60, GL, 18, 16, 2, 35);

  // --- Young birch (right of pond) ---
  placeTrunk(grid, 85, 55, SZ, 18, 1);
  placeBranches(grid, 85, 55, SZ + 12, 4, 0);
  placeBranches(grid, 85, 55, SZ + 14, 3, 1);
  placeCrown(grid, 85, 55, SZ + 10, 6, 8, 'narrow');

  // --- Pine (back-right) ---
  placeTrunk(grid, 95, 85, SZ, 22, 1);
  placeCrown(grid, 95, 85, SZ + 6, 8, 16, 'conical');

  // --- Willow near pond ---
  placeTrunk(grid, 52, 68, SZ, 14, 1);
  placeBranches(grid, 52, 68, SZ + 10, 6, 2);
  placeCrown(grid, 52, 68, SZ + 8, 8, 6, 'wide');

  // --- Shrubs: fern cluster ---
  for (const [fx, fy] of [[25, 45], [28, 48], [22, 50], [30, 44]]) {
    placeTrunk(grid, fx, fy, SZ, 2, 0);
    placeCrown(grid, fx, fy, SZ + 1, 3, 3, 'round');
  }

  // --- Berry bush near pond ---
  for (const [bx, by] of [[70, 55], [72, 58], [68, 52]]) {
    placeTrunk(grid, bx, by, SZ, 3, 0);
    placeCrown(grid, bx, by, SZ + 2, 3, 3, 'round');
  }

  // --- Flowers scattered around ---
  for (const [fx, fy] of [[40, 40], [42, 38], [38, 42], [55, 45], [58, 42],
                           [75, 70], [78, 72], [80, 68], [45, 80], [48, 82]]) {
    set(fx, fy, SZ, Material.Trunk); // stem
    set(fx, fy, SZ + 1, Material.Leaf); // bloom (rendered as foliage billboard)
  }

  // --- Groundcover: moss patches ---
  for (const [mx, my] of [[30, 55], [32, 57], [34, 56], [33, 54],
                           [80, 40], [82, 42], [81, 38], [83, 41]]) {
    for (let dx = -2; dx <= 2; dx++) {
      for (let dy = -2; dy <= 2; dy++) {
        if (dx * dx + dy * dy <= 4 && get(mx + dx, my + dy, SZ) === Material.Air) {
          set(mx + dx, my + dy, SZ, Material.Leaf);
        }
      }
    }
  }

  // --- Clover patch near oak (nitrogen handshake demo) ---
  for (let dx = -4; dx <= 4; dx++) {
    for (let dy = -4; dy <= 4; dy++) {
      if (dx * dx + dy * dy <= 16) {
        const gx = 40 + dx, gy = 50 + dy;
        if (get(gx, gy, SZ) === Material.Air) {
          set(gx, gy, SZ, Material.Leaf);
        }
      }
    }
  }

  return grid;
}
