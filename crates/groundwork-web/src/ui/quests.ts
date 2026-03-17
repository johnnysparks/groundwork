/**
 * Tutorial quest system — gentle onboarding progression.
 *
 * Quest lines introduce one concept at a time with breathing room between each.
 * The player starts in an empty meadow with a pond and gnome — no UI visible.
 * Each quest line unlocks exactly one new tool or system.
 *
 * Quest Line 0: Meet Your Gnome — tap gnome, camera follows, orbit around
 * Quest Line 1: Start Your Garden — sow small (zone seeds), inspect a cell
 * Quest Line 2: See Below the Surface — x-ray mode, irrigation lens
 * Quest Line 3: Shape the Water — dig channels, watch first bloom + bee
 * Quest Line 4+: WIP (future quest lines)
 *
 * See decisions/2026-03-17T18:00:00_reduce_progression_intensity.md
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
  | 'tapGnome'
  | 'orbitCamera'
  | 'sowSmall'
  | 'inspectCell'
  | 'openXray'
  | 'irrigationLens'
  | 'digChannel'
  | 'firstBloom'
  | 'idleWatch'
  | 'wildSpecies';

interface QuestDef {
  id: QuestId;
  name: string;
  chapter: number;
  detail: string;
}

const QUEST_DEFS: QuestDef[] = [
  // Chapter 0: Meet Your Gnome — just you, the meadow, and a little friend
  {
    id: 'tapGnome',
    name: 'Say hello',
    chapter: 0,
    detail: 'Tap your garden gnome — he\'s waiting by the pond.',
  },
  {
    id: 'orbitCamera',
    name: 'Look around',
    chapter: 0,
    detail: 'Drag to orbit the camera. Take in the meadow — this is your garden.',
  },
  // Chapter 1: Start Your Garden — the seed tool appears, one action at a time
  {
    id: 'sowSmall',
    name: 'Sow a patch',
    chapter: 1,
    detail: 'Click near the pond to zone a small area. Your gnome will plant seeds — you don\'t know what will grow yet.',
  },
  {
    id: 'inspectCell',
    name: 'Inspect your seedling',
    chapter: 1,
    detail: 'Tap a planted cell to see what species grew. The inspect panel shows moisture and soil conditions.',
  },
  // Chapter 2: See Below the Surface — x-ray with lens picker
  {
    id: 'openXray',
    name: 'Go underground',
    chapter: 2,
    detail: 'Press Q to enter x-ray mode. See the roots growing beneath your garden.',
  },
  {
    id: 'irrigationLens',
    name: 'Check moisture',
    chapter: 2,
    detail: 'Open the lens picker and select "Irrigation." Blue is wet, red is dry — see where water flows.',
  },
  // Chapter 3: Shape the Water — irrigation and first ecological surprise
  {
    id: 'digChannel',
    name: 'Dig a channel',
    chapter: 3,
    detail: 'Use the shovel to dig toward the pond. Water flows through channels into dry soil.',
  },
  {
    id: 'firstBloom',
    name: 'First bloom',
    chapter: 3,
    detail: 'Let time pass. When your first flower blooms, something special arrives...',
  },
  // Chapter 4: Discovery — the garden grows on its own
  {
    id: 'idleWatch',
    name: 'Watch it grow',
    chapter: 4,
    detail: 'Stop planting and watch. Your garden is growing without you.',
  },
  {
    id: 'wildSpecies',
    name: 'An uninvited guest',
    chapter: 4,
    detail: 'A new species appeared somewhere you didn\'t plant. How did it get there?',
  },
];

const CHAPTER_NAMES = [
  'Meet Your Gnome',
  'Start Your Garden',
  'See Below the Surface',
  'Shape the Water',
  'The Garden Grows',
];

// ---------------------------------------------------------------------------
// Action tracking
// ---------------------------------------------------------------------------

interface ActionTracker {
  tappedGnome: boolean;
  orbited: boolean;
  panCount: number;
  plantedSeed: boolean;
  inspectedCell: boolean;
  openedXray: boolean;
  selectedIrrigationLens: boolean;
  usedShovel: boolean;
  /** Last voxel the player clicked on: [x, y, z] */
  lastClickedVoxel: [number, number, number] | null;
  speciesPlanted: Set<number>;
  depthChanged: boolean;
  placedWater: boolean;
  toggledAutoTick: boolean;
  steppedManually: boolean;
  /** Ticks since the player last used a tool */
  idleTicks: number;
  /** Species count when the player last planted */
  speciesAtLastPlant: number;
  /** True when a species appeared that the player didn't sow */
  wildSpeciesAppeared: boolean;
}

