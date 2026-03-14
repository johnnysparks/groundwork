/**
 * HUD overlay: tool palette, species picker, and status display.
 *
 * Built as HTML/CSS overlay on top of the Three.js canvas for crisp text
 * and standard UI behavior. Communicates tool/species selection via callbacks.
 */

import { ToolCode, type ToolCodeType } from '../bridge';

/** Tool definitions for the palette */
export interface ToolDef {
  code: ToolCodeType;
  name: string;
  icon: string;
  key: string;
  description: string;
}

export const TOOLS: ToolDef[] = [
  { code: ToolCode.Shovel, name: 'Shovel', icon: 'S', key: '1', description: 'Dig / remove' },
  { code: ToolCode.Seed,   name: 'Seed',   icon: 'P', key: '2', description: 'Plant a seed' },
  { code: ToolCode.Water,  name: 'Water',  icon: 'W', key: '3', description: 'Pour water' },
  { code: ToolCode.Soil,   name: 'Soil',   icon: 'D', key: '4', description: 'Place soil' },
  { code: ToolCode.Stone,  name: 'Stone',  icon: 'R', key: '5', description: 'Place stone' },
];

/** Species definitions (matches Rust's PlantType species) */
export interface SpeciesDef {
  id: string;
  name: string;
  type: string;
}

export const SPECIES: SpeciesDef[] = [
  // Trees
  { id: 'oak',        name: 'Oak',        type: 'Tree' },
  { id: 'birch',      name: 'Birch',      type: 'Tree' },
  { id: 'willow',     name: 'Willow',     type: 'Tree' },
  { id: 'pine',       name: 'Pine',       type: 'Tree' },
  // Shrubs
  { id: 'fern',       name: 'Fern',       type: 'Shrub' },
  { id: 'berry-bush', name: 'Berry Bush', type: 'Shrub' },
  { id: 'holly',      name: 'Holly',      type: 'Shrub' },
  // Flowers
  { id: 'wildflower', name: 'Wildflower', type: 'Flower' },
  { id: 'daisy',      name: 'Daisy',      type: 'Flower' },
  // Groundcover
  { id: 'moss',       name: 'Moss',       type: 'Ground' },
  { id: 'grass',      name: 'Grass',      type: 'Ground' },
  { id: 'clover',     name: 'Clover',     type: 'Ground' },
];

export interface HudState {
  activeTool: ToolCodeType;
  activeSpecies: string;
  autoTick: boolean;
  tickCount: number;
}

type HudChangeCallback = (state: HudState) => void;

/**
 * Creates and manages the HUD overlay DOM elements.
 */
export class Hud {
  readonly state: HudState = {
    activeTool: ToolCode.Seed,
    activeSpecies: 'oak',
    autoTick: false,
    tickCount: 0,
  };

  private container: HTMLElement;
  private toolButtons: HTMLElement[] = [];
  private speciesButtons: HTMLElement[] = [];
  private speciesPanel: HTMLElement;
  private statusEl: HTMLElement;
  private onChange: HudChangeCallback | null = null;

  constructor() {
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
    for (const tool of TOOLS) {
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

    // Wire up species picker
    this.speciesPanel = this.container.querySelector('#species-panel')! as HTMLElement;
    const speciesList = this.speciesPanel.querySelector('#species-list')!;
    let currentType = '';
    for (const sp of SPECIES) {
      if (sp.type !== currentType) {
        currentType = sp.type;
        const header = document.createElement('div');
        header.className = 'species-group-header';
        header.textContent = sp.type;
        speciesList.appendChild(header);
      }
      const btn = document.createElement('button');
      btn.className = 'species-btn';
      btn.dataset.species = sp.id;
      btn.textContent = sp.name;
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        this.selectSpecies(sp.id);
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

  /** Select a species by ID */
  selectSpecies(id: string): void {
    this.state.activeSpecies = id;
    this.render();
    this.notify();
  }

  /** Cycle to next species */
  cycleSpecies(direction: 1 | -1): void {
    const idx = SPECIES.findIndex(s => s.id === this.state.activeSpecies);
    const next = (idx + direction + SPECIES.length) % SPECIES.length;
    this.selectSpecies(SPECIES[next].id);
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
      btn.classList.toggle('active', btn.dataset.species === this.state.activeSpecies);
    }

    this.renderStatus();
  }

  private renderStatus(): void {
    const tickState = this.state.autoTick ? 'ON' : 'OFF';
    this.statusEl.textContent = `Tick: ${this.state.tickCount} | Auto: ${tickState} [Space]`;
  }
}

// --- HTML Template ---

const HUD_HTML = `
  <div id="tool-bar"></div>
  <div id="species-panel">
    <div id="species-list"></div>
  </div>
  <div id="hud-status"></div>
  <div id="hud-help">Drag: orbit | Scroll: zoom | 1-5: tools | Q/E: species</div>
  <button id="tick-toggle" title="Toggle auto-tick [Space]">Tick</button>
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
`;
