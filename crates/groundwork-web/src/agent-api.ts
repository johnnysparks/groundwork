/**
 * Agent API — programmatic interface for the player agent's Playwright harness.
 *
 * Exposes `window.agentAPI` with methods to drive the simulation and capture
 * screenshots from the Three.js renderer. The Playwright harness reads a trace
 * JSON (from groundwork-player), replays actions through this API, and captures
 * actual rendered PNGs at Screenshot action points.
 *
 * This is the bridge between the Rust player agent and the visual renderer.
 */

import { isInitialized, tick as simTick, placeTool, fillTool, setSelectedSpecies, getGridView, getTick, getFaunaCount, GRID_X, GRID_Y, GRID_Z, GROUND_LEVEL, SPECIES, ToolCode } from './bridge';
import { captureScreenshot } from './ui/screenshot';

/** Action types matching groundwork-player's Action enum */
interface TickAction { type: 'Tick'; n: number }
interface PlaceAction { type: 'Place'; tool: string; x: number; y: number; z: number; species?: string }
interface FillAction { type: 'Fill'; tool: string; x1: number; y1: number; z1: number; x2: number; y2: number; z2: number }
interface CameraOrbitAction { type: 'CameraOrbit'; theta_deg: number; phi_deg: number }
interface CameraPanAction { type: 'CameraPan'; x: number; y: number; z: number }
interface CameraZoomAction { type: 'CameraZoom'; level: number }
interface CameraCutawayAction { type: 'CameraCutaway'; z: number }
interface CameraResetAction { type: 'CameraReset' }
interface ScreenshotAction { type: 'Screenshot'; label: string }
interface StatusAction { type: 'Status' }
interface InspectAction { type: 'Inspect'; x: number; y: number; z: number }
interface ViewAction { type: 'View'; z: number }
interface CheckpointAction { type: 'Checkpoint'; label: string }

type AgentAction =
  | TickAction | PlaceAction | FillAction
  | CameraOrbitAction | CameraPanAction | CameraZoomAction
  | CameraCutawayAction | CameraResetAction
  | ScreenshotAction | StatusAction | InspectAction | ViewAction | CheckpointAction;

/** Trace step from groundwork-player JSON output */
interface TraceStep {
  action: AgentAction;
  observation: { text: string; tick: number };
}

/** Full trace from groundwork-player */
interface AgentTrace {
  scenario_name: string;
  steps: TraceStep[];
}

/** Result of executing an action */
interface ActionResult {
  action: string;
  tick: number;
  screenshotBlob?: Blob;
  screenshotLabel?: string;
}

/** Resolve a tool name (from Rust Action) to the ToolCode number */
function resolveToolCode(tool: string): number {
  const lower = tool.toLowerCase();
  switch (lower) {
    case 'air': case 'dig': return ToolCode.Shovel;
    case 'seed': return ToolCode.Seed;
    case 'water': return ToolCode.Water;
    case 'soil': return ToolCode.Soil;
    case 'stone': return ToolCode.Stone;
    default: return ToolCode.Seed; // species names → seed tool
  }
}

/** Resolve species name to index */
function resolveSpeciesIndex(name?: string): number {
  if (!name) return 0;
  const lower = name.toLowerCase().replace(/[_-]/g, ' ');
  const idx = SPECIES.findIndex(s => s.name.toLowerCase() === lower);
  return idx >= 0 ? idx : 0;
}

// Store references set by main.ts
let _orbitCamera: any = null;
let _remeshDirty: (() => void) | null = null;
let _dayCycle: any = null;
let _setXrayMode: ((active: boolean) => void) | null = null;
let _setTickCount: ((count: number) => void) | null = null;
let _overlay: any = null;
let _taskQueue: any = null;
let _scenery: { forestRing: any; skirt: any; scene: any } | null = null;

export interface AgentAPIConfig {
  orbitCamera: any;
  remeshDirty: () => void;
  dayCycle?: any;
  setXrayMode?: (active: boolean) => void;
  setTickCount?: (count: number) => void;
  overlay?: any;
  taskQueue?: any;
  scenery?: { forestRing: any; skirt: any; scene: any };
}

