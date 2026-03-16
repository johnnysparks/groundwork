import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import {
  setGridDimensions,
  VOXEL_BYTES,
  Material,
  GRID_X,
  GRID_Y,
  GRID_Z,
} from '../bridge';
import {
  meshChunk,
  createMockGrid,
  CHUNK_SIZE,
  isFoliage,
  isSeed,
  type MeshQuad,
} from './greedy';

// Use a small grid for fast, focused tests
const SMALL = 16;
let savedX: number, savedY: number, savedZ: number;

beforeAll(() => {
  savedX = GRID_X;
  savedY = GRID_Y;
  savedZ = GRID_Z;
  setGridDimensions(SMALL, SMALL, SMALL, 8);
});

afterAll(() => {
  setGridDimensions(savedX, savedY, savedZ, 40);
});

/** Helper: create a small empty grid */
function makeGrid(): Uint8Array {
  return new Uint8Array(SMALL * SMALL * SMALL * VOXEL_BYTES);
}

/** Helper: set a voxel's material in the small grid */
function setVoxel(grid: Uint8Array, x: number, y: number, z: number, mat: number, water = 0): void {
  const idx = (x + y * SMALL + z * SMALL * SMALL) * VOXEL_BYTES;
  grid[idx] = mat;
  grid[idx + 1] = water;
}

describe('meshChunk — empty chunk', () => {
  it('produces no quads for an all-air chunk', () => {
    const grid = makeGrid();
    const quads = meshChunk(grid, 0, 0, 0);
    expect(quads).toHaveLength(0);
  });
});

describe('meshChunk — single voxel', () => {
  it('produces 6 faces for a lone solid voxel', () => {
    const grid = makeGrid();
    setVoxel(grid, 5, 5, 5, Material.Stone);
    const quads = meshChunk(grid, 0, 0, 0);
    expect(quads).toHaveLength(6);
    for (const q of quads) {
      expect(q.material).toBe(Material.Stone);
      expect(q.w).toBe(1);
      expect(q.h).toBe(1);
    }
  });

  it('all 6 face directions are present', () => {
    const grid = makeGrid();
    setVoxel(grid, 5, 5, 5, Material.Soil);
    const quads = meshChunk(grid, 0, 0, 0);
    const faces = new Set(quads.map(q => q.face));
    expect(faces.size).toBe(6);
    expect(faces).toEqual(new Set([0, 1, 2, 3, 4, 5]));
  });
});

describe('meshChunk — greedy merging', () => {
  it('merges a 4x1 row of same material into one quad', () => {
    const grid = makeGrid();
    for (let x = 2; x < 6; x++) {
      setVoxel(grid, x, 5, 5, Material.Soil);
    }
    const quads = meshChunk(grid, 0, 0, 0);

    // The +Z and -Z faces should merge into a single 4-wide quad
    const topFaces = quads.filter(q => q.face === 4); // +Z
    expect(topFaces.length).toBe(1);
    expect(topFaces[0].w).toBe(4);
    expect(topFaces[0].h).toBe(1);
  });

  it('merges a 3x3 slab into one quad per face direction', () => {
    const grid = makeGrid();
    for (let x = 2; x < 5; x++) {
      for (let y = 2; y < 5; y++) {
        setVoxel(grid, x, y, 5, Material.Stone);
      }
    }
    const quads = meshChunk(grid, 0, 0, 0);

    // Top face (+Z) should be a single 3x3 quad
    const topFaces = quads.filter(q => q.face === 4);
    expect(topFaces.length).toBe(1);
    expect(topFaces[0].w).toBe(3);
    expect(topFaces[0].h).toBe(3);
  });

  it('does not merge different materials', () => {
    const grid = makeGrid();
    setVoxel(grid, 3, 5, 5, Material.Stone);
    setVoxel(grid, 4, 5, 5, Material.Soil);

    const quads = meshChunk(grid, 0, 0, 0);
    // Each produces its own top face — 2 quads for face 4
    const topFaces = quads.filter(q => q.face === 4);
    expect(topFaces.length).toBe(2);
    expect(topFaces.every(q => q.w === 1 && q.h === 1)).toBe(true);
  });
});

