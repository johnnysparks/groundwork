/**
 * Tutorial quest system — ported from groundwork-tui/src/quest.rs.
 *
 * 20 quests across 9 chapters guide the player through the core mechanics.
 * Quests complete based on player actions (panning, tool use) and world state
 * (materials in the grid). Chapter advances when all quests in it are done.
 *
 * Adapted for the web renderer: 3D orbit replaces "Switch to 3D", inspect
 * quests are simplified (no inspect panel yet), range tool quest removed.
 */

import {
  GRID_X, GRID_Y, GRID_Z, GROUND_LEVEL, VOXEL_BYTES,
  Material, ToolCode, type ToolCodeType,
} from '../bridge';
import { SPECIES } from '../bridge';

// ---------------------------------------------------------------------------
// Quest definitions
// ---------------------------------------------------------------------------

export type QuestId =
  | 'panAround'
  | 'changeDepth'
  | 'orbitCamera'
  | 'findTheSpring'
  | 'placeWater'
  | 'observeWetSoil'
  | 'plantFirstSeed'
  | 'watchItGrow'
  | 'changeSpecies'
  | 'toggleAutoTick'
  | 'stepManually'
  | 'viewUnderground'
  | 'findRoots'
  | 'plantThreeSpecies'
  | 'plantAllTypes'
  | 'useShovel'
  | 'growATree';

interface QuestDef {
  id: QuestId;
  name: string;
  chapter: number;
  detail: string;
}

const QUEST_DEFS: QuestDef[] = [
  // Chapter 0: Getting Your Bearings
  {
    id: 'panAround',
    name: 'Move the camera',
    chapter: 0,
    detail: 'Your garden is a 60m \u00d7 60m plot of living earth. Use WASD or arrow keys to slide your view across the landscape. Everything you see is a half-meter voxel you can shape.',
  },
  {
    id: 'changeDepth',
    name: 'Toggle x-ray view',
    chapter: 0,
    detail: 'Your garden has layers. Press Q to toggle x-ray mode — soil and stone become transparent so you can see roots, water channels, and underground structures.',
  },
  {
    id: 'orbitCamera',
    name: 'Orbit the camera',
    chapter: 0,
    detail: 'Click and drag on the terrain to orbit around your garden. Scroll to zoom in and out. Press R to reset the view.',
  },
  // Chapter 1: Water is Life
  {
    id: 'findTheSpring',
    name: 'Find the water spring',
    chapter: 1,
    detail: 'Every garden has a natural water spring deep underground. Water bubbles up, flows outward, and soaks into soil. Click on water to find it \u2014 it\u2019s the source of life for everything you\u2019ll grow.',
  },
  {
    id: 'placeWater',
    name: 'Use the watering can',
    chapter: 1,
    detail: 'Press 3 to select the watering can, then click to pour. Water obeys gravity \u2014 it falls through air and pools on solid ground. Plants need water nearby to grow.',
  },
  {
    id: 'observeWetSoil',
    name: 'Find wet soil',
    chapter: 1,
    detail: 'When water touches soil, it soaks in. Wet soil retains moisture that roots absorb. The wetter the soil, the faster seeds germinate. Click on dark soil near water.',
  },
  // Chapter 2: First Planting
  {
    id: 'plantFirstSeed',
    name: 'Plant a seed',
    chapter: 2,
    detail: 'Press 2 to select the seed bag, then click on soil to plant. Seeds fall through air and land on solid ground. They need both water and light to germinate.',
  },
  {
    id: 'watchItGrow',
    name: 'Watch a seed sprout',
    chapter: 2,
    detail: 'Seeds germinate into seedlings, then saplings, then mature plants. Enable auto-tick with Space and watch your seed become a living plant.',
  },
  {
    id: 'changeSpecies',
    name: 'Try a different species',
    chapter: 2,
    detail: 'Press Q and E to cycle through 12 species: trees, shrubs, flowers, and groundcover. Or click the species panel when the seed tool is active. Each has unique height, water needs, and growth rate.',
  },
  // Chapter 3: Time Control
  {
    id: 'toggleAutoTick',
    name: 'Toggle auto-tick',
    chapter: 3,
    detail: 'Press Space to start the simulation clock. Time advances and your garden evolves. Press Space again to pause. Each tick, water flows, light spreads, and plants grow.',
  },
  {
    id: 'stepManually',
    name: 'Step time manually',
    chapter: 3,
    detail: 'Press T to advance exactly one tick. Watch water seep, soil moisten, and seeds accumulate nutrients tick by tick.',
  },
  // Chapter 4: Going Underground
  {
    id: 'viewUnderground',
    name: 'Go below surface',
    chapter: 4,
    detail: 'Press Q to toggle x-ray mode. Soil and stone become transparent, revealing underground layers, water channels, and \u2014 once plants grow \u2014 roots spreading through the earth.',
  },
  {
    id: 'findRoots',
    name: 'Find plant roots',
    chapter: 4,
    detail: 'Plant roots spread underground seeking water. They absorb moisture from soil and deliver nutrients upward. Healthy root networks mean healthy plants above. Click on a root to complete this quest.',
  },
  // Chapter 5: Biodiversity
  {
    id: 'plantThreeSpecies',
    name: 'Plant 3 species',
    chapter: 5,
    detail: 'Different species thrive in different conditions. Trees grow tall but slowly. Shrubs fill gaps. Flowers bloom fast. A diverse garden is a resilient garden.',
  },
  {
    id: 'plantAllTypes',
    name: 'Plant all plant types',
    chapter: 5,
    detail: 'Plant a tree, shrub, flower, and groundcover. Each type fills a different ecological niche. Check the species panel \u2014 the four groups are Trees, Shrubs, Flowers, and Groundcover.',
  },
  // Chapter 6: Shaping the Land
  {
    id: 'useShovel',
    name: 'Dig with the shovel',
    chapter: 6,
    detail: 'Press 1 to select the shovel, then click to dig. The shovel removes anything: soil, stone, seeds, even roots. Carve channels, clear space, or reshape terrain.',
  },
  // Chapter 7: Living Ecosystem
  {
    id: 'growATree',
    name: 'Grow a tree',
    chapter: 7,
    detail: 'A mature tree is the crown jewel of your garden. It disperses seeds that become new plants \u2014 the beginning of a self-sustaining ecosystem. Your garden is alive.',
  },
];

