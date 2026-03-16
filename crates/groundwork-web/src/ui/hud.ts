/**
 * HUD overlay: tool palette, species picker, and status display.
 *
 * Built as HTML/CSS overlay on top of the Three.js canvas for crisp text
 * and standard UI behavior. Communicates tool/species selection via callbacks.
 */

import { ToolCode, type ToolCodeType, TOOLS as BRIDGE_TOOLS, SPECIES as BRIDGE_SPECIES, type SpeciesDef, setSelectedSpecies, isInitialized } from '../bridge';
import { SCENES, getSceneId, switchScene } from '../mesher/mockGrid';

/** Re-export SPECIES so other UI modules can import from hud */
export { BRIDGE_SPECIES as SPECIES };
import { captureScreenshot } from './screenshot';

/** Tool UI definition (extends bridge ToolDef with display properties) */
export interface ToolUIDef {
  code: ToolCodeType;
  name: string;
  icon: string;
  key: string;
  description: string;
}

/** UI metadata for tools (icon, hotkey, description). Keyed by tool name. */
const TOOL_UI: Record<string, { icon: string; description: string }> = {
  Shovel: { icon: 'Dig', description: 'Clear an area' },
  Seed:   { icon: 'Plant', description: 'Zone with seeds (costs 15 water)' },
  Water:  { icon: 'Water', description: 'Irrigate an area (costs 20 water)' },
  Soil:   { icon: 'Soil', description: 'Place soil (costs 10 water)' },
  Stone:  { icon: 'Stone', description: 'Place stone (costs 10 water)' },
};

/** Build tool UI definitions from engine-provided tool list */
function buildTools(): ToolUIDef[] {
  return BRIDGE_TOOLS.map((t, i) => {
    const ui = TOOL_UI[t.name] ?? { icon: t.name.charAt(0), description: t.name };
    return { code: t.code, name: t.name, icon: ui.icon, key: String(i + 1), description: ui.description };
  });
}

export { type SpeciesDef } from '../bridge';

export interface HudState {
  activeTool: ToolCodeType;
  activeSpeciesIndex: number;
  autoTick: boolean;
  tickCount: number;
  gardenStats?: { plants: number; fauna: number; species: number };
  water: number;
  maxWater: number;
  queueCount?: number;
}

type HudChangeCallback = (state: HudState) => void;

/**
 * Creates and manages the HUD overlay DOM elements.
 */
export class Hud {
  readonly state: HudState = {
    activeTool: ToolCode.Seed,
    activeSpeciesIndex: 0,
    autoTick: false,
    tickCount: 0,
    water: 150,
    maxWater: 150,
  };

  private _lastScore = 0;
  private _onNewGarden: (() => void) | null = null;
  private tools: ToolUIDef[];
  private species: SpeciesDef[];
  private container: HTMLElement;
  private toolButtons: HTMLElement[] = [];
  private speciesButtons: HTMLElement[] = [];
  private speciesPanel: HTMLElement;
  private statusEl: HTMLElement;
  private onChange: HudChangeCallback | null = null;

