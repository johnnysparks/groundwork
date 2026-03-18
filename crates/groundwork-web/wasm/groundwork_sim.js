/* @ts-self-types="./groundwork_sim.d.ts" */

/**
 * Cancel all gnome tasks.
 */
export function cancel_all_gnome_tasks() {
    wasm.cancel_all_gnome_tasks();
}

/**
 * Cancel gnome tasks at a specific position.
 * @param {number} x
 * @param {number} y
 * @param {number} z
 */
export function cancel_gnome_task(x, y, z) {
    wasm.cancel_gnome_task(x, y, z);
}

/**
 * Bitfield of discovered species. Bit N set = species N discovered.
 * JS can check `(discovered_species() >> speciesIdx) & 1` for each species.
 * @returns {number}
 */
export function discovered_species() {
    const ret = wasm.discovered_species();
    return ret >>> 0;
}

/**
 * Number of discovered species.
 * @returns {number}
 */
export function discovered_species_count() {
    const ret = wasm.discovered_species_count();
    return ret >>> 0;
}

/**
 * Number of active fauna creatures.
 * @returns {number}
 */
export function fauna_count() {
    const ret = wasm.fauna_count();
    return ret >>> 0;
}

/**
 * Length of fauna data in bytes.
 * @returns {number}
 */
export function fauna_len() {
    const ret = wasm.fauna_len();
    return ret >>> 0;
}

/**
 * Pointer to packed fauna data in WASM linear memory.
 * Each fauna is 16 bytes: [type: u8, state: u8, _pad: u8, _pad: u8, x: f32, y: f32, z: f32].
 * @returns {number}
 */
export function fauna_ptr() {
    const ret = wasm.fauna_ptr();
    return ret >>> 0;
}

/**
 * Fill a rectangular region with a tool.
 * Skips seeds and roots to protect living plants (matching CLI behavior).
 * @param {number} tool
 * @param {number} x1
 * @param {number} y1
 * @param {number} z1
 * @param {number} x2
 * @param {number} y2
 * @param {number} z2
 */
export function fill_tool(tool, x1, y1, z1, x2, y2, z2) {
    wasm.fill_tool(tool, x1, y1, z1, x2, y2, z2);
}

/**
 * Current day phase (0-99). Dawn=0-24, Day=25-49, Dusk=50-74, Night=75-99.
 * JS can use this to sync the visual day cycle with the sim's growth rhythm.
 * @returns {number}
 */
export function get_day_phase() {
    const ret = wasm.get_day_phase();
    return ret;
}

/**
 * Get focus position.
 * @returns {number}
 */
export function get_focus_x() {
    const ret = wasm.get_focus_x();
    return ret >>> 0;
}

/**
 * @returns {number}
 */
export function get_focus_y() {
    const ret = wasm.get_focus_y();
    return ret >>> 0;
}

/**
 * @returns {number}
 */
export function get_focus_z() {
    const ret = wasm.get_focus_z();
    return ret >>> 0;
}

/**
 * Get the currently selected species index.
 * @returns {number}
 */
export function get_selected_species() {
    const ret = wasm.get_selected_species();
    return ret >>> 0;
}

/**
 * Current tick count.
 * @returns {bigint}
 */
export function get_tick() {
    const ret = wasm.get_tick();
    return BigInt.asUintN(64, ret);
}

/**
 * Ticks remaining in current weather state.
 * @returns {number}
 */
export function get_weather_duration() {
    const ret = wasm.get_weather_duration();
    return ret >>> 0;
}

/**
 * Current weather state: 0=Clear, 1=Rain, 2=Drought.
 * JS can use this for visual effects (rain particles, dry palette shift).
 * @returns {number}
 */
export function get_weather_state() {
    const ret = wasm.get_weather_state();
    return ret;
}

/**
 * Length of ghost zone data in bytes (8 bytes per ghost).
 * @returns {number}
 */
export function ghost_len() {
    const ret = wasm.ghost_len();
    return ret >>> 0;
}

/**
 * Pointer to packed ghost zone data.
 * @returns {number}
 */
export function ghost_ptr() {
    const ret = wasm.ghost_ptr();
    return ret >>> 0;
}

/**
 * Length of gnome export data in bytes (always 32).
 * @returns {number}
 */
export function gnome_len() {
    const ret = wasm.gnome_len();
    return ret >>> 0;
}

/**
 * Pointer to packed gnome state (32 bytes).
 * @returns {number}
 */
export function gnome_ptr() {
    const ret = wasm.gnome_ptr();
    return ret >>> 0;
}