const CHAPTER_NAMES = [
  'Getting Your Bearings',
  'Water is Life',
  'First Planting',
  'Time Control',
  'Going Underground',
  'Biodiversity',
  'Shaping the Land',
  'Living Ecosystem',
];

// ---------------------------------------------------------------------------
// Action tracking
// ---------------------------------------------------------------------------

interface ActionTracker {
  panCount: number;
  depthChanged: boolean;
  orbited: boolean;
  placedWater: boolean;
  plantedSeed: boolean;
  cycledSpecies: boolean;
  toggledAutoTick: boolean;
  steppedManually: boolean;
  usedShovel: boolean;
  speciesPlanted: Set<number>;
  /** Last voxel the player clicked on: [x, y, z] */
  lastClickedVoxel: [number, number, number] | null;
}

function createActionTracker(): ActionTracker {
  return {
    panCount: 0,
    depthChanged: false,
    orbited: false,
    placedWater: false,
    plantedSeed: false,
    cycledSpecies: false,
    toggledAutoTick: false,
    steppedManually: false,
    usedShovel: false,
    speciesPlanted: new Set(),
    lastClickedVoxel: null,
  };
}

// ---------------------------------------------------------------------------
// Quest state
// ---------------------------------------------------------------------------

export interface Quest {
  id: QuestId;
  name: string;
  chapter: number;
  detail: string;
  completed: boolean;
}

export class QuestLog {
  quests: Quest[];
  currentChapter = 0;
  allComplete = false;
  private actions: ActionTracker;
  private notification: { message: string; framesLeft: number } | null = null;
  private expanded = true;
  private panel: HTMLElement;
  private notificationEl: HTMLElement;