function createActionTracker(): ActionTracker {
  return {
    tappedGnome: false,
    orbited: false,
    panCount: 0,
    plantedSeed: false,
    inspectedCell: false,
    openedXray: false,
    selectedIrrigationLens: false,
    usedShovel: false,
    lastClickedVoxel: null,
    speciesPlanted: new Set(),
    depthChanged: false,
    placedWater: false,
    toggledAutoTick: false,
    steppedManually: false,
    idleTicks: 0,
    speciesAtLastPlant: 0,
    wildSpeciesAppeared: false,
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
  private _onChapterChange: ((chapter: number) => void) | null = null;

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

  /** Register a callback for chapter changes (used for progressive UI reveal) */
  onChapterChange(cb: (chapter: number) => void): void {
    this._onChapterChange = cb;
    // Fire immediately with current chapter so HUD can set initial state
    cb(this.currentChapter);
  }

  /** Reset all quest progress for a new game */
  reset(): void {
    this.currentChapter = 0;
    this.allComplete = false;
    for (const q of this.quests) q.completed = false;
    this.actions = createActionTracker();
    this.notification = null;
    this._onChapterChange?.(0);
    this.render();
  }

  // -------------------------------------------------------------------------
  // Action recording
  // -------------------------------------------------------------------------

  recordTapGnome(): void {
    this.actions.tappedGnome = true;
  }

  recordPan(): void {
    this.actions.panCount++;
  }

  recordDepthChange(): void {
    this.actions.depthChanged = true;
    this.actions.openedXray = true;
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
    this.actions.idleTicks = 0;
  }

  /** Update the species count from the garden (call when species count changes) */
  recordSpeciesCount(count: number): void {
    if (this.actions.speciesAtLastPlant === 0) {
      this.actions.speciesAtLastPlant = count;
    }
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

  recordInspectCell(): void {
    this.actions.inspectedCell = true;
  }

  recordSelectIrrigationLens(): void {
    this.actions.selectedIrrigationLens = true;
  }

  recordToolUse(tool: ToolCodeType, speciesIndex: number): void {
    this.actions.idleTicks = 0; // any tool use resets idle counter
    switch (tool) {
      case ToolCode.Shovel:
        this.recordUseShovel();
        // Digging IS irrigation — shovel completes the dig quest
        this.recordPlaceWater();
        break;
      case ToolCode.Seed:
        this.recordPlantSeed(speciesIndex);
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
   * @param speciesCount - number of unique species currently in the garden
   */
  check(grid: Uint8Array, speciesCount = 0): void {
    if (this.allComplete) return;

    // Increment idle ticks each check (called once per sim tick)
    this.actions.idleTicks++;

    // Check for bloom (any Leaf material = flower bloomed or tree leafed)
    const needsBloom = this.questActive('firstBloom');
    let hasBloom = false;
    if (needsBloom) {
      for (let i = 0; i < grid.length; i += VOXEL_BYTES) {
        if (grid[i] === Material.Leaf) { hasBloom = true; break; }
      }
    }

    // Fauna check for first bloom quest (bee arrives)
    let hasFauna = false;
    if (needsBloom) {
      try { hasFauna = getFaunaCount() > 0; } catch {}
    }

    // Wild species detection: species count grew while player was idle
    if (speciesCount > this.actions.speciesAtLastPlant && this.actions.idleTicks > 30) {
      this.actions.wildSpeciesAppeared = true;
    }

    const newlyCompleted: { index: number; name: string }[] = [];

    for (let i = 0; i < this.quests.length; i++) {
      const quest = this.quests[i];
      if (quest.completed || quest.chapter !== this.currentChapter) continue;

      let complete = false;
      switch (quest.id) {
        case 'tapGnome':
          complete = this.actions.tappedGnome;
          break;
        case 'orbitCamera':
          complete = this.actions.orbited || this.actions.panCount >= 4;
          break;
        case 'sowSmall':
          complete = this.actions.plantedSeed;
          break;
        case 'inspectCell':
          complete = this.actions.inspectedCell;
          break;
        case 'openXray':
          complete = this.actions.openedXray;
          break;
        case 'irrigationLens':
          complete = this.actions.selectedIrrigationLens;
          break;
        case 'digChannel':
          complete = this.actions.usedShovel;
          break;
        case 'firstBloom':
          // Complete when a flower blooms AND fauna arrives (the magical moment)
          complete = hasBloom && hasFauna;
          break;
        case 'idleWatch':
          // 100 idle ticks without planting (~8 seconds) — the player observes
          complete = this.actions.idleTicks >= 100;
          break;
        case 'wildSpecies':
          // A new species appeared while the player wasn't planting
          complete = this.actions.wildSpeciesAppeared;
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
        this._onChapterChange?.(this.currentChapter);
      } else {
        this.allComplete = true;
        this.showNotification('Your garden is coming alive!');
        this._onChapterChange?.(CHAPTER_NAMES.length);
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
      body.innerHTML = '<div class="quest-complete-msg">Your garden is alive.<br>Keep exploring — there\'s more to discover.</div>';
      body.style.display = 'block';
      return;
    }

    // Chapter progress
    const chapterQuests = this.quests.filter(q => q.chapter === this.currentChapter);
    const done = chapterQuests.filter(q => q.completed).length;
    const total = chapterQuests.length;

    // Header
    const headerLabel = this.panel.querySelector('#quest-chapter-label')!;
    headerLabel.textContent = `${CHAPTER_NAMES[this.currentChapter]}`;
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