/** Initialize the agent API with references to scene objects */
export function initAgentAPI(config: AgentAPIConfig): void {
  _orbitCamera = config.orbitCamera;
  _remeshDirty = config.remeshDirty;
  _dayCycle = config.dayCycle;
  _setXrayMode = config.setXrayMode ?? null;
  _setTickCount = config.setTickCount ?? null;
  _overlay = config.overlay ?? null;
  _taskQueue = config.taskQueue ?? null;
  _scenery = config.scenery ?? null;

  const api = {
    /** Check if sim is ready */
    isReady: () => isInitialized(),

    /** Get current tick */
    getTick: () => isInitialized() ? Number(getTick()) : 0,

    /** Get grid info */
    getGridInfo: () => ({ width: GRID_X, height: GRID_Y, depth: GRID_Z, groundLevel: GROUND_LEVEL }),

    /** Get fauna count */
    getFaunaCount: () => getFaunaCount(),

    /** Set time of day (0–1): 0.25=dawn, 0.5=noon, 0.75=golden, 0.0=blue hour */
    setTimeOfDay: (t: number) => { if (_dayCycle) _dayCycle.setTime(t); },

    /** Queue a task for the garden gnome (instead of instant placement) */
    queueTask: (tool: number, x: number, y: number, z: number, species?: number) => {
      if (_taskQueue) {
        _taskQueue.enqueue({ tool, x, y, z, species });
      }
    },

    /** Get gnome task queue length */
    getQueueLength: () => _taskQueue?.length ?? 0,

    /** Set overlay mode: 0=off, 1=water, 2=light, 3=nutrient */
    setOverlay: (mode: number) => {
      if (_overlay) {
        _overlay.setMode(mode);
        if (mode > 0) {
          const g = isInitialized() ? getGridView() : null;
          if (g) _overlay.rebuild(g);
        }
      }
    },

    /** Execute a single action and return the result */
    executeAction: async (action: AgentAction): Promise<ActionResult> => {
      return executeAgentAction(action);
    },

    /** Execute a full trace (array of steps) with screenshots */
    replayTrace: async (trace: AgentTrace): Promise<ActionResult[]> => {
      const results: ActionResult[] = [];
      for (const step of trace.steps) {
        const result = await executeAgentAction(step.action);
        results.push(result);
        // Small delay between actions so renderer can update
        await new Promise(r => setTimeout(r, 50));
      }
      return results;
    },

    /** Capture a screenshot right now */
    captureScreenshot: async (label?: string): Promise<Blob | null> => {
      // Wait for a frame to render
      await new Promise(r => requestAnimationFrame(r));
      await new Promise(r => requestAnimationFrame(r));
      return captureScreenshot();
    },

    /** Toggle decorative scenery (forest ring + skirt) for clean captures */
    setSceneryVisible: (visible: boolean) => {
      if (_scenery) {
        _scenery.forestRing.visible = visible;
        _scenery.skirt.visible = visible;
      }
    },

    /** Toggle fog for clean captures */
    setFogEnabled: (enabled: boolean) => {
      if (_scenery?.scene) {
        if (!enabled) {
          _scenery.scene._savedFog = _scenery.scene.fog;
          _scenery.scene.fog = null;
        } else if (_scenery.scene._savedFog) {
          _scenery.scene.fog = _scenery.scene._savedFog;
        }
      }
    },

    /** Hide all HTML overlay elements for clean screenshots */
    hideUI: () => {
      document.querySelectorAll('div, span, button, select, label, p, h1, h2, h3, h4').forEach(el => {
        const htmlEl = el as HTMLElement;
        if (htmlEl.tagName !== 'CANVAS' && htmlEl.id !== 'app' && !htmlEl.contains(document.querySelector('canvas'))) {
          htmlEl.style.setProperty('display', 'none', 'important');
        }
      });
      document.querySelectorAll('[style*="position: absolute"], [style*="position: fixed"]').forEach(el => {
        const htmlEl = el as HTMLElement;
        if (htmlEl.tagName !== 'CANVAS') {
          htmlEl.style.setProperty('display', 'none', 'important');
        }
      });
    },

    /** Get material counts from current grid */
    getMaterialCounts: () => {
      if (!isInitialized()) return {};
      const grid = getGridView();
      const counts: Record<string, number> = {};
      const names = ['air', 'soil', 'stone', 'water', 'root', 'seed', 'trunk', 'branch', 'leaf', 'deadwood'];
      for (let i = 0; i < names.length; i++) counts[names[i]] = 0;
      for (let i = 0; i < grid.length; i += 4) {
        const mat = grid[i];
        if (mat < names.length) counts[names[mat]]++;
      }
      return counts;
    },

    /** Get voxel counts per species (only plant materials: seed, trunk, branch, leaf, root) */
    getSpeciesCounts: () => {
      if (!isInitialized()) return {};
      const grid = getGridView();
      const plantMats = new Set([4, 6, 7, 8]); // root, trunk, branch, leaf (not seed — byte 3 is growth counter)
      const maxSpeciesId = SPECIES.length; // valid IDs: 0..11
      const speciesCounts: Record<number, number> = {};
      let invalidCount = 0;
      for (let i = 0; i < grid.length; i += 4) {
        if (!plantMats.has(grid[i])) continue;
        const speciesId = grid[i + 3];
        if (speciesId >= maxSpeciesId) { invalidCount++; continue; }
        speciesCounts[speciesId] = (speciesCounts[speciesId] ?? 0) + 1;
      }
      // Map species IDs to names
      const result: Record<string, number> = {};
      for (const [id, count] of Object.entries(speciesCounts)) {
        const sp = SPECIES.find(s => s.index === Number(id));
        result[sp?.name ?? `id-${id}`] = count;
      }
      if (invalidCount > 0) result['_invalid'] = invalidCount;
      return result;
    },
  };

  (window as any).agentAPI = api;
}