/**
 * Number of pending gnome tasks.
 * @returns {number}
 */
export function gnome_queue_len() {
    const ret = wasm.gnome_queue_len();
    return ret >>> 0;
}

/**
 * @returns {number}
 */
export function grid_depth() {
    const ret = wasm.grid_depth();
    return ret >>> 0;
}

/**
 * @returns {number}
 */
export function grid_height() {
    const ret = wasm.grid_height();
    return ret >>> 0;
}

/**
 * Length of the voxel grid in bytes.
 * @returns {number}
 */
export function grid_len() {
    const ret = wasm.grid_len();
    return ret >>> 0;
}

/**
 * Pointer to the raw voxel grid data in WASM linear memory.
 * JS can wrap this as `new Uint8Array(wasm.memory.buffer, ptr, len)`.
 * Each voxel is 4 bytes: [material, water_level, light_level, nutrient_level].
 * @returns {number}
 */
export function grid_ptr() {
    const ret = wasm.grid_ptr();
    return ret >>> 0;
}

/**
 * Grid dimensions as a packed u32: x | (y << 16). Z is returned separately.
 * @returns {number}
 */
export function grid_width() {
    const ret = wasm.grid_width();
    return ret >>> 0;
}

/**
 * Ground level Z coordinate (surface height baseline).
 * @returns {number}
 */
export function ground_level() {
    const ret = wasm.ground_level();
    return ret >>> 0;
}

/**
 * Initialize the simulation. Must be called before any other function.
 */
export function init() {
    wasm.init();
}

/**
 * Whether a specific species has been discovered.
 * @param {number} species_id
 * @returns {boolean}
 */
export function is_species_discovered(species_id) {
    const ret = wasm.is_species_discovered(species_id);
    return ret !== 0;
}

/**
 * Total number of material types.
 * @returns {number}
 */
export function material_count() {
    const ret = wasm.material_count();
    return ret;
}

/**
 * Whether a material is foliage (rendered as billboard sprites).
 * @param {number} idx
 * @returns {boolean}
 */
export function material_is_foliage(idx) {
    const ret = wasm.material_is_foliage(idx);
    return ret !== 0;
}

/**
 * Whether a material is a seed (rendered as small sprites).
 * @param {number} idx
 * @returns {boolean}
 */
export function material_is_seed(idx) {
    const ret = wasm.material_is_seed(idx);
    return ret !== 0;
}

/**
 * Whether a material generates solid mesh geometry.
 * @param {number} idx
 * @returns {boolean}
 */
export function material_is_solid(idx) {
    const ret = wasm.material_is_solid(idx);
    return ret !== 0;
}

/**
 * Name of material at the given index, or empty string if invalid.
 * @param {number} idx
 * @returns {string}
 */
export function material_name(idx) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.material_name(idx);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * Total active fauna count (progress toward tier 3).
 * @returns {number}
 */
export function milestone_fauna_count() {
    const ret = wasm.milestone_fauna_count();
    return ret;
}

/**
 * Groundcover leaf voxel count (progress toward tier 1).
 * @returns {number}
 */
export function milestone_groundcover_count() {
    const ret = wasm.milestone_groundcover_count();
    return ret;
}

/**
 * Active pollinator count (progress toward tier 2).
 * @returns {number}
 */
export function milestone_pollinator_count() {
    const ret = wasm.milestone_pollinator_count();
    return ret;
}

/**
 * Plant species diversity count (progress toward tier 3).
 * @returns {number}
 */
export function milestone_species_diversity() {
    const ret = wasm.milestone_species_diversity();
    return ret;
}

/**
 * Whether flowers are unlocked (tier 1: groundcover established).
 * @returns {boolean}
 */
export function milestone_tier1_flowers() {
    const ret = wasm.milestone_tier1_flowers();
    return ret !== 0;
}

/**
 * Whether shrubs are unlocked (tier 2: pollinators attracted).
 * @returns {boolean}
 */
export function milestone_tier2_shrubs() {
    const ret = wasm.milestone_tier2_shrubs();
    return ret !== 0;
}

/**
 * Whether trees are unlocked (tier 3: fauna ecosystem active).
 * @returns {boolean}
 */
export function milestone_tier3_trees() {
    const ret = wasm.milestone_tier3_trees();
    return ret !== 0;
}

