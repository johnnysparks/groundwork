/**
 * WASM bridge: typed array views into simulation memory.
 *
 * The sim exposes raw pointers into its linear memory. We wrap them as
 * typed arrays for zero-copy access from the renderer.
 *
 * Grid dimensions, material constants, tool codes, and species metadata
 * are all populated from the WASM engine at init time. TypeScript never
 * defines these values — the Rust engine is the single source of truth.
 */

// These will be populated after WASM init
let wasmModule: any = null;
let wasmMemory: WebAssembly.Memory | null = null;

/** Grid dimensions — populated from WASM engine at init, or overridden by demo grid */
export let GRID_X = 80;
export let GRID_Y = 80;
export let GRID_Z = 100;
export let GROUND_LEVEL = 40;

/** Override grid dimensions (used by demo/mock grids that need more space) */
export function setGridDimensions(x: number, y: number, z: number, groundLevel: number): void {
  GRID_X = x;
  GRID_Y = y;
  GRID_Z = z;
  GROUND_LEVEL = groundLevel;
}

/** Voxel byte layout: [material, water_level, light_level, nutrient_level] */
export const VOXEL_BYTES = 4;

/** Soil byte layout: [sand, clay, organic, rock, ph, bacteria] */
export const SOIL_BYTES = 6;

/**
 * Material enum — populated from WASM engine at init.
 * Fallback values match the Rust Material repr(u8) for mock mode.
 */
