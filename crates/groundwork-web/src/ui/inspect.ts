/**
 * Inspect panel — shows information about a tapped voxel cell.
 *
 * Appears when the player taps/clicks a planted cell. Shows species name,
 * moisture level, light level, and nutrient level in a warm, readable panel.
 *
 * Part of the gentle onboarding: the player learns to observe before optimizing.
 * See decisions/2026-03-17T18:00:00_reduce_progression_intensity.md
 */

import {
  GRID_X, GRID_Y, GRID_Z, VOXEL_BYTES,
  Material, SPECIES, isInitialized,
} from '../bridge';

export interface InspectData {
  x: number;
  y: number;
  z: number;
  material: number;
  materialName: string;
  speciesId: number;
  speciesName: string;
  water: number;
  light: number;
  nutrient: number;
  waterLabel: string;
  lightLabel: string;
  nutrientLabel: string;
  /** Plant condition: 'thriving' | 'healthy' | 'stressed' | 'dying' | null (non-plant) */
  condition: string | null;
  /** Short diagnosis of why the plant is stressed, if applicable */
  stressHint: string | null;
}

/** Find the nearest competing species within a search radius.
 *  Scans for trunk/root/leaf voxels of a different species. */
function findNearbyCompetitor(grid: Uint8Array, x: number, y: number, z: number, mySpeciesId: number): string | null {
  const radius = 8;
  const plantMats = new Set([Material.Trunk, Material.Root, Material.Leaf, Material.Branch]);
  let bestDist = Infinity;
  let bestSpeciesId = -1;

  for (let dz = -radius; dz <= radius; dz++) {
    for (let dy = -radius; dy <= radius; dy++) {
      for (let dx = -radius; dx <= radius; dx++) {
        if (dx === 0 && dy === 0 && dz === 0) continue;
        const nx = x + dx, ny = y + dy, nz = z + dz;
        if (nx < 0 || nx >= GRID_X || ny < 0 || ny >= GRID_Y || nz < 0 || nz >= GRID_Z) continue;
        const nIdx = (nx + ny * GRID_X + nz * GRID_X * GRID_Y) * VOXEL_BYTES;
        const nMat = grid[nIdx];
        if (!plantMats.has(nMat)) continue;
        const nSpecies = grid[nIdx + 3];
        if (nSpecies === mySpeciesId) continue;
        const dist = dx * dx + dy * dy + dz * dz;
        if (dist < bestDist) {
          bestDist = dist;
          bestSpeciesId = nSpecies;
        }
      }
    }
  }

  if (bestSpeciesId < 0) return null;
  const sp = SPECIES.find(s => s.index === bestSpeciesId);
  return sp ? sp.name : null;
}

/** Check if groundcover species (moss/grass/clover) exist within a small radius. */
function hasNearbyGroundcover(grid: Uint8Array, x: number, y: number, z: number): boolean {
  const radius = 6;
  const groundcoverIds = new Set([9, 10, 11]); // moss, grass, clover
  const plantMats = new Set([Material.Seed, Material.Leaf, Material.Root]);
  for (let dy = -radius; dy <= radius; dy++) {
    for (let dx = -radius; dx <= radius; dx++) {
      // Check at ground level only (groundcover is near surface)
      const nx = x + dx, ny = y + dy, nz = z;
      if (nx < 0 || nx >= GRID_X || ny < 0 || ny >= GRID_Y || nz < 0 || nz >= GRID_Z) continue;
      const nIdx = (nx + ny * GRID_X + nz * GRID_X * GRID_Y) * VOXEL_BYTES;
      if (plantMats.has(grid[nIdx]) && groundcoverIds.has(grid[nIdx + 3])) return true;
    }
  }
  return false;
}

/** Human-readable label for a 0-255 value */
function levelLabel(value: number): string {
  if (value === 0) return 'none';
  if (value < 40) return 'very low';
  if (value < 80) return 'low';
  if (value < 130) return 'medium';
  if (value < 190) return 'high';
  return 'very high';
}

/** Material enum value to display name */
function materialDisplayName(mat: number): string {
  for (const [key, val] of Object.entries(Material)) {
    if (val === mat && typeof key === 'string') return key.toLowerCase();
  }
  return 'unknown';
}