  constructor() {
    this.tools = buildTools();
    this.species = BRIDGE_SPECIES;

    this.container = document.createElement('div');
    this.container.id = 'hud';
    this.container.innerHTML = HUD_HTML;
    document.body.appendChild(this.container);

    // Inject styles
    const style = document.createElement('style');
    style.textContent = HUD_CSS;
    document.head.appendChild(style);

    // Wire up tool buttons
    const toolBar = this.container.querySelector('#tool-bar')!;
    for (const tool of this.tools) {
      const btn = document.createElement('button');
      btn.className = 'tool-btn';
      btn.dataset.tool = String(tool.code);
      btn.title = `${tool.name} [${tool.key}] — ${tool.description}`;
      btn.innerHTML = `<span class="tool-icon">${tool.icon}</span><span class="tool-key">${tool.key}</span>`;
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        this.selectTool(tool.code);
      });
      toolBar.appendChild(btn);
      this.toolButtons.push(btn);
    }

    // Wire up species picker (data comes from the engine via bridge)
    this.speciesPanel = this.container.querySelector('#species-panel')! as HTMLElement;
    const speciesList = this.speciesPanel.querySelector('#species-list')!;
    let currentType = '';
    for (const sp of this.species) {
      if (sp.type !== currentType) {
        currentType = sp.type;
        const header = document.createElement('div');
        header.className = 'species-group-header';
        header.textContent = sp.type;
        speciesList.appendChild(header);
      }
      const btn = document.createElement('button');
      btn.className = 'species-btn';
      btn.dataset.speciesIndex = String(sp.index);
      btn.textContent = sp.name;
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        this.selectSpecies(sp.index);
      });
      speciesList.appendChild(btn);
      this.speciesButtons.push(btn);
    }

    // Status element
    this.statusEl = this.container.querySelector('#hud-status')! as HTMLElement;

    // Auto-tick toggle
    const tickToggle = this.container.querySelector('#tick-toggle')!;
    tickToggle.addEventListener('click', (e) => {
      e.stopPropagation();
      this.state.autoTick = !this.state.autoTick;
      this.render();
      this.notify();
    });

    // Screenshot button
    const screenshotBtn = this.container.querySelector('#screenshot-btn')!;
    screenshotBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      captureScreenshot();
    });

    // Scene selector
    const sceneSelect = this.container.querySelector('#scene-select') as HTMLSelectElement;
    const wasmAvailable = isInitialized();
    const currentScene = getSceneId(wasmAvailable);
    for (const scene of SCENES) {
      const opt = document.createElement('option');
      opt.value = scene.id;
      opt.textContent = scene.name;
      opt.title = scene.description;
      // Disable sim option if WASM not available
      if (scene.id === 'sim' && !wasmAvailable) {
        opt.disabled = true;
        opt.textContent += ' (no wasm)';
      }
      if (scene.id === currentScene) opt.selected = true;
      sceneSelect.appendChild(opt);
    }
    sceneSelect.addEventListener('change', () => {
      switchScene(sceneSelect.value);
    });

    // New Garden button
    const newGardenBtn = this.container.querySelector('#new-garden-btn')!;
    newGardenBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      this._onNewGarden?.();
    });

    // Set initial state
    this.render();
  }

  /** Register a callback for state changes */
  onStateChange(cb: HudChangeCallback): void {
    this.onChange = cb;
  }

  /** Select a tool by code */
  selectTool(code: ToolCodeType): void {
    this.state.activeTool = code;
    this.render();
    this.notify();
  }

  /** Select a species by index */
  selectSpecies(index: number): void {
    this.state.activeSpeciesIndex = index;
    if (isInitialized()) {
      setSelectedSpecies(index);
    }
    this.render();
    this.notify();
  }

  /** Cycle to next species */
  cycleSpecies(direction: 1 | -1): void {
    const idx = this.species.findIndex(s => s.index === this.state.activeSpeciesIndex);
    const next = (idx + direction + this.species.length) % this.species.length;
    this.selectSpecies(this.species[next].index);
  }

  /** Update tick count display */
  setTickCount(count: number): void {
    this.state.tickCount = count;
    this.renderStatus();
  }

  /** Update the auto-tick state (called from main when spacebar is pressed) */
  setAutoTick(on: boolean): void {
    this.state.autoTick = on;
    this.render();
  }

  /** Returns true if the given tool places material (vs. digging) */
  isPlacingTool(): boolean {
    return this.state.activeTool !== ToolCode.Shovel;
  }

  /** Check if a DOM element is part of the HUD (to prevent click-through) */
  containsElement(el: EventTarget | null): boolean {
    if (!el || !(el instanceof HTMLElement)) return false;
    return this.container.contains(el);
  }

  private notify(): void {
    this.onChange?.({ ...this.state });
  }

  private render(): void {
    // Update tool buttons
    for (const btn of this.toolButtons) {
      const code = Number(btn.dataset.tool);
      btn.classList.toggle('active', code === this.state.activeTool);
    }

    // Show/hide species panel based on whether seed tool is active
    this.speciesPanel.classList.toggle('visible', this.state.activeTool === ToolCode.Seed);

    // Update species buttons
    for (const btn of this.speciesButtons) {
      btn.classList.toggle('active', btn.dataset.speciesIndex === String(this.state.activeSpeciesIndex));
    }

    this.renderStatus();
  }

  /** Update garden stats (call after tick with fresh grid data) */
  setGardenStats(stats: { plants: number; fauna: number; species: number }): void {
    this.state.gardenStats = stats;
    this.renderStatus();
    // Update score panel
    const score = Math.round(stats.plants * 0.1 + stats.fauna * 50 + stats.species * 100);

    // Check milestones
    const milestones = [500, 1000, 2000, 5000, 10000];
    const prevScore = this._lastScore;
    for (const m of milestones) {
      if (prevScore < m && score >= m) {
        this.showMilestone(m);
      }
    }
    this._lastScore = score;

    const scoreEl = this.container.querySelector('#score-number');
    if (scoreEl) scoreEl.textContent = score.toLocaleString();
    const plantsEl = this.container.querySelector('#stat-plants');
    if (plantsEl) plantsEl.textContent = stats.plants.toLocaleString();
    const faunaEl = this.container.querySelector('#stat-fauna');
    if (faunaEl) faunaEl.textContent = String(stats.fauna);
    const speciesEl = this.container.querySelector('#stat-species');
    if (speciesEl) speciesEl.textContent = String(stats.species);
  }

  /** Show a milestone celebration toast */
  private showMilestone(score: number): void {
    const titles: Record<number, string> = {
      500: 'Sprout',
      1000: 'Seedling Garden',
      2000: 'Thriving Grove',
      5000: 'Living Ecosystem',
      10000: 'Master Gardener',
    };
    const title = titles[score] || `Score ${score}`;
    const el = document.createElement('div');
    el.className = 'milestone-toast';
    el.innerHTML = `<div class="milestone-score">${score.toLocaleString()}</div><div class="milestone-title">${title}</div>`;
    this.container.appendChild(el);
    // Animate in
    requestAnimationFrame(() => el.classList.add('visible'));
    // Remove after 3s
    setTimeout(() => {
      el.classList.remove('visible');
      setTimeout(() => el.remove(), 500);
    }, 3000);
  }

  /** Register callback for New Garden button */
  onNewGarden(cb: () => void): void { this._onNewGarden = cb; }

  /** Reset HUD state for a new garden */
  resetForNewGarden(): void {
    this.state.tickCount = 0;
    this.state.water = this.state.maxWater;
    this.state.gardenStats = undefined;
    this._lastScore = 0;
    this.updateWaterBar();
    this.renderStatus();
    const scoreEl = this.container.querySelector('#score-number');
    if (scoreEl) scoreEl.textContent = '0';
    const plantsEl = this.container.querySelector('#stat-plants');
    if (plantsEl) plantsEl.textContent = '0';
    const faunaEl = this.container.querySelector('#stat-fauna');
    if (faunaEl) faunaEl.textContent = '0';
    const speciesEl = this.container.querySelector('#stat-species');
    if (speciesEl) speciesEl.textContent = '0';
    // Clear event feed
    const feed = this.container.querySelector('#event-feed');
    if (feed) feed.innerHTML = '';
  }

  /** Spend water — returns false if not enough */
  spendWater(amount: number): boolean {
    if (this.state.water < amount) return false;
    this.state.water = Math.max(0, this.state.water - amount);
    this.updateWaterBar();
    return true;
  }

  /** Replenish water (called each tick) */
  replenishWater(amount: number): void {
    this.state.water = Math.min(this.state.maxWater, this.state.water + amount);
    this.updateWaterBar();
  }

  private updateWaterBar(): void {
    const fill = this.container.querySelector('#water-bar-fill') as HTMLElement;
    if (fill) {
      const pct = (this.state.water / this.state.maxWater) * 100;
      fill.style.width = `${pct}%`;
      // Color shifts: blue when full, orange when low
      if (pct < 20) {
        fill.style.background = 'linear-gradient(90deg, #cc6633, #dd8844)';
      } else if (pct < 50) {
        fill.style.background = 'linear-gradient(90deg, #5599bb, #66aacc)';
      } else {
        fill.style.background = 'linear-gradient(90deg, #3388cc, #55aadd)';
      }
    }
  }

  /** Add an event to the feed */
  addEvent(text: string): void {
    const feed = this.container.querySelector('#event-feed');
    if (!feed) return;
    const el = document.createElement('div');
    el.className = 'event-item';
    el.textContent = text;
    feed.appendChild(el);
    // Keep max 5 items
    while (feed.children.length > 5) {
      feed.firstElementChild?.remove();
    }
    // Fade out after 8s
    setTimeout(() => {
      el.classList.add('fading');
      setTimeout(() => el.remove(), 500);
    }, 8000);
  }

  /** Set the gnome task queue count for HUD display */
  setQueueCount(count: number): void {
    this.state.queueCount = count;
    this.renderStatus();
  }

  private renderStatus(): void {
    const tickState = this.state.autoTick ? 'ON' : 'OFF';
    const queueStr = (this.state.queueCount ?? 0) > 0 ? ` | Tasks: ${this.state.queueCount}` : '';
    this.statusEl.textContent = `Tick: ${this.state.tickCount} | Auto: ${tickState} [Space]${queueStr}`;
  }
}

