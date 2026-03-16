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
  getFaunaCount,
} from '../bridge';
import { SPECIES } from '../bridge';

// ---------------------------------------------------------------------------
// Quest definitions
// ---------------------------------------------------------------------------

export type QuestId =
  | 'panAround'
  | 'changeDepth'
  | 'orbitCamera'
  | 'placeWater'
  | 'plantFirstSeed'
  | 'watchItGrow'
  | 'changeSpecies'
  | 'toggleAutoTick'
  | 'plantThreeSpecies'
  | 'plantAllTypes'
  | 'growATree';

interface QuestDef {
  id: QuestId;
  name: string;
  chapter: number;
  detail: string;
}

const QUEST_DEFS: QuestDef[] = [
  // Chapter 0: Welcome
  {
    id: 'panAround',
    name: 'Look around',
    chapter: 0,
    detail: 'Drag to orbit, pinch to zoom. On desktop: WASD to pan. This is your garden — a small glen waiting to come alive.',
  },
  {
    id: 'orbitCamera',
    name: 'Find the spring',
    chapter: 0,
    detail: 'There\'s a water spring at the center. Orbit the camera until you spot the blue water column.',
  },
  // Chapter 1: Plant a Zone
  {
    id: 'placeWater',
    name: 'Irrigate',
    chapter: 1,
    detail: 'Press 3 for the watering can, then click near the spring. Water spreads to moisten nearby soil — plants need it.',
  },
  {
    id: 'plantFirstSeed',
    name: 'Zone a meadow',
    chapter: 1,
    detail: 'Press 2 for seeds, pick a species (Z/C to cycle), then click near water. You\'re planting a zone, not a single seed.',
  },
  {
    id: 'watchItGrow',
    name: 'Watch it grow',
    chapter: 1,
    detail: 'The garden is already ticking. Watch your zone sprout — seedlings appear, then trunks, then canopy.',
  },
  // Chapter 2: Diversify
  {
    id: 'changeSpecies',
    name: 'Add another species',
    chapter: 2,
    detail: 'Press Z or C to pick a different species and zone it nearby. Different species = higher score. Try flowers near trees.',
  },
  {
    id: 'plantThreeSpecies',
    name: 'Grow 3 species',
    chapter: 2,
    detail: 'Zone three different species near water. Diversity attracts fauna and unlocks ecological interactions.',
  },
  // Chapter 3: Discover
  {
    id: 'changeDepth',
    name: 'Go underground',
    chapter: 3,
    detail: 'Press Q for x-ray mode. See root networks spreading underground — each species has a different color.',
  },
  {
    id: 'toggleAutoTick',
    name: 'Attract fauna',
    chapter: 3,
    detail: 'Plant flowers near trees. When enough leaves grow, pollinators appear — watch for glowing golden dots near the canopy.',
  },
  // Chapter 4: Ecology
  {
    id: 'plantAllTypes',
    name: 'Plant all 4 types',
    chapter: 4,
    detail: 'Zone a tree, a shrub, a flower, and a groundcover. Each fills a different ecological niche. Clover near oak boosts growth!',
  },
  {
    id: 'growATree',
    name: 'Reach score 1000',
    chapter: 4,
    detail: 'Your garden score rewards biodiversity and ecological life. More species + more fauna = higher score. Check the panel top-right.',
  },
];

const CHAPTER_NAMES = [
  'Welcome',
  'Plant a Zone',
  'Diversify',
  'Discover',
  'Ecology',
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

    // Fauna check for "attract fauna" quest
    let hasFauna = false;
    if (this.questActive('toggleAutoTick')) {
      try { hasFauna = getFaunaCount() > 0; } catch {}
    }

    const newlyCompleted: { index: number; name: string }[] = [];

    for (let i = 0; i < this.quests.length; i++) {
      const quest = this.quests[i];
      if (quest.completed || quest.chapter !== this.currentChapter) continue;

      let complete = false;
      switch (quest.id) {
        case 'panAround':
          // Accept either WASD pan OR touch orbit — mobile players orbit instead of pan
          complete = this.actions.panCount >= 4 || this.actions.orbited;
          break;
        case 'changeDepth':
          complete = this.actions.depthChanged;
          break;
        case 'orbitCamera':
          complete = this.actions.orbited;
          break;
        case 'placeWater':
          complete = this.actions.placedWater;
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
          // "Attract fauna" — check if fauna count > 0
          complete = hasFauna;
          break;
        case 'plantThreeSpecies':
          complete = this.actions.speciesPlanted.size >= 3;
          break;
        case 'plantAllTypes':
          complete = hasAllPlantTypes(this.actions.speciesPlanted);
          break;
        case 'growATree':
          // "Reach score 1000" — check plant count as proxy
          complete = treeGrown && this.actions.speciesPlanted.size >= 4;
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
  top: 50px;
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

/* --- Mobile responsive --- */
@media (max-width: 768px) {
  #quest-panel {
    top: 42px;
    left: 6px;
    width: min(260px, calc(100vw - 120px));
  }
  #quest-header { padding: 6px 10px; }
  #quest-body { padding: 0 10px 8px; }
  .quest-name { font-size: 12px; }
  .quest-detail { font-size: 10px; }
}

@media (max-width: 480px) {
  #quest-panel { width: min(220px, calc(100vw - 100px)); }
}
`;