export const Material: Record<string, number> = {
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

export type MaterialType = number;

/**
 * Tool codes — populated from WASM engine at init.
 * Fallback values match the Rust wasm_bridge tool codes for mock mode.
 */
export const ToolCode: Record<string, number> = {
  Shovel: 0,
  Seed: 1,
  Water: 2,
  Soil: 3,
  Stone: 4,
};

export type ToolCodeType = number;

/** Material classification — delegates to engine when available */
const solidCache = new Map<number, boolean>();
const foliageCache = new Map<number, boolean>();
const seedCache = new Map<number, boolean>();

/** Whether a material generates solid mesh geometry (not air, water, or leaf). */
export function materialIsSolid(mat: number): boolean {
  const cached = solidCache.get(mat);
  if (cached !== undefined) return cached;
  let result: boolean;
  if (wasmModule?.material_is_solid) {
    result = wasmModule.material_is_solid(mat);
  } else {
    // Fallback for mock mode
    result = mat !== Material.Air && mat !== Material.Water && mat !== Material.Leaf;
  }
  solidCache.set(mat, result);
  return result;
}

/** Whether a material is foliage rendered as billboard sprites. */
export function materialIsFoliage(mat: number): boolean {
  const cached = foliageCache.get(mat);
  if (cached !== undefined) return cached;
  let result: boolean;
  if (wasmModule?.material_is_foliage) {
    result = wasmModule.material_is_foliage(mat);
  } else {
    result = mat === Material.Leaf;
  }
  foliageCache.set(mat, result);
  return result;
}

/** Whether a material is a seed (rendered as small sprites). */
export function materialIsSeed(mat: number): boolean {
  const cached = seedCache.get(mat);
  if (cached !== undefined) return cached;
  let result: boolean;
  if (wasmModule?.material_is_seed) {
    result = wasmModule.material_is_seed(mat);
  } else {
    result = mat === Material.Seed;
  }
  seedCache.set(mat, result);
  return result;
}

/** Species definition from the engine */
export interface SpeciesDef {
  index: number;
  name: string;
  type: string;
}

/** Species list — populated from WASM engine at init */
export let SPECIES: SpeciesDef[] = [];

/** Tool definition from the engine */
export interface ToolDef {
  code: number;
  name: string;
}

/** Tool list — populated from WASM engine at init */
export let TOOLS: ToolDef[] = [];

/**
 * Populate constants from the WASM engine.
 * Called once after WASM init succeeds.
 */
function populateFromEngine(): void {
  // Grid dimensions
  GRID_X = wasmModule.grid_width();
  GRID_Y = wasmModule.grid_height();
  GRID_Z = wasmModule.grid_depth();
  GROUND_LEVEL = wasmModule.ground_level();

  // Material enum — rebuild from engine
  const matCount = wasmModule.material_count();
  for (let i = 0; i < matCount; i++) {
    const name = wasmModule.material_name(i);
    if (name) {
      // Convert "deadwood" → "DeadWood", "air" → "Air", etc.
      const key = name.charAt(0).toUpperCase() + name.slice(1);
      Material[key] = i;
    }
  }

  // Pre-populate classification caches
  solidCache.clear();
  foliageCache.clear();
  seedCache.clear();
  for (let i = 0; i < matCount; i++) {
    solidCache.set(i, wasmModule.material_is_solid(i));
    foliageCache.set(i, wasmModule.material_is_foliage(i));
    seedCache.set(i, wasmModule.material_is_seed(i));
  }

  // Tool list
  const toolCount = wasmModule.tool_count();
  TOOLS = [];
  for (let i = 0; i < toolCount; i++) {
    const name = wasmModule.tool_name(i);
    if (name) {
      TOOLS.push({ code: i, name });
      ToolCode[name] = i;
    }
  }

  // Species list
  const speciesCount = wasmModule.species_count();
  SPECIES = [];
  for (let i = 0; i < speciesCount; i++) {
    const name = wasmModule.species_name(i);
    const plantType = wasmModule.species_plant_type(i);
    if (name) {
      SPECIES.push({ index: i, name, type: plantType });
    }
  }
}

/**
 * Populate fallback constants for mock mode (when WASM fails to load).
 */
function populateFallbacks(): void {
  TOOLS = [
    { code: 0, name: 'Shovel' },
    { code: 1, name: 'Seed' },
    { code: 2, name: 'Water' },
    { code: 3, name: 'Soil' },
    { code: 4, name: 'Stone' },
  ];

  SPECIES = [
    { index: 0, name: 'Oak', type: 'Tree' },
    { index: 1, name: 'Birch', type: 'Tree' },
    { index: 2, name: 'Willow', type: 'Tree' },
    { index: 3, name: 'Pine', type: 'Tree' },
    { index: 4, name: 'Fern', type: 'Shrub' },
    { index: 5, name: 'Berry Bush', type: 'Shrub' },
    { index: 6, name: 'Holly', type: 'Shrub' },
    { index: 7, name: 'Wildflower', type: 'Flower' },
    { index: 8, name: 'Daisy', type: 'Flower' },
    { index: 9, name: 'Moss', type: 'Ground' },
    { index: 10, name: 'Grass', type: 'Ground' },
    { index: 11, name: 'Clover', type: 'Ground' },
  ];
}

/**
 * Initialize the WASM simulation module.
 * Must be called before any other bridge function.
 */
export async function initSim(): Promise<boolean> {
  // Dynamic import — the WASM module is built by wasm-pack.
  // Fails gracefully if WASM hasn't been built yet (mock data mode).
  try {
    // Try multiple WASM paths: relative (dev), public dir (production)
    const candidates = [
      new URL(/* @vite-ignore */ '../wasm/groundwork_sim.js', import.meta.url).href,
      new URL('/wasm/groundwork_sim.js', window.location.origin + ((import.meta as any).env?.BASE_URL ?? '/')).href,
    ];
    let wasmHref: string | null = null;
    for (const url of candidates) {
      const resp = await fetch(url, { method: 'HEAD' }).catch(() => null);
      if (resp?.ok) { wasmHref = url; break; }
    }
    if (!wasmHref) {
      console.warn('WASM module not found — running in mock data mode');
      populateFallbacks();
      return false;
    }
    const wasm = await import(/* @vite-ignore */ wasmHref);
    // default() initializes the WASM module and returns InitOutput (instance.exports)
    const initOutput = await wasm.default();
    wasm.init(); // Create bevy_ecs World + Schedule
    wasmModule = wasm;
    // Memory lives on the InitOutput (instance.exports), not the ES module
    wasmMemory = initOutput.memory;
    populateFromEngine();
    return true;
  } catch (e) {
    console.warn('WASM init failed — running in mock data mode:', e);
    populateFallbacks();
    return false;
  }
}

/** Advance simulation by n ticks */
export function tick(n: number = 1): void {
  wasmModule.tick(n);
}

/** Reset the simulation to a fresh garden */
export function resetSim(): void {
  if (wasmModule?.init) {
    wasmModule.init();
  }
}

/** Get current tick count */
export function getTick(): bigint {
  return wasmModule.get_tick();
}

/** Save the voxel grid to a Uint8Array (copy from WASM memory) */
export function saveGrid(): Uint8Array | null {
  if (!wasmModule?.grid_ptr || !wasmMemory) return null;
  const ptr = wasmModule.grid_ptr();
  const len = wasmModule.grid_len();
  const view = new Uint8Array(wasmMemory.buffer, ptr, len);
  return new Uint8Array(view); // copy
}

/** Restore the voxel grid from a Uint8Array (write into WASM memory) */
export function restoreGrid(data: Uint8Array): boolean {
  if (!wasmModule?.grid_ptr || !wasmMemory) return false;
  const ptr = wasmModule.grid_ptr();
  const len = wasmModule.grid_len();
  if (data.length !== len) return false;
  const view = new Uint8Array(wasmMemory.buffer, ptr, len);
  view.set(data);
  return true;
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

/** Set the selected species index for seed placement. */
export function setSelectedSpecies(idx: number): void {
  wasmModule.set_selected_species(idx);
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

// --- Fauna data ---

/** Fauna export record: 16 bytes per fauna.
 *  [type: u8, state: u8, _pad: u8, _pad: u8, x: f32, y: f32, z: f32] */
export const FAUNA_BYTES = 16;

/** Fauna type enum (matches Rust FaunaType repr(u8)) */
export const FaunaType = {
  Bee: 0,
  Butterfly: 1,
  Bird: 2,
  Worm: 3,
  Beetle: 4,
} as const;

/** Fauna state enum (matches Rust FaunaState repr(u8)) */
export const FaunaState = {
  Idle: 0,
  Seeking: 1,
  Acting: 2,
  Leaving: 3,
} as const;

/** Get number of active fauna creatures */
export function getFaunaCount(): number {
  if (!wasmModule?.fauna_count) return 0;
  return wasmModule.fauna_count();
}

/** Get a live DataView of the fauna data buffer */
export function getFaunaView(): DataView | null {
  if (!wasmModule?.fauna_ptr || !wasmMemory) return null;
  const ptr = wasmModule.fauna_ptr();
  const len = wasmModule.fauna_len();
  if (len === 0) return null;
  return new DataView(wasmMemory.buffer, ptr, len);
}

/** Read a fauna record from the packed buffer */
export function readFauna(view: DataView, index: number): {
  type: number;
  state: number;
  x: number;
  y: number;
  z: number;
} {
  const off = index * FAUNA_BYTES;
  return {
    type: view.getUint8(off),
    state: view.getUint8(off + 1),
    x: view.getFloat32(off + 4, true),
    y: view.getFloat32(off + 8, true),
    z: view.getFloat32(off + 12, true),
  };
}
