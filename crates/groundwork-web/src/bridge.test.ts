import { describe, it, expect, beforeEach } from 'vitest';
import {
  VOXEL_BYTES,
  SOIL_BYTES,
  FAUNA_BYTES,
  Material,
  ToolCode,
  FaunaType,
  FaunaState,
  gridIndex,
  getMaterial,
  getWaterLevel,
  materialIsSolid,
  materialIsFoliage,
  materialIsSeed,
  readFauna,
  setGridDimensions,
  GRID_X,
  GRID_Y,
  GRID_Z,
} from './bridge';

describe('bridge constants', () => {
  it('VOXEL_BYTES is 4', () => {
    expect(VOXEL_BYTES).toBe(4);
  });

  it('SOIL_BYTES is 6', () => {
    expect(SOIL_BYTES).toBe(6);
  });

  it('FAUNA_BYTES is 16', () => {
    expect(FAUNA_BYTES).toBe(16);
  });

  it('Material enum has expected values', () => {
    expect(Material.Air).toBe(0);
    expect(Material.Soil).toBe(1);
    expect(Material.Stone).toBe(2);
    expect(Material.Water).toBe(3);
    expect(Material.Root).toBe(4);
    expect(Material.Seed).toBe(5);
    expect(Material.Trunk).toBe(6);
    expect(Material.Branch).toBe(7);
    expect(Material.Leaf).toBe(8);
    expect(Material.DeadWood).toBe(9);
  });

  it('ToolCode enum has expected values', () => {
    expect(ToolCode.Shovel).toBe(0);
    expect(ToolCode.Seed).toBe(1);
    expect(ToolCode.Water).toBe(2);
    expect(ToolCode.Soil).toBe(3);
    expect(ToolCode.Stone).toBe(4);
  });

  it('FaunaType enum has expected values', () => {
    expect(FaunaType.Bee).toBe(0);
    expect(FaunaType.Butterfly).toBe(1);
    expect(FaunaType.Bird).toBe(2);
    expect(FaunaType.Worm).toBe(3);
    expect(FaunaType.Beetle).toBe(4);
  });

  it('FaunaState enum has expected values', () => {
    expect(FaunaState.Idle).toBe(0);
    expect(FaunaState.Seeking).toBe(1);
    expect(FaunaState.Acting).toBe(2);
    expect(FaunaState.Leaving).toBe(3);
  });
});

describe('gridIndex', () => {
  it('computes flat index for origin', () => {
    expect(gridIndex(0, 0, 0)).toBe(0);
  });

  it('increments by 1 along x', () => {
    expect(gridIndex(1, 0, 0)).toBe(1);
    expect(gridIndex(5, 0, 0)).toBe(5);
  });

  it('increments by GRID_X along y', () => {
    expect(gridIndex(0, 1, 0)).toBe(GRID_X);
    expect(gridIndex(0, 3, 0)).toBe(GRID_X * 3);
  });

  it('increments by GRID_X*GRID_Y along z', () => {
    expect(gridIndex(0, 0, 1)).toBe(GRID_X * GRID_Y);
  });

  it('computes combined index correctly', () => {
    const x = 10, y = 20, z = 30;
    expect(gridIndex(x, y, z)).toBe(x + y * GRID_X + z * GRID_X * GRID_Y);
  });
});

describe('getMaterial / getWaterLevel', () => {
  let grid: Uint8Array;

  beforeEach(() => {
    grid = new Uint8Array(GRID_X * GRID_Y * GRID_Z * VOXEL_BYTES);
  });

  it('reads material byte (offset 0) from voxel', () => {
    const x = 5, y = 10, z = 15;
    const idx = (x + y * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
    grid[idx] = Material.Stone;
    expect(getMaterial(grid, x, y, z)).toBe(Material.Stone);
  });

  it('reads water level byte (offset 1) from voxel', () => {
    const x = 3, y = 7, z = 20;
    const idx = (x + y * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
    grid[idx + 1] = 200;
    expect(getWaterLevel(grid, x, y, z)).toBe(200);
  });

  it('returns Air (0) for uninitialized voxels', () => {
    expect(getMaterial(grid, 0, 0, 0)).toBe(Material.Air);
  });
});

describe('material classification (mock mode)', () => {
  it('Air is not solid', () => {
    expect(materialIsSolid(Material.Air)).toBe(false);
  });

  it('Water is not solid', () => {
    expect(materialIsSolid(Material.Water)).toBe(false);
  });

  it('Leaf is not solid (rendered as foliage)', () => {
    expect(materialIsSolid(Material.Leaf)).toBe(false);
  });

  it('Soil is solid', () => {
    expect(materialIsSolid(Material.Soil)).toBe(true);
  });

  it('Stone is solid', () => {
    expect(materialIsSolid(Material.Stone)).toBe(true);
  });

  it('Trunk is solid', () => {
    expect(materialIsSolid(Material.Trunk)).toBe(true);
  });

  it('Root is solid', () => {
    expect(materialIsSolid(Material.Root)).toBe(true);
  });

  it('Leaf is foliage', () => {
    expect(materialIsFoliage(Material.Leaf)).toBe(true);
  });

  it('Soil is not foliage', () => {
    expect(materialIsFoliage(Material.Soil)).toBe(false);
  });

  it('Seed is a seed', () => {
    expect(materialIsSeed(Material.Seed)).toBe(true);
  });

  it('Trunk is not a seed', () => {
    expect(materialIsSeed(Material.Trunk)).toBe(false);
  });
});

describe('setGridDimensions', () => {
  it('overrides grid dimensions', () => {
    const origX = GRID_X, origY = GRID_Y, origZ = GRID_Z;
    setGridDimensions(16, 16, 32, 10);
    expect(GRID_X).toBe(16);
    expect(GRID_Y).toBe(16);
    expect(GRID_Z).toBe(32);
    // Restore
    setGridDimensions(origX, origY, origZ, 40);
  });
});

describe('readFauna', () => {
  it('unpacks a fauna record from a DataView', () => {
    const buffer = new ArrayBuffer(FAUNA_BYTES * 2);
    const view = new DataView(buffer);

    // Write fauna 0: Bee, Seeking, at (1.5, 2.5, 3.5)
    view.setUint8(0, FaunaType.Bee);
    view.setUint8(1, FaunaState.Seeking);
    view.setFloat32(4, 1.5, true);
    view.setFloat32(8, 2.5, true);
    view.setFloat32(12, 3.5, true);

    const f0 = readFauna(view, 0);
    expect(f0.type).toBe(FaunaType.Bee);
    expect(f0.state).toBe(FaunaState.Seeking);
    expect(f0.x).toBeCloseTo(1.5);
    expect(f0.y).toBeCloseTo(2.5);
    expect(f0.z).toBeCloseTo(3.5);
  });

  it('reads second fauna record at correct offset', () => {
    const buffer = new ArrayBuffer(FAUNA_BYTES * 2);
    const view = new DataView(buffer);

    // Write fauna 1 at byte offset 16
    view.setUint8(16, FaunaType.Bird);
    view.setUint8(17, FaunaState.Acting);
    view.setFloat32(20, 10.0, true);
    view.setFloat32(24, 20.0, true);
    view.setFloat32(28, 30.0, true);

    const f1 = readFauna(view, 1);
    expect(f1.type).toBe(FaunaType.Bird);
    expect(f1.state).toBe(FaunaState.Acting);
    expect(f1.x).toBeCloseTo(10.0);
    expect(f1.y).toBeCloseTo(20.0);
    expect(f1.z).toBeCloseTo(30.0);
  });
});
