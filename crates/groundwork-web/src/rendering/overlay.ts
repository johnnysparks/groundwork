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
  /** Irrigation lens: 3D moisture heatmap for x-ray mode.
   *  Ocean blue (wet) → transparent (mid) → brick red (dry).
   *  See decisions/2026-03-17T18:00:00_reduce_progression_intensity.md */
  Irrigation = 4,
}

const MODE_NAMES = ['Off', 'Water', 'Light', 'Nutrient', 'Irrigation'];

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

/**
 * Irrigation lens color ramp — designed for the x-ray lens picker.
 * Returns [r, g, b, alpha] where alpha controls per-cell opacity.
 *
 * - Zero moisture: 50% opaque brick red (0.75, 0.28, 0.18)
 * - Full water/pond: 50% opaque ocean blue (0.15, 0.35, 0.65)
 * - Mid-range: increasingly transparent, neutral gray tone
 */
function irrigationColor(value: number): [number, number, number, number] {
  const t = value / 255;
  // Color: brick red → neutral → ocean blue
  const r = 0.75 * (1 - t) + 0.15 * t;
  const g = 0.28 * (1 - t) + 0.35 * t;
  const b = 0.18 * (1 - t) + 0.65 * t;
  // Alpha: high at extremes (0.55), visible in mid-range (0.25)
  // U-shaped curve with raised floor so the heatmap reads as a continuous
  // gradient, not just highlights at dry/wet extremes.
  const dist = Math.abs(t - 0.5) * 2; // 0 at midpoint, 1 at extremes
  const alpha = 0.25 + 0.30 * dist * dist;
  return [r, g, b, alpha];
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
    this._mode = ((this._mode + 1) % 5) as OverlayMode;
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
   * For Irrigation mode, scans all non-air voxels in the ground range (3D heatmap).
   */
  rebuild(grid: Uint8Array): void {
    if (this._mode === OverlayMode.Off) {
      this.clear();
      return;
    }

    this.clear();

    // Irrigation mode: 3D volumetric heatmap (shows all ground-level cells)
    if (this._mode === OverlayMode.Irrigation) {
      this.rebuildIrrigation(grid);
      return;
    }

    // Standard surface overlay for Water/Light/Nutrient modes
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
            switch (this._mode) {
              case OverlayMode.Water:   dataValue = grid[idx + 1]; break;
              case OverlayMode.Light:   dataValue = grid[idx + 2]; break;
              case OverlayMode.Nutrient: dataValue = grid[idx + 3]; break;
            }
            break;
          }
        }

        if (surfZ < 0) continue;
        if (dataValue === 0) continue;

        let r: number, g: number, b: number;
        switch (this._mode) {
          case OverlayMode.Water:   [r, g, b] = waterColor(dataValue); break;
          case OverlayMode.Light:   [r, g, b] = lightColor(dataValue); break;
          case OverlayMode.Nutrient: [r, g, b] = nutrientColor(dataValue); break;
          default: r = g = b = 0.5;
        }

        const qy = surfZ + 0.6;
        const baseIdx = vi;
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

  /**
   * Irrigation lens: 3D moisture heatmap visible through x-ray.
   * Scans all non-air voxels in the ground range.
   * Blue (wet) → transparent (mid) → red (dry).
   * Water/pond cells are ocean blue at 50% opacity.
   */
  private rebuildIrrigation(grid: Uint8Array): void {
    const positions: number[] = [];
    const colors: number[] = [];
    const alphas: number[] = [];
    const indices: number[] = [];
    let vi = 0;

    // Scan ground range: from a few below ground to surface
    const zMin = Math.max(0, GROUND_LEVEL - 15);
    const zMax = Math.min(GRID_Z, GROUND_LEVEL + 5);

    for (let z = zMin; z < zMax; z++) {
      for (let y = 0; y < GRID_Y; y++) {
        for (let x = 0; x < GRID_X; x++) {
          const idx = (x + y * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
          const mat = grid[idx];
          if (mat === Material.Air) continue;

          const waterVal = grid[idx + 1];

          // Water/pond cells get special treatment: solid ocean blue
          const isWaterCell = mat === Material.Water;
          let r: number, g: number, b: number, a: number;

          if (isWaterCell) {
            r = 0.12; g = 0.30; b = 0.60;
            a = 0.50;
          } else {
            [r, g, b, a] = irrigationColor(waterVal);
          }

          // Skip very transparent cells for performance
          if (a < 0.05) continue;

          const qy = z + 0.5; // center of voxel
          const baseIdx = vi;

          // Top face quad
          positions.push(x, qy, y);       colors.push(r, g, b); alphas.push(a);
          positions.push(x + 1, qy, y);   colors.push(r, g, b); alphas.push(a);
          positions.push(x + 1, qy, y + 1); colors.push(r, g, b); alphas.push(a);
          positions.push(x, qy, y + 1);   colors.push(r, g, b); alphas.push(a);
          indices.push(baseIdx, baseIdx + 1, baseIdx + 2);
          indices.push(baseIdx, baseIdx + 2, baseIdx + 3);
          vi += 4;
        }
      }
    }

    if (positions.length === 0) return;

    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
    geometry.setAttribute('color', new THREE.Float32BufferAttribute(colors, 3));
    geometry.setIndex(indices);
    geometry.computeVertexNormals();

    // Use custom shader material for per-vertex alpha.
    // depthTest disabled so the heatmap renders on top of root meshes
    // (roots are opaque + depth-writing, which would otherwise hide the overlay).
    const material = new THREE.ShaderMaterial({
      vertexShader: `
        attribute float alpha;
        varying vec3 vColor;
        varying float vAlpha;
        void main() {
          vColor = color;
          vAlpha = alpha;
          gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
        }
      `,
      fragmentShader: `
        varying vec3 vColor;
        varying float vAlpha;
        void main() {
          gl_FragColor = vec4(vColor, vAlpha);
        }
      `,
      transparent: true,
      depthWrite: false,
      depthTest: false,
      side: THREE.DoubleSide,
      vertexColors: true,
    });

    geometry.setAttribute('alpha', new THREE.Float32BufferAttribute(alphas, 1));

    this.mesh = new THREE.Mesh(geometry, material);
    this.mesh.name = 'data-overlay-mesh';
    this.mesh.renderOrder = 100; // Render after roots so heatmap is visible
    this.group.add(this.mesh);
  }

  dispose(): void {
    this.clear();
  }
}
