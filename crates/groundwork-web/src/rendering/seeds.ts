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

/** Mound scale relative to a voxel — visible at default zoom */
const MOUND_RADIUS = 0.45;
const MOUND_HEIGHT = 0.3;

/** Seed color palette — green-golden to suggest emerging life */
const SEED_COLORS = [
  new THREE.Color(0.45, 0.55, 0.25), // green-gold
  new THREE.Color(0.40, 0.50, 0.22), // olive
  new THREE.Color(0.50, 0.55, 0.20), // warm green
  new THREE.Color(0.35, 0.48, 0.25), // deep green-brown
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
      opacity: 0.85,
      depthWrite: false,
      emissive: new THREE.Color(0.15, 0.25, 0.05),
      emissiveIntensity: 0.5,
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

          // Position at bottom of voxel (sim Y↔Z swap: sim Z=up → Three.js Y=up)
          this.dummy.position.set(x + 0.5, z, y + 0.5);
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

  /** Pulse emissive glow — seeds "breathe" with anticipation of germination */
  update(elapsed: number): void {
    if (this.instanceCount === 0) return;
    const pulse = 0.3 + Math.sin(elapsed * 1.8) * 0.2 + Math.sin(elapsed * 0.7) * 0.1;
    this.material.emissiveIntensity = pulse;
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