// --- HTML Template ---

const HUD_HTML = `
  <div id="tool-bar"></div>
  <div id="species-panel">
    <div id="species-list"></div>
  </div>
  <div id="garden-score">
    <div id="score-title">Garden</div>
    <div id="score-number">0</div>
    <div id="score-details">
      <div class="score-row"><span class="score-label">Plants</span><span id="stat-plants">0</span></div>
      <div class="score-row"><span class="score-label">Fauna</span><span id="stat-fauna">0</span></div>
      <div class="score-row"><span class="score-label">Species</span><span id="stat-species">0</span></div>
    </div>
    <div id="water-bar-container">
      <div id="water-bar-label">Water</div>
      <div id="water-bar-track"><div id="water-bar-fill"></div></div>
    </div>
  </div>
  <div id="event-feed"></div>
  <div id="hud-top-bar">
    <div id="hud-status"></div>
    <div id="hud-controls">
      <button id="tick-toggle" title="Toggle auto-tick [Space]">Tick</button>
      <button id="screenshot-btn" title="Capture screenshot [F2]">Snap</button>
      <select id="scene-select" title="Choose a scene"></select>
    </div>
  </div>
  <div id="hud-help">Drag: orbit | Scroll: zoom | 1-5: tools | Z/C: species | Q: x-ray | V: overlay</div>
  <button id="new-garden-btn" title="Start a fresh garden">New Garden</button>
`;