/** Read voxel data at a grid position */
export function readVoxelAt(grid: Uint8Array, x: number, y: number, z: number): InspectData | null {
  if (x < 0 || x >= GRID_X || y < 0 || y >= GRID_Y || z < 0 || z >= GRID_Z) return null;
  const idx = (x + y * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
  if (idx < 0 || idx + 3 >= grid.length) return null;

  const mat = grid[idx];
  const water = grid[idx + 1];
  const light = grid[idx + 2];
  const nutrient = grid[idx + 3];

  // Species is encoded in the nutrient byte for plant materials
  const speciesId = nutrient;
  const species = SPECIES.find(s => s.index === speciesId);

  // Only show species for plant materials
  const isPlant = mat === Material.Seed || mat === Material.Trunk || mat === Material.Leaf ||
                  mat === Material.Branch || mat === Material.Root;
  const speciesName = isPlant && species ? species.name : '';

  // Plant health diagnosis for leaves/trunk
  let condition: string | null = null;
  let stressHint: string | null = null;
  if (isPlant && mat !== Material.Seed) {
    // For leaf voxels: sim writes tree.health * 255 into water_level (255=healthy, 0=dead)
    // For trunk/root voxels: use light as proxy for shade stress
    const isLeaf = mat === Material.Leaf;
    const health = isLeaf ? water / 255 : null;

    // Assess conditions from environment
    const lowLight = light < 30;
    const lowWater = water < 30 && !isLeaf; // leaf water_level is health, not moisture

    if (health !== null) {
      if (health > 0.8) condition = 'thriving';
      else if (health > 0.5) condition = 'healthy';
      else if (health > 0.2) condition = 'stressed';
      else condition = 'dying';
    } else if (mat === Material.Trunk || mat === Material.Branch) {
      // Trunk/branch: infer from light
      condition = lowLight ? 'shaded' : 'healthy';
    } else if (mat === Material.Root) {
      condition = lowWater ? 'dry' : 'healthy';
    }

    // Stress diagnosis hints — identify competitor when possible
    const competitor = (condition === 'stressed' || condition === 'dying' || condition === 'shaded')
      ? findNearbyCompetitor(grid, x, y, z, speciesId) : null;

    if (condition === 'stressed' || condition === 'dying') {
      if (lowLight && competitor) {
        stressHint = `Shaded by ${competitor} — not enough light reaching the canopy`;
      } else if (lowLight) {
        stressHint = 'Not enough light — shaded by a taller tree?';
      } else if (competitor) {
        stressHint = `Competing with ${competitor} for water — roots are overlapping`;
      } else {
        stressHint = 'Struggling for water — dig channels from the spring';
      }
    } else if (condition === 'shaded') {
      stressHint = competitor
        ? `In the shadow of a nearby ${competitor}`
        : 'In the shadow of a taller tree';
    } else if (condition === 'dry') {
      stressHint = 'Roots need moisture — dig irrigation channels nearby';
    } else if (condition === 'thriving') {
      // Positive hints — teach why the plant succeeds
      const hasGroundcover = hasNearbyGroundcover(grid, x, y, z);
      if (hasGroundcover) {
        stressHint = 'Enriched soil — groundcover fixes nitrogen nearby';
      } else if (light > 180) {
        stressHint = 'Strong sunlight and good water access';
      } else {
        stressHint = 'Good balance of water, light, and soil';
      }
    }
  }

  return {
    x, y, z,
    material: mat,
    materialName: materialDisplayName(mat),
    speciesId: isPlant ? speciesId : -1,
    speciesName,
    water,
    light,
    nutrient,
    waterLabel: levelLabel(water),
    lightLabel: levelLabel(light),
    nutrientLabel: levelLabel(nutrient),
    condition,
    stressHint,
  };
}

/**
 * Inspect panel overlay — shows voxel info on tap.
 */
export class InspectPanel {
  private el: HTMLElement;
  private _visible = false;
  private _onInspect: (() => void) | null = null;

  constructor() {
    this.el = document.createElement('div');
    this.el.id = 'inspect-panel';
    this.el.innerHTML = '';
    document.body.appendChild(this.el);

    const style = document.createElement('style');
    style.textContent = INSPECT_CSS;
    document.head.appendChild(style);

    // Click outside to dismiss
    document.addEventListener('pointerdown', (e) => {
      if (this._visible && !this.el.contains(e.target as Node)) {
        this.hide();
      }
    });
  }

  get visible(): boolean { return this._visible; }

  /** Register callback for when a cell is inspected (for quest tracking) */
  onInspect(cb: () => void): void {
    this._onInspect = cb;
  }

  /** Show inspect data for a voxel */
  show(data: InspectData): void {
    const moistureBar = this.barHTML(data.water, '#4488cc', '#cc6644');
    const lightBar = this.barHTML(data.light, '#ddcc44', '#554488');

    let title = data.materialName;
    if (data.speciesName) {
      title = data.speciesName;
    }

    // Condition badge color
    const conditionColor = data.condition === 'thriving' ? '#6a6' :
      data.condition === 'healthy' ? '#8a8' :
      data.condition === 'stressed' ? '#ca6' :
      data.condition === 'dying' ? '#c66' :
      data.condition === 'shaded' ? '#88a' :
      data.condition === 'dry' ? '#ca8' : '';

    const conditionHTML = data.condition
      ? `<div class="inspect-condition" style="color:${conditionColor}">${data.condition}</div>`
      : '';

    const hintHTML = data.stressHint
      ? `<div class="inspect-hint">${data.stressHint}</div>`
      : '';

    this.el.innerHTML = `
      <div class="inspect-title">${title}</div>
      ${conditionHTML}
      <div class="inspect-row">
        <span class="inspect-label">Moisture</span>
        <span class="inspect-value">${data.waterLabel}</span>
        ${moistureBar}
      </div>
      <div class="inspect-row">
        <span class="inspect-label">Light</span>
        <span class="inspect-value">${data.lightLabel}</span>
        ${lightBar}
      </div>
      ${hintHTML}
      <div class="inspect-coord">(${data.x}, ${data.y}, z=${data.z})</div>
    `;
    this.el.classList.add('visible');
    this._visible = true;
    this._onInspect?.();
  }

  hide(): void {
    this.el.classList.remove('visible');
    this._visible = false;
  }

  private barHTML(value: number, highColor: string, lowColor: string): string {
    const pct = Math.round((value / 255) * 100);
    const color = value > 128 ? highColor : value > 40 ? '#888' : lowColor;
    return `<div class="inspect-bar"><div class="inspect-bar-fill" style="width:${pct}%;background:${color}"></div></div>`;
  }
}

const INSPECT_CSS = `
#inspect-panel {
  position: absolute;
  bottom: 100px;
  right: 16px;
  width: 200px;
  background: rgba(20, 18, 15, 0.92);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  padding: 12px 14px;
  backdrop-filter: blur(8px);
  font-family: system-ui, -apple-system, sans-serif;
  color: #d4c8a8;
  z-index: 15;
  pointer-events: auto;
  opacity: 0;
  visibility: hidden;
  transform: translateY(8px);
  transition: opacity 0.25s ease, visibility 0.25s ease, transform 0.25s ease;
}
#inspect-panel.visible {
  opacity: 1;
  visibility: visible;
  transform: translateY(0);
}

.inspect-title {
  font-size: 14px;
  font-weight: 600;
  color: #e8d8b8;
  margin-bottom: 8px;
  text-transform: capitalize;
}

.inspect-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
  margin-bottom: 6px;
}

.inspect-label {
  font-size: 11px;
  color: rgba(200, 180, 140, 0.6);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  width: 60px;
}

.inspect-value {
  font-size: 12px;
  color: #d4c8a8;
  flex: 1;
  text-align: right;
}

.inspect-bar {
  width: 100%;
  height: 4px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 2px;
  overflow: hidden;
}

.inspect-bar-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.3s ease;
}

.inspect-condition {
  font-size: 12px;
  font-weight: 500;
  text-transform: capitalize;
  margin-bottom: 6px;
}

.inspect-hint {
  font-size: 11px;
  color: rgba(220, 190, 140, 0.7);
  font-style: italic;
  margin: 4px 0 2px;
  line-height: 1.3;
}

.inspect-coord {
  font-size: 10px;
  color: rgba(200, 180, 140, 0.3);
  margin-top: 6px;
  text-align: right;
}

@media (max-width: 768px) {
  #inspect-panel {
    bottom: 80px;
    right: 8px;
    width: 180px;
    padding: 10px 12px;
  }
}
`;
