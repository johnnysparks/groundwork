/* tslint:disable */
/* eslint-disable */

/**
 * Cancel all gnome tasks.
 */
export function cancel_all_gnome_tasks(): void;

/**
 * Cancel gnome tasks at a specific position.
 */
export function cancel_gnome_task(x: number, y: number, z: number): void;

/**
 * Bitfield of discovered species. Bit N set = species N discovered.
 * JS can check `(discovered_species() >> speciesIdx) & 1` for each species.
 */
export function discovered_species(): number;

/**
 * Number of discovered species.
 */
export function discovered_species_count(): number;

/**
 * Number of active fauna creatures.
 */
export function fauna_count(): number;

/**
 * Length of fauna data in bytes.
 */
export function fauna_len(): number;

/**
 * Pointer to packed fauna data in WASM linear memory.
 * Each fauna is 16 bytes: [type: u8, state: u8, _pad: u8, _pad: u8, x: f32, y: f32, z: f32].
 */
export function fauna_ptr(): number;

/**
 * Fill a rectangular region with a tool.
 * Skips seeds and roots to protect living plants (matching CLI behavior).
 */
export function fill_tool(tool: number, x1: number, y1: number, z1: number, x2: number, y2: number, z2: number): void;

/**
 * Current day phase (0-99). Dawn=0-24, Day=25-49, Dusk=50-74, Night=75-99.
 * JS can use this to sync the visual day cycle with the sim's growth rhythm.
 */
export function get_day_phase(): number;

/**
 * Get focus position.
 */
export function get_focus_x(): number;

export function get_focus_y(): number;

export function get_focus_z(): number;

/**
 * Get the currently selected species index.
 */
export function get_selected_species(): number;

/**
 * Current tick count.
 */
export function get_tick(): bigint;

/**
 * Ticks remaining in current weather state.
 */
export function get_weather_duration(): number;

/**
 * Current weather state: 0=Clear, 1=Rain, 2=Drought.
 * JS can use this for visual effects (rain particles, dry palette shift).
 */
export function get_weather_state(): number;

/**
 * Length of ghost zone data in bytes (8 bytes per ghost).
 */
export function ghost_len(): number;

/**
 * Pointer to packed ghost zone data.
 */
export function ghost_ptr(): number;

/**
 * Length of gnome export data in bytes (always 32).
 */
export function gnome_len(): number;

/**
 * Pointer to packed gnome state (32 bytes).
 */
export function gnome_ptr(): number;

/**
 * Number of pending gnome tasks.
 */
export function gnome_queue_len(): number;

export function grid_depth(): number;

export function grid_height(): number;

/**
 * Length of the voxel grid in bytes.
 */
export function grid_len(): number;

/**
 * Pointer to the raw voxel grid data in WASM linear memory.
 * JS can wrap this as `new Uint8Array(wasm.memory.buffer, ptr, len)`.
 * Each voxel is 4 bytes: [material, water_level, light_level, nutrient_level].
 */
export function grid_ptr(): number;

/**
 * Grid dimensions as a packed u32: x | (y << 16). Z is returned separately.
 */
export function grid_width(): number;

/**
 * Ground level Z coordinate (surface height baseline).
 */
export function ground_level(): number;

/**
 * Initialize the simulation. Must be called before any other function.
 */
export function init(): void;

/**
 * Whether a specific species has been discovered.
 */
export function is_species_discovered(species_id: number): boolean;

/**
 * Total number of material types.
 */
export function material_count(): number;

/**
 * Whether a material is foliage (rendered as billboard sprites).
 */
export function material_is_foliage(idx: number): boolean;

/**
 * Whether a material is a seed (rendered as small sprites).
 */
export function material_is_seed(idx: number): boolean;

/**
 * Whether a material generates solid mesh geometry.
 */
export function material_is_solid(idx: number): boolean;

/**
 * Name of material at the given index, or empty string if invalid.
 */
export function material_name(idx: number): string;

/**
 * Total active fauna count (progress toward tier 3).
 */
export function milestone_fauna_count(): number;

/**
 * Groundcover leaf voxel count (progress toward tier 1).
 */
export function milestone_groundcover_count(): number;

/**
 * Active pollinator count (progress toward tier 2).
 */
export function milestone_pollinator_count(): number;

/**
 * Plant species diversity count (progress toward tier 3).
 */
export function milestone_species_diversity(): number;

/**
 * Whether flowers are unlocked (tier 1: groundcover established).
 */
export function milestone_tier1_flowers(): boolean;

/**
 * Whether shrubs are unlocked (tier 2: pollinators attracted).
 */
export function milestone_tier2_shrubs(): boolean;

/**
 * Whether trees are unlocked (tier 3: fauna ecosystem active).
 */
export function milestone_tier3_trees(): boolean;

/**
 * Pack tree stats into an export buffer for root war visualization.
 * Each tree: 12 bytes [species_id: u8, health_u8: u8, stage: u8, _pad: u8,
 *   root_x: u16le, root_y: u16le, root_count: u16le, water_intake: u16le]
 * Call this before reading tree_stats_ptr/len.
 */
