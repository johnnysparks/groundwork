import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import {
  setGridDimensions,
  VOXEL_BYTES,
  Material,
  GRID_X,
  GRID_Y,
  GRID_Z,
} from '../bridge';
import { ChunkManager } from './chunk';
import { CHUNK_SIZE } from './greedy';

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

function makeGrid(): Uint8Array {
  return new Uint8Array(SMALL * SMALL * SMALL * VOXEL_BYTES);
}

function setVoxel(grid: Uint8Array, x: number, y: number, z: number, mat: number): void {
  const idx = (x + y * SMALL + z * SMALL * SMALL) * VOXEL_BYTES;
  grid[idx] = mat;
}

describe('ChunkManager construction', () => {
  it('creates correct number of chunks for 16x16x16 grid', () => {
    const mgr = new ChunkManager();
    const chunks = mgr.allChunks();
    // 16/16 = 1 chunk per axis
    expect(chunks.length).toBe(1);
  });

  it('all chunks start dirty', () => {
    const mgr = new ChunkManager();
    expect(mgr.dirtyCount()).toBe(mgr.allChunks().length);
  });
});

describe('ChunkManager.detectChanges', () => {
  it('marks all dirty on first call (no previous snapshot)', () => {
    const mgr = new ChunkManager();
    const grid = makeGrid();
    // Clear dirty state first by rebuilding
    mgr.rebuildDirty(grid);
    expect(mgr.dirtyCount()).toBe(0);

    // detectChanges with no previous snapshot → marks all dirty
    const mgr2 = new ChunkManager();
    mgr2.rebuildDirty(makeGrid());
    expect(mgr2.dirtyCount()).toBe(0);

    mgr2.detectChanges(grid);
    // First detectChanges marks all dirty
    expect(mgr2.dirtyCount()).toBe(1);
  });

  it('detects a single voxel change', () => {
    const mgr = new ChunkManager();
    const grid = makeGrid();

    // Initial snapshot
    mgr.detectChanges(grid);
    mgr.rebuildDirty(grid);
    expect(mgr.dirtyCount()).toBe(0);

    // Modify one voxel
    setVoxel(grid, 5, 5, 5, Material.Stone);
    mgr.detectChanges(grid);
    expect(mgr.dirtyCount()).toBeGreaterThan(0);
  });

  it('no change means no dirty chunks', () => {
    const mgr = new ChunkManager();
    const grid = makeGrid();
    setVoxel(grid, 3, 3, 3, Material.Soil);

    mgr.detectChanges(grid);
    mgr.rebuildDirty(grid);
    expect(mgr.dirtyCount()).toBe(0);

    // Pass same grid again — no changes
    mgr.detectChanges(grid);
    expect(mgr.dirtyCount()).toBe(0);
  });
});

describe('ChunkManager.rebuildDirty', () => {
  it('returns updated chunks and clears dirty flags', () => {
    const mgr = new ChunkManager();
    const grid = makeGrid();
    setVoxel(grid, 5, 5, 5, Material.Stone);

    const updated = mgr.rebuildDirty(grid);
    expect(updated.length).toBeGreaterThan(0);
    expect(mgr.dirtyCount()).toBe(0);
  });

  it('meshed quads are populated on updated chunks', () => {
    const mgr = new ChunkManager();
    const grid = makeGrid();
    setVoxel(grid, 5, 5, 5, Material.Stone);

    const updated = mgr.rebuildDirty(grid);
    // Stone voxel should generate 6 quads
    const totalQuads = updated.reduce((sum, c) => sum + c.quads.length, 0);
    expect(totalQuads).toBe(6);
  });

  it('second rebuild with no changes returns empty', () => {
    const mgr = new ChunkManager();
    const grid = makeGrid();
    setVoxel(grid, 5, 5, 5, Material.Stone);

    mgr.rebuildDirty(grid);
    // No changes detected, already clean
    const updated = mgr.rebuildDirty(grid);
    expect(updated.length).toBe(0);
  });
});

describe('ChunkManager — multi-chunk grid', () => {
  it('handles 32x32x32 grid with 8 chunks', () => {
    setGridDimensions(32, 32, 32, 16);
    const mgr = new ChunkManager();
    // 32/16 = 2 chunks per axis → 8 total
    expect(mgr.allChunks().length).toBe(8);
    setGridDimensions(SMALL, SMALL, SMALL, 8);
  });
});

describe('ChunkManager.markAllDirty', () => {
  it('sets every chunk to dirty', () => {
    const mgr = new ChunkManager();
    const grid = makeGrid();
    mgr.rebuildDirty(grid);
    expect(mgr.dirtyCount()).toBe(0);

    mgr.markAllDirty();
    expect(mgr.dirtyCount()).toBe(mgr.allChunks().length);
  });
});
