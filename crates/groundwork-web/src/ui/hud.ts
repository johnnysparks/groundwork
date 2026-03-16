/**
 * HUD overlay: tool palette, species picker, and status display.
 *
 * Built as HTML/CSS overlay on top of the Three.js canvas for crisp text
 * and standard UI behavior. Communicates tool/species selection via callbacks.
 */

import { ToolCode, type ToolCodeType, TOOLS as BRIDGE_TOOLS, SPECIES as BRIDGE_SPECIES, type SpeciesDef, setSelectedSpecies, isInitialized } from '../bridge';

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
  Shovel: { icon: 'S', description: 'Dig / remove' },
  Seed:   { icon: 'P', description: 'Plant a seed' },
  Water:  { icon: 'W', description: 'Pour water' },
  Soil:   { icon: 'D', description: 'Place soil' },
  Stone:  { icon: 'R', description: 'Place stone' },
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
  };

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
      btn.innerHTML = `<span class="tool-icon">${tool.icon}</span><span class="tool-label">${tool.name}</span>`;
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
    const scoreEl = this.container.querySelector('#score-number');
    if (scoreEl) scoreEl.textContent = score.toLocaleString();
    const plantsEl = this.container.querySelector('#stat-plants');
    if (plantsEl) plantsEl.textContent = stats.plants.toLocaleString();
    const faunaEl = this.container.querySelector('#stat-fauna');
    if (faunaEl) faunaEl.textContent = String(stats.fauna);
    const speciesEl = this.container.querySelector('#stat-species');
    if (speciesEl) speciesEl.textContent = String(stats.species);
  }

  private renderStatus(): void {
    const tickState = this.state.autoTick ? 'ON' : 'OFF';
    const stats = this.state.gardenStats;
    const statsStr = stats
      ? ` | 🌿 ${stats.plants} | 🦋 ${stats.fauna} | Species: ${stats.species}`
      : '';
    this.statusEl.textContent = `Tick: ${this.state.tickCount} | Auto: ${tickState} [Space]${statsStr}`;
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
  </div>
  <div id="hud-status"></div>
  <div id="hud-help">Drag: orbit | Scroll: zoom | 1-5: tools | Z/C: species | Q: x-ray | V: overlay</div>
  <button id="tick-toggle" title="Toggle auto-tick [Space]">Tick</button>
  <button id="screenshot-btn" title="Capture screenshot [F2]">Snap</button>
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
  font-size: 18px;
  font-weight: bold;
  line-height: 1;
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

/* --- Status (top-left) --- */
#hud-status {
  position: absolute;
  top: 12px;
  left: 12px;
  font-size: 12px;
  color: rgba(200, 180, 140, 0.7);
  pointer-events: none;
}

/* --- Help (top-right) --- */
#hud-help {
  position: absolute;
  top: 12px;
  right: 12px;
  font-size: 11px;
  color: rgba(200, 180, 140, 0.4);
  pointer-events: none;
}

/* --- Tick toggle (top-left, below status) --- */
#tick-toggle {
  position: absolute;
  top: 34px;
  left: 12px;
  padding: 4px 10px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  background: rgba(20, 18, 15, 0.7);
  color: #b8a88a;
  cursor: pointer;
  font-size: 11px;
  font-family: inherit;
  pointer-events: auto;
  transition: all 0.12s ease;
}
#tick-toggle:hover {
  background: rgba(255, 255, 255, 0.12);
  color: #e8d8b8;
}

/* --- Screenshot button (top-left, below tick toggle) --- */
#screenshot-btn {
  position: absolute;
  top: 58px;
  left: 12px;
  padding: 4px 10px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  background: rgba(20, 18, 15, 0.7);
  color: #b8a88a;
  cursor: pointer;
  font-size: 11px;
  font-family: inherit;
  pointer-events: auto;
  transition: all 0.12s ease;
}
#screenshot-btn:hover {
  background: rgba(255, 255, 255, 0.12);
  color: #e8d8b8;
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
`;
