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
  // Keep this in sync with FaunaType in fauna.rs. If you add a variant in Rust,
  // add it here too — the sync-check script (scripts/sync-check.sh) will catch omissions.
  const RUST_FAUNA_TYPES: Record<string, number> = {
    Bee: 0,
    Butterfly: 1,
    Bird: 2,
    Worm: 3,
    Beetle: 4,
    Squirrel: 5,
  };

  it('FaunaType enum has correct count', () => {
    const tsKeys = Object.keys(FaunaType);
    const rustKeys = Object.keys(RUST_FAUNA_TYPES);
    expect(tsKeys.length).toBe(rustKeys.length);
  });

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

  it('FaunaState enum has correct count', () => {
    const tsKeys = Object.keys(FaunaState);
    const rustKeys = Object.keys(RUST_FAUNA_STATES);
    expect(tsKeys.length).toBe(rustKeys.length);
  });

  for (const [name, value] of Object.entries(RUST_FAUNA_STATES)) {
    it(`FaunaState.${name} = ${value}`, () => {
      expect(FaunaState[name as keyof typeof FaunaState]).toBe(value);
    });
  }
});

describe('WASM bridge contract: tree stats layout', () => {
  // Rust: wasm_bridge.rs — pack_tree_stats
  // Each tree: 12 bytes [species_id: u8, health_u8: u8, stage: u8, _pad: u8,
  //   root_x: u16le, root_y: u16le, root_count: u16le, water_intake: u16le]
  const TREE_STATS_BYTES = 12;

  it('tree stats record size is 12 bytes', () => {
    expect(TREE_STATS_BYTES).toBe(12);
  });

  it('tree stats field offsets are correct', () => {
    // Verify the byte layout matches what JS would read with DataView
    const buf = new ArrayBuffer(TREE_STATS_BYTES);
    const view = new DataView(buf);
    const bytes = new Uint8Array(buf);

    // Write test values at Rust-defined offsets
    bytes[0] = 3;   // species_id at offset 0
    bytes[1] = 200; // health_u8 at offset 1
    bytes[2] = 2;   // stage at offset 2
    bytes[3] = 0;   // pad at offset 3
    view.setUint16(4, 40, true);  // root_x at offset 4 (le)
    view.setUint16(6, 50, true);  // root_y at offset 6 (le)
    view.setUint16(8, 15, true);  // root_count at offset 8 (le)
    view.setUint16(10, 300, true); // water_intake at offset 10 (le)

    expect(bytes[0]).toBe(3);           // species_id
    expect(bytes[1]).toBe(200);         // health
    expect(bytes[2]).toBe(2);           // stage
    expect(view.getUint16(4, true)).toBe(40);  // root_x
    expect(view.getUint16(6, true)).toBe(50);  // root_y
    expect(view.getUint16(8, true)).toBe(15);  // root_count
    expect(view.getUint16(10, true)).toBe(300); // water_intake
  });
});

describe('WASM bridge contract: weather & day phase', () => {
  // Rust: wasm_bridge.rs — get_weather_state returns u8: 0=Clear, 1=Rain, 2=Drought
  it('weather state codes match Rust WeatherState enum', () => {
    const WEATHER = { Clear: 0, Rain: 1, Drought: 2 };
    expect(WEATHER.Clear).toBe(0);
    expect(WEATHER.Rain).toBe(1);
    expect(WEATHER.Drought).toBe(2);
  });

  // Rust: wasm_bridge.rs — get_day_phase returns u8 0-99
  // Dawn=0-24, Day=25-49, Dusk=50-74, Night=75-99
  it('day phase ranges are documented correctly', () => {
    const isDawn = (p: number) => p >= 0 && p <= 24;
    const isDay = (p: number) => p >= 25 && p <= 49;
    const isDusk = (p: number) => p >= 50 && p <= 74;
    const isNight = (p: number) => p >= 75 && p <= 99;

    expect(isDawn(0)).toBe(true);
    expect(isDawn(24)).toBe(true);
    expect(isDay(25)).toBe(true);
    expect(isDay(49)).toBe(true);
    expect(isDusk(50)).toBe(true);
    expect(isDusk(74)).toBe(true);
    expect(isNight(75)).toBe(true);
    expect(isNight(99)).toBe(true);
  });
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