/**
 * Pack tree stats into an export buffer for root war visualization.
 * Each tree: 12 bytes [species_id: u8, health_u8: u8, stage: u8, _pad: u8,
 *   root_x: u16le, root_y: u16le, root_count: u16le, water_intake: u16le]
 * Call this before reading tree_stats_ptr/len.
 * @returns {number}
 */
export function pack_tree_stats() {
    const ret = wasm.pack_tree_stats();
    return ret >>> 0;
}

/**
 * Pick a random discovered species of a given plant type.
 * plant_type: 0=Tree, 1=Shrub, 2=Flower, 3=Groundcover
 * Returns species_id, or 255 if no species of that type discovered.
 * @param {number} plant_type
 * @param {number} rng_hint
 * @returns {number}
 */
export function pick_discovered_species(plant_type, rng_hint) {
    const ret = wasm.pick_discovered_species(plant_type, rng_hint);
    return ret;
}

/**
 * Place a tool at (x, y, z). Tool codes: 0=shovel, 1=seed, 2=water (debug only — irrigation via digging replaces watering can), 3=soil, 4=stone.
 * Returns the landing z coordinate, or -1 if the action had no effect.
 * @param {number} tool
 * @param {number} x
 * @param {number} y
 * @param {number} z
 * @returns {number}
 */
export function place_tool(tool, x, y, z) {
    const ret = wasm.place_tool(tool, x, y, z);
    return ret;
}

/**
 * Queue a task for the garden gnome. Tool codes: 0=shovel, 1=seed, 2=water (debug only), 3=soil, 4=stone.
 * Species is used for seed tasks (pass 255 for non-seed tools).
 * Returns true if the task was queued, false if the queue is full.
 * @param {number} tool
 * @param {number} x
 * @param {number} y
 * @param {number} z
 * @param {number} species
 * @returns {boolean}
 */
export function queue_gnome_task(tool, x, y, z, species) {
    const ret = wasm.queue_gnome_task(tool, x, y, z, species);
    return ret !== 0;
}

/**
 * Set the day phase explicitly (for syncing JS day cycle to sim).
 * @param {number} phase
 */
export function set_day_phase(phase) {
    wasm.set_day_phase(phase);
}

/**
 * Set focus position.
 * @param {number} x
 * @param {number} y
 * @param {number} z
 */
export function set_focus(x, y, z) {
    wasm.set_focus(x, y, z);
}

/**
 * Set the species index used when placing seeds.
 * @param {number} idx
 */
export function set_selected_species(idx) {
    wasm.set_selected_species(idx);
}

/**
 * Length of the soil grid in bytes.
 * @returns {number}
 */
export function soil_len() {
    const ret = wasm.soil_len();
    return ret >>> 0;
}

/**
 * Pointer to the soil composition grid data.
 * Each cell is 6 bytes: [sand, clay, organic, rock, ph, bacteria].
 * @returns {number}
 */
export function soil_ptr() {
    const ret = wasm.soil_ptr();
    return ret >>> 0;
}

/**
 * Number of species in the species table.
 * @returns {number}
 */
export function species_count() {
    const ret = wasm.species_count();
    return ret >>> 0;
}

/**
 * Name of species at the given index.
 * @param {number} idx
 * @returns {string}
 */
export function species_name(idx) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.species_name(idx);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * Plant type name of species at the given index (Tree, Shrub, Ground, Flower).
 * @param {number} idx
 * @returns {string}
 */
export function species_plant_type(idx) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.species_plant_type(idx);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * Advance the simulation by `n` ticks.
 * @param {number} n
 */
export function tick(n) {
    wasm.tick(n);
}

/**
 * Number of available tools.
 * @returns {number}
 */
export function tool_count() {
    const ret = wasm.tool_count();
    return ret;
}

/**
 * Name of tool at the given index.
 * @param {number} idx
 * @returns {string}
 */
export function tool_name(idx) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.tool_name(idx);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * Length of packed tree stats buffer in bytes.
 * @returns {number}
 */
export function tree_stats_len() {
    const ret = wasm.tree_stats_len();
    return ret >>> 0;
}

/**
 * Pointer to packed tree stats buffer. Call pack_tree_stats() first.
 * @returns {number}
 */
export function tree_stats_ptr() {
    const ret = wasm.tree_stats_ptr();
    return ret >>> 0;
}

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg_error_a6fa202b58aa1cd3: function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_new_227d7c05414eb861: function() {
            const ret = new Error();
            return ret;
        },
        __wbg_stack_3b0d974bbf31e44f: function(arg0, arg1) {
            const ret = arg1.stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./groundwork_sim_bg.js": import0,
    };
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('groundwork_sim_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
