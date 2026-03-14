/**
 * Seed renderer — tiny, semi-transparent mound sprites.
 *
 * Seeds are rendered as small instanced hemispheres instead of full voxel
 * cubes, giving them a delicate, organic look. Each seed is a low golden
 * mound sitting on the soil surface with partial opacity.
 *
 * Uses InstancedMesh with a squished SphereGeometry for the mound shape.
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GRID_Z, VOXEL_BYTES } from '../bridge';
import { isSeed } from '../mesher/greedy';

/** Maximum number of seed instances */
const MAX_SEEDS = 10_000;

/** Mound scale relative to a voxel (small bump, not a full cube) */
const MOUND_RADIUS = 0.3;
const MOUND_HEIGHT = 0.2;

/** Seed color palette — earthy golden tones */
const SEED_COLORS = [
  new THREE.Color(0.55, 0.50, 0.25), // golden
  new THREE.Color(0.50, 0.42, 0.22), // darker brown
  new THREE.Color(0.60, 0.52, 0.28), // lighter gold
  new THREE.Color(0.48, 0.44, 0.20), // olive-brown
];

export class SeedRenderer {
  readonly group: THREE.Group;

  private mesh: THREE.InstancedMesh;
  private material: THREE.MeshLambertMaterial;
  private colorAttr: THREE.InstancedBufferAttribute;
  private instanceCount = 0;
  private dummy = new THREE.Object3D();

  constructor() {
    this.group = new THREE.Group();
    this.group.name = 'seeds';

    // Hemisphere geometry — upper half of a sphere, scaled to a low mound
    const geo = new THREE.SphereGeometry(
      MOUND_RADIUS,  // radius
      8,             // width segments (low poly is fine for tiny mounds)
      4,             // height segments
      0,             // phiStart
      Math.PI * 2,   // phiLength (full circle)
      0,             // thetaStart (top)
      Math.PI / 2,   // thetaLength (upper hemisphere only)
    );
    // Squish vertically for a low mound shape
    geo.scale(1, 1, MOUND_HEIGHT / MOUND_RADIUS);

    this.material = new THREE.MeshLambertMaterial({
      transparent: true,
      opacity: 0.75,
      depthWrite: false,
    });

    // Per-instance colors
    const colorArray = new Float32Array(MAX_SEEDS * 3);
    this.colorAttr = new THREE.InstancedBufferAttribute(colorArray, 3);

    this.mesh = new THREE.InstancedMesh(geo, this.material, MAX_SEEDS);
    this.mesh.instanceColor = null;
    this.mesh.geometry.setAttribute('instanceColor', this.colorAttr);
    this.mesh.frustumCulled = false;
    this.mesh.count = 0;

    // Use vertex colors via instanceColor attribute
    this.material.vertexColors = false;

    this.group.add(this.mesh);
  }

  /**
   * Scan the voxel grid and build seed instances.
   * Call after grid changes (tick, tool placement).
   */
  rebuild(grid: Uint8Array): void {
    let count = 0;

    for (let z = 0; z < GRID_Z && count < MAX_SEEDS; z++) {
      for (let y = 0; y < GRID_Y && count < MAX_SEEDS; y++) {
        for (let x = 0; x < GRID_X && count < MAX_SEEDS; x++) {
          const idx = (x + y * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
          const mat = grid[idx];

          if (!isSeed(mat)) continue;

          // Position at bottom of voxel (sitting on soil surface)
          this.dummy.position.set(x + 0.5, y + 0.5, z);
          this.dummy.scale.setScalar(1);
          this.dummy.rotation.set(0, 0, 0);
          this.dummy.updateMatrix();
          this.mesh.setMatrixAt(count, this.dummy.matrix);

          // Pick color with variation based on position hash
          const hash = (x * 73856093 ^ y * 19349663 ^ z * 83492791) & 0xffff;
          const colorIdx = hash % SEED_COLORS.length;
          const baseColor = SEED_COLORS[colorIdx];
          const brightness = 0.85 + ((hash >> 4) & 0xf) / 15.0 * 0.3;

          this.colorAttr.setXYZ(
            count,
            baseColor.r * brightness,
            baseColor.g * brightness,
            baseColor.b * brightness,
          );

          count++;
        }
      }
    }

    this.instanceCount = count;
    this.mesh.count = count;
    this.mesh.instanceMatrix.needsUpdate = true;
    this.colorAttr.needsUpdate = true;

    // Wire up per-instance colors on first rebuild
    if (this.mesh.instanceColor === null) {
      this.mesh.instanceColor = this.colorAttr;
    }
  }

  /** Current number of active seed sprites */
  get count(): number {
    return this.instanceCount;
  }

  dispose(): void {
    this.mesh.geometry.dispose();
    this.material.dispose();
  }
}