  constructor() {
    this.quests = QUEST_DEFS.map(def => ({
      id: def.id,
      name: def.name,
      chapter: def.chapter,
      detail: def.detail,
      completed: false,
    }));
    this.actions = createActionTracker();

    // Build the DOM panel
    this.panel = document.createElement('div');
    this.panel.id = 'quest-panel';
    this.panel.innerHTML = QUEST_PANEL_HTML;
    document.body.appendChild(this.panel);

    // Notification toast (top center)
    this.notificationEl = document.createElement('div');
    this.notificationEl.id = 'quest-notification';
    document.body.appendChild(this.notificationEl);

    // Inject styles
    const style = document.createElement('style');
    style.textContent = QUEST_CSS;
    document.head.appendChild(style);

    // Toggle expand/collapse
    const header = this.panel.querySelector('#quest-header')!;
    header.addEventListener('click', (e) => {
      e.stopPropagation();
      this.expanded = !this.expanded;
      this.render();
    });

    // Toggle with M key
    document.addEventListener('keydown', (e) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
      if (e.key.toLowerCase() === 'm') {
        this.expanded = !this.expanded;
        this.render();
      }
    });

    this.render();
  }

  // -------------------------------------------------------------------------
  // Action recording
  // -------------------------------------------------------------------------

  recordPan(): void {
    this.actions.panCount++;
  }

  recordDepthChange(): void {
    this.actions.depthChanged = true;
  }

  recordOrbit(): void {
    this.actions.orbited = true;
  }

  recordPlaceWater(): void {
    this.actions.placedWater = true;
  }

  recordPlantSeed(speciesIndex: number): void {
    this.actions.plantedSeed = true;
    this.actions.speciesPlanted.add(speciesIndex);
  }

  recordCycleSpecies(): void {
    this.actions.cycledSpecies = true;
  }

  recordToggleAutoTick(): void {
    this.actions.toggledAutoTick = true;
  }

  recordStepManually(): void {
    this.actions.steppedManually = true;
  }

  recordUseShovel(): void {
    this.actions.usedShovel = true;
  }

  recordToolUse(tool: ToolCodeType, speciesIndex: number): void {
    switch (tool) {
      case ToolCode.Shovel:
        this.recordUseShovel();
        break;
      case ToolCode.Seed:
        this.recordPlantSeed(speciesIndex);
        break;
      case ToolCode.Water:
        this.recordPlaceWater();
        break;
    }
  }

  recordClick(x: number, y: number, z: number): void {
    this.actions.lastClickedVoxel = [x, y, z];
  }

  // -------------------------------------------------------------------------
  // Completion checks
  // -------------------------------------------------------------------------

  /**
   * Check quest completion against current grid state and recorded actions.
   * Call after each tick and after each tool placement.
   */
  check(grid: Uint8Array): void {
    if (this.allComplete) return;

    // Get focus voxel properties (from last click)
    let focusMat: number | null = null;
    let focusWater = 0;
    let focusZ = GROUND_LEVEL; // default to surface
    if (this.actions.lastClickedVoxel) {
      const [fx, fy, fz] = this.actions.lastClickedVoxel;
      focusZ = fz;
      if (fx >= 0 && fx < GRID_X && fy >= 0 && fy < GRID_Y && fz >= 0 && fz < GRID_Z) {
        const idx = (fx + fy * GRID_X + fz * GRID_X * GRID_Y) * VOXEL_BYTES;
        focusMat = grid[idx];
        focusWater = grid[idx + 1];
      }
    }

    // Lazily scan grid only for quests that need it
    const needsTrunk = this.questActive('watchItGrow');
    let hasTrunk = false;
    if (needsTrunk) {
      for (let i = 0; i < grid.length; i += VOXEL_BYTES) {
        if (grid[i] === Material.Trunk) { hasTrunk = true; break; }
      }
    }

    const needsTree = this.questActive('growATree');
    let treeGrown = false;
    if (needsTree) {
      let leaves = 0;
      let branches = 0;
      for (let i = 0; i < grid.length; i += VOXEL_BYTES) {
        if (grid[i] === Material.Leaf) leaves++;
        if (grid[i] === Material.Branch) branches++;
        if (leaves >= 50 && branches >= 10) { treeGrown = true; break; }
      }
    }

    const newlyCompleted: { index: number; name: string }[] = [];

    for (let i = 0; i < this.quests.length; i++) {
      const quest = this.quests[i];
      if (quest.completed || quest.chapter !== this.currentChapter) continue;

      let complete = false;
      switch (quest.id) {
        case 'panAround':
          complete = this.actions.panCount >= 4;
          break;
        case 'changeDepth':
          complete = this.actions.depthChanged;
          break;
        case 'orbitCamera':
          complete = this.actions.orbited;
          break;
        case 'findTheSpring':
          complete = focusMat === Material.Water;
          break;
        case 'placeWater':
          complete = this.actions.placedWater;
          break;
        case 'observeWetSoil':
          complete = focusMat === Material.Soil && focusWater > 50;
          break;
        case 'plantFirstSeed':
          complete = this.actions.plantedSeed;
          break;
        case 'watchItGrow':
          complete = hasTrunk;
          break;
        case 'changeSpecies':
          complete = this.actions.cycledSpecies;
          break;
        case 'toggleAutoTick':
          complete = this.actions.toggledAutoTick;
          break;
        case 'stepManually':
          complete = this.actions.steppedManually;
          break;
        case 'viewUnderground':
          complete = focusZ < GROUND_LEVEL;
          break;
        case 'findRoots':
          complete = focusMat === Material.Root;
          break;
        case 'plantThreeSpecies':
          complete = this.actions.speciesPlanted.size >= 3;
          break;
        case 'plantAllTypes':
          complete = hasAllPlantTypes(this.actions.speciesPlanted);
          break;
        case 'useShovel':
          complete = this.actions.usedShovel;
          break;
        case 'growATree':
          complete = treeGrown;
          break;
      }

      if (complete) {
        newlyCompleted.push({ index: i, name: quest.name });
      }
    }

    // Apply completions
    for (const { index, name } of newlyCompleted) {
      this.quests[index].completed = true;
      this.showNotification(`Done: ${name}`);
    }

    if (newlyCompleted.length > 0) {
      this.tryAdvanceChapter();
      this.render();
    }
  }

  private questActive(id: QuestId): boolean {
    return this.quests.some(q => q.id === id && q.chapter === this.currentChapter && !q.completed);
  }

  private tryAdvanceChapter(): void {
    const chapterComplete = this.quests
      .filter(q => q.chapter === this.currentChapter)
      .every(q => q.completed);

    if (chapterComplete) {
      if (this.currentChapter < CHAPTER_NAMES.length - 1) {
        this.currentChapter++;
        this.showNotification(`Chapter: ${CHAPTER_NAMES[this.currentChapter]}`);
      } else {
        this.allComplete = true;
        this.showNotification('All missions complete!');
      }
    }
  }

  // -------------------------------------------------------------------------
  // Notification
  // -------------------------------------------------------------------------

  private showNotification(message: string): void {
    this.notification = { message, framesLeft: 180 }; // ~3 seconds at 60fps
    this.notificationEl.textContent = message;
    this.notificationEl.classList.add('visible');
  }

  /** Call once per frame to fade notifications. */
  tickNotification(): void {
    if (!this.notification) return;
    this.notification.framesLeft--;
    if (this.notification.framesLeft <= 0) {
      this.notification = null;
      this.notificationEl.classList.remove('visible');
    }
  }

  // -------------------------------------------------------------------------
  // DOM rendering
  // -------------------------------------------------------------------------

  render(): void {
    const body = this.panel.querySelector('#quest-body')! as HTMLElement;
    const toggle = this.panel.querySelector('#quest-toggle')!;

    if (this.allComplete) {
      toggle.textContent = '\u2714';
      body.innerHTML = '<div class="quest-complete-msg">All missions complete!<br>Your garden is alive.</div>';
      body.style.display = 'block';
      return;
    }

    // Chapter progress
    const chapterQuests = this.quests.filter(q => q.chapter === this.currentChapter);
    const done = chapterQuests.filter(q => q.completed).length;
    const total = chapterQuests.length;

    // Header
    const headerLabel = this.panel.querySelector('#quest-chapter-label')!;
    headerLabel.textContent = `Ch.${this.currentChapter + 1}: ${CHAPTER_NAMES[this.currentChapter]}`;
    const headerProgress = this.panel.querySelector('#quest-progress')!;
    headerProgress.textContent = `${done}/${total}`;
    toggle.textContent = this.expanded ? '\u25BC' : '\u25B6';

    if (!this.expanded) {
      body.style.display = 'none';
      return;
    }

    body.style.display = 'block';

    // Build quest list
    let html = '';
    for (const quest of chapterQuests) {
      const check = quest.completed ? '\u2714' : '\u25CB';
      const cls = quest.completed ? 'quest-item completed' : 'quest-item';
      html += `<div class="${cls}">
        <span class="quest-check">${check}</span>
        <div class="quest-text">
          <div class="quest-name">${quest.name}</div>
          <div class="quest-detail">${quest.detail}</div>
        </div>
      </div>`;
    }
    body.innerHTML = html;
  }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/**
 * Check if the player has planted at least one of each plant type.
 * Species indices 0-3 = Tree, 4-6 = Shrub, 7-8 = Flower, 9-11 = Groundcover
 */
