/**
 * WASM bridge: typed array views into simulation memory.
 *
 * The sim exposes raw pointers into its linear memory. We wrap them as
 * typed arrays for zero-copy access from the renderer.
 */

// These will be populated after WASM init
let wasmModule: any = null;
let wasmMemory: WebAssembly.Memory | null = null;

/** Grid dimensions (constants from Rust) */
export const GRID_X = 120;
export const GRID_Y = 120;
export const GRID_Z = 60;
export const GROUND_LEVEL = 30;

/** Voxel byte layout: [material, water_level, light_level, nutrient_level] */
export const VOXEL_BYTES = 4;

/** Soil byte layout: [sand, clay, organic, rock, ph, bacteria] */
export const SOIL_BYTES = 6;

/** Material enum matching Rust's Material repr(u8) */
export const Material = {
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
} as const;

export type MaterialType = (typeof Material)[keyof typeof Material];

/** Tool codes matching the WASM bridge */
export const ToolCode = {
  Shovel: 0,
  Seed: 1,
  Water: 2,
  Soil: 3,
  Stone: 4,
} as const;

export type ToolCodeType = (typeof ToolCode)[keyof typeof ToolCode];

/**
 * Initialize the WASM simulation module.
 * Must be called before any other bridge function.
 */
export async function initSim(): Promise<void> {
  // Dynamic import — the WASM module is built by wasm-pack
  const wasm = await import('../wasm/groundwork_sim.js');
  await wasm.default(); // Initialize WASM
  wasm.init();
  wasmModule = wasm;
  wasmMemory = (wasm as any).__wbg_get_memory?.() ?? wasm.memory;
}

/** Advance simulation by n ticks */
export function tick(n: number = 1): void {
  wasmModule.tick(n);
}

/** Get current tick count */
export function getTick(): bigint {
  return wasmModule.get_tick();
}

/**
 * Get a live typed array view of the voxel grid.
 * IMPORTANT: This view becomes invalid after any WASM allocation (tick, place_tool, etc).
 * Re-fetch after each sim step.
 */
export function getGridView(): Uint8Array {
  const ptr = wasmModule.grid_ptr();
  const len = wasmModule.grid_len();
  return new Uint8Array(wasmMemory!.buffer, ptr, len);
}

/** Get a live view of the soil composition grid */
export function getSoilView(): Uint8Array {
  const ptr = wasmModule.soil_ptr();
  const len = wasmModule.soil_len();
  return new Uint8Array(wasmMemory!.buffer, ptr, len);
}

/** Read a single voxel's material at (x, y, z) */
export function getMaterial(grid: Uint8Array, x: number, y: number, z: number): number {
  const idx = (x + y * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
  return grid[idx];
}

/** Read a voxel's water level */
export function getWaterLevel(grid: Uint8Array, x: number, y: number, z: number): number {
  const idx = (x + y * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
  return grid[idx + 1];
}

/** Place a tool at coordinates. Returns landing z or -1 on no effect. */
export function placeTool(tool: ToolCodeType, x: number, y: number, z: number): number {
  return wasmModule.place_tool(tool, x, y, z);
}

/** Fill a rectangular region with a tool */
export function fillTool(
  tool: ToolCodeType,
  x1: number, y1: number, z1: number,
  x2: number, y2: number, z2: number,
): void {
  wasmModule.fill_tool(tool, x1, y1, z1, x2, y2, z2);
}

/** Get focus cursor position */
export function getFocus(): { x: number; y: number; z: number } {
  return {
    x: wasmModule.get_focus_x(),
    y: wasmModule.get_focus_y(),
    z: wasmModule.get_focus_z(),
  };
}

/** Set focus cursor position */
export function setFocus(x: number, y: number, z: number): void {
  wasmModule.set_focus(x, y, z);
}

/**
 * Compute flat index for (x, y, z) in the grid.
 * Matches Rust's VoxelGrid::index.
 */
export function gridIndex(x: number, y: number, z: number): number {
  return x + y * GRID_X + z * GRID_X * GRID_Y;
}

/** Check if WASM sim is initialized */
export function isInitialized(): boolean {
  return wasmModule !== null;
}
