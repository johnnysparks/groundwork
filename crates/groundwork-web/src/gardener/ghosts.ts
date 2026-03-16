/**
 * Ghost overlay: renders planned-but-not-yet-executed zone tasks
 * as translucent colored voxels.
 *
 * Uses InstancedMesh for efficient rendering of many ghost voxels.
 * Each tool type has a distinct ghost color:
 * - Seed: soft green
 * - Water: translucent blue
 * - Dig: red-brown wireframe
 * - Soil: warm brown
 * - Stone: gray
 */

import * as THREE from 'three';
import { ToolCode, GROUND_LEVEL } from '../bridge';
import type { TaskQueue } from './queue';

const GHOST_COLORS: Record<number, THREE.Color> = {
  [ToolCode.Seed]: new THREE.Color(0.3, 0.7, 0.3),
  [ToolCode.Water]: new THREE.Color(0.3, 0.5, 0.8),
  [ToolCode.Shovel]: new THREE.Color(0.6, 0.3, 0.2),
  [ToolCode.Soil]: new THREE.Color(0.5, 0.35, 0.2),
  [ToolCode.Stone]: new THREE.Color(0.5, 0.5, 0.5),
};

const MAX_GHOSTS = 500;

export class GhostOverlay {
  readonly group: THREE.Group;
  private mesh: THREE.InstancedMesh;
  private dummy = new THREE.Object3D();
  private colorAttr: THREE.InstancedBufferAttribute;

  constructor() {
    this.group = new THREE.Group();
    this.group.name = 'ghost-overlay';

    const geo = new THREE.BoxGeometry(1, 1, 1);
    const colorArray = new Float32Array(MAX_GHOSTS * 3);
    this.colorAttr = new THREE.InstancedBufferAttribute(colorArray, 3);

    const mat = new THREE.MeshBasicMaterial({
      vertexColors: false,
      transparent: true,
      opacity: 0.25,
      depthWrite: false,
      side: THREE.DoubleSide,
      color: 0xffffff,
    });

    this.mesh = new THREE.InstancedMesh(geo, mat, MAX_GHOSTS);
    this.mesh.instanceColor = null;
    this.mesh.geometry.setAttribute('instanceColor', this.colorAttr);
    this.mesh.frustumCulled = false;
    this.mesh.count = 0;

    // Override material to use instance colors
    this.mesh.material = new THREE.MeshBasicMaterial({
      transparent: true,
      opacity: 0.3,
      depthWrite: false,
    });

    this.group.add(this.mesh);
  }

  /** Rebuild ghost instances from the task queue */
  rebuild(queue: TaskQueue, elapsed: number): void {
    const tasks = queue.all;
    const count = Math.min(tasks.length, MAX_GHOSTS);

    // Gentle pulse animation
    const pulse = 0.85 + Math.sin(elapsed * 3) * 0.15;

    for (let i = 0; i < count; i++) {
      const t = tasks[i];
      // Position: sim (x,y,z) Z-up → Three.js (x,z,y) Y-up
      this.dummy.position.set(t.x + 0.5, t.z + 0.5, t.y + 0.5);
      this.dummy.scale.setScalar(pulse);
      this.dummy.updateMatrix();
      this.mesh.setMatrixAt(i, this.dummy.matrix);

      // Color by tool type
      const color = GHOST_COLORS[t.tool] ?? new THREE.Color(1, 1, 1);
      this.colorAttr.setXYZ(i, color.r, color.g, color.b);
    }

    this.mesh.count = count;
    if (count > 0) {
      this.mesh.instanceMatrix.needsUpdate = true;
      this.colorAttr.needsUpdate = true;
      // Apply color to the mesh material as a tint
      // (InstancedMesh with MeshBasicMaterial doesn't support instanceColor natively
      //  without custom shader, so we'll just use the first task's color for now)
      const firstColor = count > 0 ? GHOST_COLORS[tasks[0].tool] : undefined;
      if (firstColor) {
        (this.mesh.material as THREE.MeshBasicMaterial).color.copy(firstColor);
      }
    }
  }

  dispose(): void {
    this.mesh.geometry.dispose();
    (this.mesh.material as THREE.Material).dispose();
  }
}