export function pack_tree_stats(): number;

/**
 * Pick a random discovered species of a given plant type.
 * plant_type: 0=Tree, 1=Shrub, 2=Flower, 3=Groundcover
 * Returns species_id, or 255 if no species of that type discovered.
 */
export function pick_discovered_species(plant_type: number, rng_hint: number): number;

/**
 * Place a tool at (x, y, z). Tool codes: 0=shovel, 1=seed, 2=water (debug only — irrigation via digging replaces watering can), 3=soil, 4=stone.
 * Returns the landing z coordinate, or -1 if the action had no effect.
 */
export function place_tool(tool: number, x: number, y: number, z: number): number;

/**
 * Queue a task for the garden gnome. Tool codes: 0=shovel, 1=seed, 2=water (debug only), 3=soil, 4=stone.
 * Species is used for seed tasks (pass 255 for non-seed tools).
 * Returns true if the task was queued, false if the queue is full.
 */
export function queue_gnome_task(tool: number, x: number, y: number, z: number, species: number): boolean;

/**
 * Set the day phase explicitly (for syncing JS day cycle to sim).
 */
export function set_day_phase(phase: number): void;

/**
 * Set focus position.
 */
export function set_focus(x: number, y: number, z: number): void;

/**
 * Set the species index used when placing seeds.
 */
export function set_selected_species(idx: number): void;

/**
 * Length of the soil grid in bytes.
 */
export function soil_len(): number;

/**
 * Pointer to the soil composition grid data.
 * Each cell is 6 bytes: [sand, clay, organic, rock, ph, bacteria].
 */
export function soil_ptr(): number;

/**
 * Number of species in the species table.
 */
export function species_count(): number;

/**
 * Name of species at the given index.
 */
export function species_name(idx: number): string;

/**
 * Plant type name of species at the given index (Tree, Shrub, Ground, Flower).
 */
export function species_plant_type(idx: number): string;

/**
 * Advance the simulation by `n` ticks.
 */
export function tick(n: number): void;

/**
 * Number of available tools.
 */
export function tool_count(): number;

/**
 * Name of tool at the given index.
 */
export function tool_name(idx: number): string;

/**
 * Length of packed tree stats buffer in bytes.
 */
export function tree_stats_len(): number;

/**
 * Pointer to packed tree stats buffer. Call pack_tree_stats() first.
 */
export function tree_stats_ptr(): number;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly cancel_gnome_task: (a: number, b: number, c: number) => void;
    readonly fauna_ptr: () => number;
    readonly fill_tool: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly get_day_phase: () => number;
    readonly get_weather_state: () => number;
    readonly ghost_ptr: () => number;
    readonly gnome_ptr: () => number;
    readonly grid_depth: () => number;
    readonly grid_height: () => number;
    readonly grid_ptr: () => number;
    readonly ground_level: () => number;
    readonly is_species_discovered: (a: number) => number;
    readonly material_count: () => number;
    readonly material_is_foliage: (a: number) => number;
    readonly material_is_seed: (a: number) => number;
    readonly material_is_solid: (a: number) => number;
    readonly material_name: (a: number) => [number, number];
    readonly milestone_fauna_count: () => number;
    readonly milestone_groundcover_count: () => number;
    readonly milestone_pollinator_count: () => number;
    readonly milestone_species_diversity: () => number;
    readonly milestone_tier1_flowers: () => number;
    readonly milestone_tier2_shrubs: () => number;
    readonly milestone_tier3_trees: () => number;
    readonly pick_discovered_species: (a: number, b: number) => number;
    readonly place_tool: (a: number, b: number, c: number, d: number) => number;
    readonly queue_gnome_task: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly set_day_phase: (a: number) => void;
    readonly set_focus: (a: number, b: number, c: number) => void;
    readonly soil_ptr: () => number;
    readonly species_name: (a: number) => [number, number];
    readonly species_plant_type: (a: number) => [number, number];
    readonly tool_count: () => number;
    readonly tool_name: (a: number) => [number, number];
    readonly tree_stats_ptr: () => number;
    readonly fauna_len: () => number;
    readonly ghost_len: () => number;
    readonly tick: (a: number) => void;
    readonly discovered_species: () => number;
    readonly get_focus_x: () => number;
    readonly init: () => void;
    readonly gnome_len: () => number;
    readonly cancel_all_gnome_tasks: () => void;
    readonly get_tick: () => bigint;
    readonly pack_tree_stats: () => number;
    readonly grid_len: () => number;
    readonly soil_len: () => number;
    readonly grid_width: () => number;
    readonly discovered_species_count: () => number;
    readonly get_focus_y: () => number;
    readonly get_focus_z: () => number;
    readonly get_weather_duration: () => number;
    readonly gnome_queue_len: () => number;
    readonly set_selected_species: (a: number) => void;
    readonly fauna_count: () => number;
    readonly species_count: () => number;
    readonly tree_stats_len: () => number;
    readonly get_selected_species: () => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