// --- CSS ---

const HUD_CSS = `
#hud {
  position: absolute;
  inset: 0;
  pointer-events: none;
  font-family: system-ui, -apple-system, sans-serif;
  z-index: 10;
  user-select: none;
}

/* --- Tool Bar (bottom center) --- */
#tool-bar {
  position: absolute;
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: 4px;
  padding: 6px;
  background: rgba(20, 18, 15, 0.85);
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(8px);
  pointer-events: auto;
}

.tool-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 8px 12px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.05);
  color: #b8a88a;
  cursor: pointer;
  transition: all 0.15s ease;
  font-family: inherit;
}
.tool-btn:hover {
  background: rgba(255, 255, 255, 0.12);
  color: #e8d8b8;
}
.tool-btn.active {
  background: rgba(120, 90, 50, 0.5);
  border-color: rgba(200, 170, 100, 0.6);
  color: #ffe4b5;
}
.tool-icon {
  font-size: 13px;
  font-weight: 600;
  line-height: 1;
}
.tool-key {
  font-size: 9px;
  opacity: 0.5;
}
.tool-label {
  font-size: 10px;
  opacity: 0.7;
}

/* --- Species Panel (above tool bar, slides up when seed tool active) --- */
#species-panel {
  position: absolute;
  bottom: 90px;
  left: 50%;
  transform: translateX(-50%) translateY(10px);
  display: flex;
  flex-direction: column;
  padding: 8px;
  background: rgba(20, 18, 15, 0.85);
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(8px);
  pointer-events: auto;
  opacity: 0;
  visibility: hidden;
  transition: all 0.2s ease;
  max-width: 320px;
}
#species-panel.visible {
  opacity: 1;
  visibility: visible;
  transform: translateX(-50%) translateY(0);
}

#species-list {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.species-group-header {
  width: 100%;
  font-size: 9px;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: rgba(180, 160, 130, 0.5);
  padding: 4px 4px 0;
  margin-top: 2px;
}
.species-group-header:first-child {
  margin-top: 0;
}

.species-btn {
  padding: 4px 10px;
  border: 1px solid transparent;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.05);
  color: #b8a88a;
  cursor: pointer;
  font-size: 12px;
  font-family: inherit;
  transition: all 0.12s ease;
}
.species-btn:hover {
  background: rgba(255, 255, 255, 0.12);
  color: #e8d8b8;
}
.species-btn.active {
  background: rgba(80, 120, 50, 0.5);
  border-color: rgba(140, 190, 80, 0.6);
  color: #c4e890;
}

/* --- Top bar (status + controls) --- */
#hud-top-bar {
  position: absolute;
  top: 12px;
  left: 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 10px;
  background: rgba(20, 18, 15, 0.85);
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(8px);
  pointer-events: auto;
}

#hud-status {
  font-size: 12px;
  color: rgba(200, 180, 140, 0.7);
  white-space: nowrap;
}

#hud-controls {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-left: 4px;
  padding-left: 8px;
  border-left: 1px solid rgba(255, 255, 255, 0.08);
}

/* --- Help (bottom-right, above New Garden) --- */
#hud-help {
  position: absolute;
  bottom: 96px;
  right: 16px;
  font-size: 10px;
  color: rgba(200, 180, 140, 0.3);
  pointer-events: none;
  text-align: right;
  max-width: 200px;
  line-height: 1.5;
}

/* --- Shared button style for top controls --- */
#tick-toggle,
#screenshot-btn {
  padding: 3px 8px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.05);
  color: #b8a88a;
  cursor: pointer;
  font-size: 11px;
  font-family: inherit;
  transition: all 0.12s ease;
}
#tick-toggle:hover,
#screenshot-btn:hover {
  background: rgba(255, 255, 255, 0.12);
  color: #e8d8b8;
}
#new-garden-btn {
  position: absolute;
  bottom: 64px;
  right: 16px;
  padding: 6px 14px;
  border: 1px solid rgba(120, 180, 80, 0.3);
  border-radius: 6px;
  background: rgba(40, 60, 30, 0.7);
  color: #a8d888;
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  font-family: inherit;
  pointer-events: auto;
  transition: all 0.15s ease;
}
#new-garden-btn:hover {
  background: rgba(60, 100, 40, 0.8);
  border-color: rgba(140, 200, 100, 0.5);
  color: #c8f0a8;
}

/* --- Milestone Toast --- */
.milestone-toast {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%) scale(0.8);
  text-align: center;
  background: rgba(20, 18, 12, 0.92);
  border: 2px solid rgba(255, 200, 80, 0.4);
  border-radius: 16px;
  padding: 20px 40px;
  backdrop-filter: blur(12px);
  opacity: 0;
  transition: opacity 0.4s ease, transform 0.4s ease;
  pointer-events: none;
  z-index: 100;
}
.milestone-toast.visible {
  opacity: 1;
  transform: translate(-50%, -50%) scale(1);
}
.milestone-score {
  font-size: 48px;
  font-weight: 800;
  color: #ffc850;
  text-shadow: 0 0 20px rgba(255, 200, 80, 0.5);
  line-height: 1;
}
.milestone-title {
  font-size: 16px;
  color: rgba(255, 220, 140, 0.8);
  letter-spacing: 2px;
  text-transform: uppercase;
  margin-top: 6px;
}

/* --- Event Feed (bottom left) --- */
#event-feed {
  position: absolute;
  bottom: 80px;
  left: 16px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-width: 300px;
}
.event-item {
  font-size: 12px;
  color: rgba(200, 190, 160, 0.85);
  background: rgba(20, 18, 15, 0.7);
  padding: 4px 10px;
  border-radius: 6px;
  border-left: 3px solid rgba(255, 200, 80, 0.5);
  opacity: 1;
  transition: opacity 0.5s ease;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.event-item.fading { opacity: 0; }

/* --- Water Bar --- */
#water-bar-container {
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}
#water-bar-label {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: rgba(120, 180, 220, 0.7);
  margin-bottom: 4px;
}
#water-bar-track {
  height: 6px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 3px;
  overflow: hidden;
}
#water-bar-fill {
  height: 100%;
  width: 100%;
  background: linear-gradient(90deg, #3388cc, #55aadd);
  border-radius: 3px;
  transition: width 0.3s ease;
}

/* --- Garden Score Panel (top right) --- */
#garden-score {
  position: absolute;
  top: 16px;
  right: 16px;
  background: rgba(20, 18, 15, 0.88);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 12px 16px;
  backdrop-filter: blur(8px);
  min-width: 120px;
  text-align: center;
}
#score-title {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 1.5px;
  color: rgba(200, 180, 140, 0.6);
  margin-bottom: 2px;
}
#score-number {
  font-size: 32px;
  font-weight: 700;
  color: #e8d8b8;
  line-height: 1.1;
  margin-bottom: 8px;
}
#score-details {
  display: flex;
  flex-direction: column;
  gap: 3px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  padding-top: 8px;
}
.score-row {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: rgba(200, 180, 140, 0.7);
}
.score-row span:last-child {
  color: #e8d8b8;
  font-weight: 500;
}

/* --- Scene selector (inline in top bar) --- */
#scene-select {
  padding: 3px 6px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.05);
  color: #b8a88a;
  cursor: pointer;
  font-size: 11px;
  font-family: inherit;
  transition: all 0.12s ease;
  -webkit-appearance: none;
  appearance: none;
}
#scene-select:hover {
  background: rgba(255, 255, 255, 0.12);
  color: #e8d8b8;
}
#scene-select option {
  background: #1a1815;
  color: #b8a88a;
}
#scene-select option:disabled {
  color: #665e50;
}
`;
