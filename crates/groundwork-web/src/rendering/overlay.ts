/**
 * Data overlay renderer: heat map visualization of simulation channels.
 *
 * Renders a flat grid of colored quads just above the terrain surface,
 * showing water levels, nutrient density, light penetration, or species
 * distribution as color-coded heat maps.
 *
 * Toggle with 'V' key, cycle modes with Shift+V.
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GRID_Z, VOXEL_BYTES, Material, GROUND_LEVEL } from '../bridge';

/** Available overlay modes */
export enum OverlayMode {
  Off = 0,
  Water = 1,
  Light = 2,
  Nutrient = 3,
}

const MODE_NAMES = ['Off', 'Water', 'Light', 'Nutrient'];

/** Color ramps for each data channel */
function waterColor(value: number): [number, number, number] {
  const t = value / 255;
  // Dry = brown/transparent → wet = bright blue
  return [0.15 * (1 - t), 0.25 * (1 - t) + 0.6 * t, 0.1 * (1 - t) + 0.9 * t];
}

function lightColor(value: number): [number, number, number] {
  const t = value / 255;
  // Dark = deep purple → bright = warm yellow
  return [0.15 + 0.85 * t, 0.1 + 0.8 * t, 0.3 * (1 - t) + 0.2 * t];
}

function nutrientColor(value: number): [number, number, number] {
  const t = value / 255;
  // Low = pale → high = rich warm orange
  return [0.3 + 0.7 * t, 0.3 + 0.3 * t, 0.2 * (1 - t)];
}

export class DataOverlay {
  readonly group: THREE.Group;
  private mesh: THREE.Mesh | null = null;
  private _mode: OverlayMode = OverlayMode.Off;

  constructor() {
    this.group = new THREE.Group();
    this.group.name = 'data-overlay';
  }

  get mode(): OverlayMode { return this._mode; }
  get modeName(): string { return MODE_NAMES[this._mode]; }

  /** Toggle overlay on/off */
  toggle(): OverlayMode {
    if (this._mode === OverlayMode.Off) {
      this._mode = OverlayMode.Water;
    } else {
      this._mode = OverlayMode.Off;
    }
    if (this._mode === OverlayMode.Off) this.clear();
    return this._mode;
  }

  /** Cycle to next overlay mode */
  cycle(): OverlayMode {
    this._mode = ((this._mode + 1) % 4) as OverlayMode;
    if (this._mode === OverlayMode.Off) this.clear();
    return this._mode;
  }

  /** Set mode directly */
  setMode(mode: OverlayMode): void {
    this._mode = mode;
    if (this._mode === OverlayMode.Off) this.clear();
  }

  /** Clear the overlay mesh */
  private clear(): void {
    if (this.mesh) {
      this.group.remove(this.mesh);
      this.mesh.geometry.dispose();
      (this.mesh.material as THREE.Material).dispose();
      this.mesh = null;
    }
  }

  /**
   * Rebuild the overlay from the current grid data.
   * Scans the surface layer and colors each cell by the selected data channel.
   */
  rebuild(grid: Uint8Array): void {
    if (this._mode === OverlayMode.Off) {
      this.clear();
      return;
    }

    this.clear();

    // Scan surface: for each (x,y), find the top non-air voxel
    const positions: number[] = [];
    const colors: number[] = [];
    const indices: number[] = [];
    let vi = 0;

    for (let y = 0; y < GRID_Y; y++) {
      for (let x = 0; x < GRID_X; x++) {
        // Find surface Z (scan down from above ground)
        let surfZ = -1;
        let dataValue = 0;
        for (let z = GROUND_LEVEL + 10; z >= GROUND_LEVEL - 5; z--) {
          if (z < 0 || z >= GRID_Z) continue;
          const idx = (x + y * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
          const mat = grid[idx];
          if (mat !== Material.Air) {
            surfZ = z;
            // Read the appropriate data channel
            switch (this._mode) {
              case OverlayMode.Water:   dataValue = grid[idx + 1]; break;
              case OverlayMode.Light:   dataValue = grid[idx + 2]; break;
              case OverlayMode.Nutrient: dataValue = grid[idx + 3]; break;
            }
            break;
          }
        }

        if (surfZ < 0) continue;
        if (dataValue === 0) continue; // Skip empty cells for cleaner visual

        // Color from ramp
        let r: number, g: number, b: number;
        switch (this._mode) {
          case OverlayMode.Water:   [r, g, b] = waterColor(dataValue); break;
          case OverlayMode.Light:   [r, g, b] = lightColor(dataValue); break;
          case OverlayMode.Nutrient: [r, g, b] = nutrientColor(dataValue); break;
          default: r = g = b = 0.5;
        }

        // Quad slightly above surface (Three.js Y-up: sim Z → Three Y)
        const qy = surfZ + 0.6; // float above surface
        const baseIdx = vi;

        // 4 corners of the cell
        positions.push(x, qy, y);       colors.push(r, g, b);
        positions.push(x + 1, qy, y);   colors.push(r, g, b);
        positions.push(x + 1, qy, y + 1); colors.push(r, g, b);
        positions.push(x, qy, y + 1);   colors.push(r, g, b);

        indices.push(baseIdx, baseIdx + 1, baseIdx + 2);
        indices.push(baseIdx, baseIdx + 2, baseIdx + 3);
        vi += 4;
      }
    }

    if (positions.length === 0) return;

    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
    geometry.setAttribute('color', new THREE.Float32BufferAttribute(colors, 3));
    geometry.setIndex(indices);
    geometry.computeVertexNormals();

    const material = new THREE.MeshBasicMaterial({
      vertexColors: true,
      transparent: true,
      opacity: 0.45,
      depthWrite: false,
      side: THREE.DoubleSide,
    });

    this.mesh = new THREE.Mesh(geometry, material);
    this.mesh.name = 'data-overlay-mesh';
    this.group.add(this.mesh);
  }

  dispose(): void {
    this.clear();
  }
}