/** Execute a single action against the sim + renderer */
async function executeAgentAction(action: AgentAction): Promise<ActionResult> {
  const currentTick = isInitialized() ? Number(getTick()) : 0;

  switch (action.type) {
    case 'Tick': {
      if (!isInitialized()) return { action: `tick ${action.n} (mock — no-op)`, tick: 0 };
      simTick(action.n);
      _remeshDirty?.();
      const tick = Number(getTick());
      _setTickCount?.(tick);
      return { action: `tick ${action.n}`, tick };
    }

    case 'Place': {
      if (!isInitialized()) {
        // In mock mode, apply directly to grid via the remeshDirty path
        // (the mock grid tool application is handled by main.ts's applyToolToMockGrid)
        return { action: `place ${action.species || action.tool} (mock — no-op)`, tick: 0 };
      }
      const speciesIdx = resolveSpeciesIndex(action.species || action.tool);
      setSelectedSpecies(speciesIdx);
      const toolCode = resolveToolCode(action.species ? 'seed' : action.tool);
      placeTool(toolCode, action.x, action.y, action.z);
      _remeshDirty?.();
      return { action: `place ${action.species || action.tool} at (${action.x},${action.y},${action.z})`, tick: Number(getTick()) };
    }

    case 'Fill': {
      if (!isInitialized()) return { action: `fill ${action.tool} (mock — no-op)`, tick: 0 };
      const toolCode = resolveToolCode(action.tool);
      fillTool(toolCode, action.x1, action.y1, action.z1, action.x2, action.y2, action.z2);
      _remeshDirty?.();
      return { action: `fill ${action.tool}`, tick: Number(getTick()) };
    }

    case 'CameraOrbit': {
      if (_orbitCamera) {
        // OrbitCamera uses rotate(dTheta, dPhi) for relative movement.
        // For absolute positioning, reset then rotate to the target.
        _orbitCamera.reset();
        const targetTheta = action.theta_deg * Math.PI / 180;
        const targetPhi = action.phi_deg * Math.PI / 180;
        const defaultTheta = Math.PI / 4; // 45 deg
        const defaultPhi = Math.PI / 3;   // 60 deg
        _orbitCamera.rotate(targetTheta - defaultTheta, targetPhi - defaultPhi);
        _orbitCamera.snap(); // Apply instantly for headless/automated use
      }
      return { action: `camera orbit ${action.theta_deg}° ${action.phi_deg}°`, tick: currentTick };
    }

    case 'CameraPan': {
      if (_orbitCamera) {
        // Sim coords (x, y, z) → Three.js Y-up (x, z_sim, y_sim)
        _orbitCamera.setCenter(action.x, action.z, action.y);
        _orbitCamera.snap();
      }
      return { action: `camera pan (${action.x},${action.y},${action.z})`, tick: currentTick };
    }

    case 'CameraZoom': {
      if (_orbitCamera) {
        // zoom() takes a factor; to set absolute, reset first then scale
        _orbitCamera.zoom(action.level);
        _orbitCamera.snap();
      }
      return { action: `camera zoom ${action.level}x`, tick: currentTick };
    }

    case 'CameraCutaway': {
      if (_setXrayMode) {
        // z > 0 enables x-ray mode, z === 0 disables it
        _setXrayMode(action.z > 0);
      }
      return { action: `camera cutaway z=${action.z}`, tick: currentTick };
    }

    case 'CameraReset': {
      if (_orbitCamera) {
        _orbitCamera.reset();
        _orbitCamera.snap();
      }
      return { action: 'camera reset', tick: currentTick };
    }

    case 'Screenshot': {
      // Wait for render to complete
      await new Promise(r => requestAnimationFrame(r));
      await new Promise(r => requestAnimationFrame(r));
      const blob = await captureScreenshot();
      return {
        action: `screenshot "${action.label}"`,
        tick: currentTick,
        screenshotBlob: blob ?? undefined,
        screenshotLabel: action.label,
      };
    }

    case 'Status':
    case 'Inspect':
    case 'View':
    case 'Checkpoint':
      // These are observation-only actions handled by the Rust side.
      // In browser mode they're no-ops (info already visible in the renderer).
      return { action: action.type, tick: currentTick };

    default:
      return { action: 'unknown', tick: currentTick };
  }
}