function hasAllPlantTypes(planted: Set<number>): boolean {
  const types = new Set<string>();
  for (const idx of planted) {
    if (idx < SPECIES.length) {
      types.add(SPECIES[idx].type);
    }
  }
  return types.has('Tree') && types.has('Shrub') && types.has('Flower') && types.has('Ground');
}

// ---------------------------------------------------------------------------
// HTML & CSS
// ---------------------------------------------------------------------------

const QUEST_PANEL_HTML = `
  <div id="quest-header">
    <span id="quest-toggle">\u25BC</span>
    <span id="quest-chapter-label"></span>
    <span id="quest-progress"></span>
    <span id="quest-key-hint">[M]</span>
  </div>
  <div id="quest-body"></div>
`;

const QUEST_CSS = `
/* --- Quest panel (top-left, below status) --- */
#quest-panel {
  position: absolute;
  top: 60px;
  left: 12px;
  width: 280px;
  background: rgba(20, 18, 15, 0.88);
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(8px);
  font-family: system-ui, -apple-system, sans-serif;
  color: #d4c8a8;
  z-index: 10;
  pointer-events: auto;
  overflow: hidden;
}

#quest-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  cursor: pointer;
  user-select: none;
  transition: background 0.12s ease;
}
#quest-header:hover {
  background: rgba(255, 255, 255, 0.05);
}

#quest-toggle {
  font-size: 10px;
  color: rgba(200, 180, 140, 0.5);
  width: 12px;
}

#quest-chapter-label {
  font-size: 13px;
  font-weight: 600;
  color: #e8d8b8;
  flex: 1;
}

#quest-progress {
  font-size: 12px;
  color: rgba(180, 210, 120, 0.8);
  font-weight: 600;
}

#quest-key-hint {
  font-size: 10px;
  color: rgba(200, 180, 140, 0.3);
}

#quest-body {
  padding: 0 12px 10px;
}

.quest-item {
  display: flex;
  gap: 8px;
  padding: 6px 0;
  border-top: 1px solid rgba(255, 255, 255, 0.04);
}
.quest-item:first-child {
  border-top: none;
}

.quest-check {
  flex-shrink: 0;
  width: 18px;
  font-size: 13px;
  line-height: 1.4;
  color: rgba(200, 180, 140, 0.4);
}
.quest-item.completed .quest-check {
  color: rgba(140, 190, 80, 0.9);
}

.quest-text {
  flex: 1;
  min-width: 0;
}

.quest-name {
  font-size: 12px;
  font-weight: 500;
  color: #e0d4b8;
  line-height: 1.4;
}
.quest-item.completed .quest-name {
  color: rgba(200, 180, 140, 0.4);
  text-decoration: line-through;
}

.quest-detail {
  font-size: 11px;
  line-height: 1.5;
  color: rgba(200, 180, 140, 0.5);
  margin-top: 2px;
}
.quest-item.completed .quest-detail {
  display: none;
}

.quest-complete-msg {
  padding: 8px 0;
  font-size: 13px;
  font-weight: 600;
  color: rgba(140, 190, 80, 0.9);
  line-height: 1.5;
  text-align: center;
}

/* --- Quest notification toast (top center) --- */
#quest-notification {
  position: absolute;
  top: 16px;
  left: 50%;
  transform: translateX(-50%) translateY(-8px);
  padding: 8px 20px;
  background: rgba(20, 18, 15, 0.92);
  border: 1px solid rgba(140, 190, 80, 0.4);
  border-radius: 8px;
  backdrop-filter: blur(8px);
  font-family: system-ui, -apple-system, sans-serif;
  font-size: 14px;
  font-weight: 600;
  color: rgba(180, 220, 100, 0.95);
  z-index: 20;
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.3s ease, transform 0.3s ease;
}
#quest-notification.visible {
  opacity: 1;
  transform: translateX(-50%) translateY(0);
}
`;