describe('meshChunk — face culling', () => {
  it('culls shared face between two adjacent solid voxels', () => {
    const grid = makeGrid();
    setVoxel(grid, 5, 5, 5, Material.Stone);
    setVoxel(grid, 6, 5, 5, Material.Stone);

    const quads = meshChunk(grid, 0, 0, 0);

    // Shared +X/-X face is culled (10 individual faces).
    // Greedy merge combines 4 coplanar pairs into 2-wide quads → 6 quads.
    expect(quads.length).toBe(6);

    // The merged faces should be 2 wide
    const wideFaces = quads.filter(q => q.w === 2 || q.h === 2);
    expect(wideFaces.length).toBe(4); // +Z, -Z, +Y, -Y
  });

  it('does not cull faces adjacent to air', () => {
    const grid = makeGrid();
    setVoxel(grid, 5, 5, 5, Material.Soil);
    // All neighbors are air → 6 faces
    const quads = meshChunk(grid, 0, 0, 0);
    expect(quads.length).toBe(6);
  });

  it('leaf voxels do not generate solid mesh faces', () => {
    const grid = makeGrid();
    setVoxel(grid, 5, 5, 5, Material.Leaf);
    const quads = meshChunk(grid, 0, 0, 0);
    // Leaf is not solid — no solid mesh quads
    expect(quads).toHaveLength(0);
  });

  it('root voxels generate faces against soil (x-ray visibility)', () => {
    const grid = makeGrid();
    // Root surrounded by soil on all sides
    setVoxel(grid, 5, 5, 5, Material.Root);
    setVoxel(grid, 4, 5, 5, Material.Soil);
    setVoxel(grid, 6, 5, 5, Material.Soil);
    setVoxel(grid, 5, 4, 5, Material.Soil);
    setVoxel(grid, 5, 6, 5, Material.Soil);
    setVoxel(grid, 5, 5, 4, Material.Soil);
    setVoxel(grid, 5, 5, 6, Material.Soil);

    const quads = meshChunk(grid, 0, 0, 0);
    const rootFaces = quads.filter(q => q.material === Material.Root);
    // Root should have 6 faces visible against soil
    expect(rootFaces.length).toBe(6);
  });
});

describe('meshChunk — wetness prevents merging', () => {
  it('soil with different water levels does not merge', () => {
    const grid = makeGrid();
    setVoxel(grid, 3, 5, 5, Material.Soil, 0);   // dry
    setVoxel(grid, 4, 5, 5, Material.Soil, 100); // wet

    const quads = meshChunk(grid, 0, 0, 0);
    const topFaces = quads.filter(q => q.face === 4);
    expect(topFaces.length).toBe(2);
  });

  it('soil with same wetness bucket merges', () => {
    const grid = makeGrid();
    // Both in "wet" bucket (>=80)
    setVoxel(grid, 3, 5, 5, Material.Soil, 90);
    setVoxel(grid, 4, 5, 5, Material.Soil, 100);

    const quads = meshChunk(grid, 0, 0, 0);
    const topFaces = quads.filter(q => q.face === 4);
    expect(topFaces.length).toBe(1);
    expect(topFaces[0].w).toBe(2);
  });
});

describe('meshChunk — AO values', () => {
  it('isolated voxel has all-zero AO', () => {
    const grid = makeGrid();
    setVoxel(grid, 8, 8, 8, Material.Stone);
    const quads = meshChunk(grid, 0, 0, 0);
    for (const q of quads) {
      expect(q.ao).toEqual([0, 0, 0, 0]);
    }
  });

  it('AO values increase when neighbors are present', () => {
    const grid = makeGrid();
    // Place a stone voxel with a neighbor above and to the side
    setVoxel(grid, 5, 5, 5, Material.Stone);
    setVoxel(grid, 6, 5, 5, Material.Stone); // +x neighbor
    setVoxel(grid, 5, 5, 6, Material.Stone); // +z neighbor

    const quads = meshChunk(grid, 0, 0, 0);
    // At least one face should have non-zero AO
    const hasAO = quads.some(q => q.ao.some(v => v > 0));
    expect(hasAO).toBe(true);
  });
});

describe('material classification', () => {
  it('isFoliage returns true only for Leaf', () => {
    expect(isFoliage(Material.Leaf)).toBe(true);
    expect(isFoliage(Material.Trunk)).toBe(false);
    expect(isFoliage(Material.Air)).toBe(false);
  });

  it('isSeed returns true only for Seed', () => {
    expect(isSeed(Material.Seed)).toBe(true);
    expect(isSeed(Material.Soil)).toBe(false);
  });
});

describe('createMockGrid', () => {
  it('creates a grid of expected size', () => {
    // Restore full dimensions for this test
    setGridDimensions(savedX, savedY, savedZ, 40);
    const grid = createMockGrid();
    expect(grid.length).toBe(savedX * savedY * savedZ * VOXEL_BYTES);
    // Restore small grid
    setGridDimensions(SMALL, SMALL, SMALL, 8);
  });

  it('has stone below z=20', () => {
    setGridDimensions(savedX, savedY, savedZ, 40);
    const grid = createMockGrid();
    const mat = grid[(10 + 10 * savedX + 5 * savedX * savedY) * VOXEL_BYTES];
    expect(mat).toBe(Material.Stone);
    setGridDimensions(SMALL, SMALL, SMALL, 8);
  });

  it('has soil at ground level', () => {
    setGridDimensions(savedX, savedY, savedZ, 40);
    const grid = createMockGrid();
    // Check a soil voxel away from structures
    const mat = grid[(70 + 70 * savedX + 35 * savedX * savedY) * VOXEL_BYTES];
    expect(mat).toBe(Material.Soil);
    setGridDimensions(SMALL, SMALL, SMALL, 8);
  });
});
