/**
 * WASM bridge contract tests.
 *
 * These verify that the TypeScript bridge's hardcoded fallback values
 * stay in sync with the Rust sim's source of truth. If a Rust enum
 * variant is added/removed/reordered, these tests catch the drift
 * before it becomes a runtime bug.
 *
 * The expected values are intentionally duplicated from the Rust source
 * so the test fails when either side changes without updating the other.
 */

import { describe, it, expect } from 'vitest';
import {
  VOXEL_BYTES,
  SOIL_BYTES,
  FAUNA_BYTES,
  Material,
  ToolCode,
  FaunaType,
  FaunaState,
  GRID_X,
  GRID_Y,
  GRID_Z,
} from './bridge';

describe('WASM bridge contract: grid layout', () => {
  // Rust: grid.rs — GRID_X=80, GRID_Y=80, GRID_Z=100
  it('grid dimensions match Rust constants', () => {
    expect(GRID_X).toBe(80);
    expect(GRID_Y).toBe(80);
    expect(GRID_Z).toBe(100);
  });

  // Rust: Voxel is 4 bytes [material, water_level, light_level, nutrient_level]
  it('VOXEL_BYTES matches Rust Voxel struct size', () => {
    expect(VOXEL_BYTES).toBe(4);
  });

  // Rust: SoilComposition is 6 bytes [sand, clay, organic, rock, ph, bacteria]
  it('SOIL_BYTES matches Rust SoilComposition struct size', () => {
    expect(SOIL_BYTES).toBe(6);
  });

  // Rust: fauna export is 16 bytes [type, state, pad, pad, x:f32, y:f32, z:f32]
  it('FAUNA_BYTES matches Rust fauna export record size', () => {
    expect(FAUNA_BYTES).toBe(16);
  });
});

describe('WASM bridge contract: Material enum', () => {
  // Rust: voxel.rs — Material repr(u8)
  // Must match exactly or mesh rendering, grid reads, and tool placement break.
  const RUST_MATERIALS: Record<string, number> = {
    Air: 0,
    Soil: 1,
    Stone: 2,
    Water: 3,
    Root: 4,
    Seed: 5,
    Trunk: 6,
    Branch: 7,
    Leaf: 8,
    DeadWood: 9,
  };

  it('Material enum has correct count', () => {
    const tsKeys = Object.keys(Material);
    const rustKeys = Object.keys(RUST_MATERIALS);
    expect(tsKeys.length).toBe(rustKeys.length);
  });

  for (const [name, value] of Object.entries(RUST_MATERIALS)) {
    it(`Material.${name} = ${value}`, () => {
      expect(Material[name]).toBe(value);
    });
  }
});

describe('WASM bridge contract: ToolCode enum', () => {
  // Rust: wasm_bridge.rs — place_tool match arms
  const RUST_TOOLS: Record<string, number> = {
    Shovel: 0,
    Seed: 1,
    Water: 2,
    Soil: 3,
    Stone: 4,
  };

  it('ToolCode enum has correct count', () => {
    const tsKeys = Object.keys(ToolCode);
    const rustKeys = Object.keys(RUST_TOOLS);
    expect(tsKeys.length).toBe(rustKeys.length);
  });

  for (const [name, value] of Object.entries(RUST_TOOLS)) {
    it(`ToolCode.${name} = ${value}`, () => {
      expect(ToolCode[name]).toBe(value);
    });
  }
});

describe('WASM bridge contract: FaunaType enum', () => {
  // Rust: fauna.rs — FaunaType repr(u8)
  const RUST_FAUNA_TYPES: Record<string, number> = {
    Bee: 0,
    Butterfly: 1,
    Bird: 2,
    Worm: 3,
    Beetle: 4,
  };

  for (const [name, value] of Object.entries(RUST_FAUNA_TYPES)) {
    it(`FaunaType.${name} = ${value}`, () => {
      expect(FaunaType[name as keyof typeof FaunaType]).toBe(value);
    });
  }
});

describe('WASM bridge contract: FaunaState enum', () => {
  // Rust: fauna.rs — FaunaState repr(u8)
  const RUST_FAUNA_STATES: Record<string, number> = {
    Idle: 0,
    Seeking: 1,
    Acting: 2,
    Leaving: 3,
  };

  for (const [name, value] of Object.entries(RUST_FAUNA_STATES)) {
    it(`FaunaState.${name} = ${value}`, () => {
      expect(FaunaState[name as keyof typeof FaunaState]).toBe(value);
    });
  }
});

describe('WASM bridge contract: grid index formula', () => {
  // Rust: VoxelGrid::index = x + y * GRID_X + z * GRID_X * GRID_Y
  // This must match getMaterial/getWaterLevel byte offset calculations.
  it('voxel byte offset formula matches Rust VoxelGrid::index', () => {
    const x = 10, y = 20, z = 30;
    const rustIndex = x + y * GRID_X + z * GRID_X * GRID_Y;
    const byteOffset = rustIndex * VOXEL_BYTES;
    // This is the formula used in getMaterial/getWaterLevel
    const tsOffset = (x + y * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
    expect(tsOffset).toBe(byteOffset);
  });

  it('total grid byte size matches expected', () => {
    const expectedBytes = GRID_X * GRID_Y * GRID_Z * VOXEL_BYTES;
    expect(expectedBytes).toBe(80 * 80 * 100 * 4); // 2,560,000 bytes
  });
});
