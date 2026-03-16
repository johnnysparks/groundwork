/**
 * Fauna renderer: charming 3D models for ecological creatures.
 *
 * Renders pollinators (bees, butterflies), birds, worms, and beetles as
 * small but characterful 3D models built from soft primitives. Each model
 * feels like a collectible figurine placed in the diorama — part of the
 * same world as the voxel garden.
 *
 * Uses a pool of pre-built model Groups. Positions are read from the
 * WASM fauna export buffer each frame. Wing animations are driven by
 * rotating named child meshes.
 */

import * as THREE from 'three';
import {
  GROUND_LEVEL,
  FaunaType,
  FaunaState,
  getFaunaCount,
  getFaunaView,
  readFauna,
} from '../bridge';
import { buildFaunaModel } from '../models/fauna';

/** Maximum fauna instances (matches Rust MAX_FAUNA) */
const MAX_FAUNA = 128;

/** Fauna model sizes in voxels — deliberately oversized for readability.
 *  Sized so creatures are visible at default camera zoom (full 80×80 garden). */
const FAUNA_SIZES: Record<number, number> = {
  [FaunaType.Bee]: 2.5,
  [FaunaType.Butterfly]: 3.5,
  [FaunaType.Bird]: 5.0,
  [FaunaType.Worm]: 1.8,
  [FaunaType.Beetle]: 2.0,
};

/** Warm glow halo colors per fauna type — draws the eye to creatures. */
const FAUNA_GLOW_COLORS: Record<number, number> = {
  [FaunaType.Bee]: 0xffdd44,      // honey gold
  [FaunaType.Butterfly]: 0xffaa55, // warm amber
  [FaunaType.Bird]: 0xddcc88,     // soft tan
  [FaunaType.Worm]: 0xcc9966,     // earthy
  [FaunaType.Beetle]: 0x99aa66,   // moss green
};

/** Shared radial gradient texture for fauna glow halos. */
let _glowTexture: THREE.Texture | null = null;
function getGlowTexture(): THREE.Texture {
  if (_glowTexture) return _glowTexture;
  const size = 64;
  const canvas = document.createElement('canvas');
  canvas.width = size;
  canvas.height = size;
  const ctx = canvas.getContext('2d')!;
  const grad = ctx.createRadialGradient(size / 2, size / 2, 0, size / 2, size / 2, size / 2);
  grad.addColorStop(0, 'rgba(255,255,255,0.6)');
  grad.addColorStop(0.4, 'rgba(255,255,255,0.2)');
  grad.addColorStop(1, 'rgba(255,255,255,0)');
  ctx.fillStyle = grad;
  ctx.fillRect(0, 0, size, size);
  _glowTexture = new THREE.CanvasTexture(canvas);
  return _glowTexture;
}

/** Create a soft radial glow halo for a fauna creature.
 *  Uses THREE.Sprite with a radial gradient texture for a gentle
 *  circular glow instead of a hard rectangular box. */
function createFaunaGlow(faunaType: number): THREE.Sprite {
  const color = FAUNA_GLOW_COLORS[faunaType] ?? 0xffdd44;
  const mat = new THREE.SpriteMaterial({
    map: getGlowTexture(),
    color,
    transparent: true,
    opacity: 0.5,
    depthWrite: false,
    blending: THREE.AdditiveBlending,
  });
  const glow = new THREE.Sprite(mat);
  glow.name = 'glow';
  glow.scale.set(2.2, 2.2, 1);
  return glow;
}

/** Pre-built model pool entry. */
interface FaunaSlot {
  model: THREE.Group;
  type: number;
  active: boolean;
}

export class FaunaRenderer {
  readonly group: THREE.Group;

  private pool: FaunaSlot[] = [];
  private activeCount = 0;

  constructor() {
    this.group = new THREE.Group();
    this.group.name = 'fauna';

    // Pre-build model pool — one Group per possible fauna instance.
    // Each slot starts as a bee; we swap model types as needed.
    for (let i = 0; i < MAX_FAUNA; i++) {
      const model = buildFaunaModel(FaunaType.Bee, i);
      model.add(createFaunaGlow(FaunaType.Bee));
      model.visible = false;
      this.group.add(model);
      this.pool.push({ model, type: FaunaType.Bee, active: false });
    }
  }

  /**
   * Update fauna positions and animations from the WASM bridge data.
   * Call each frame after sim tick.
   */
  update(elapsedTime: number): void {
    const count = getFaunaCount();
    const view = getFaunaView();

    if (!view || count === 0) {
      // Hide all
      for (const slot of this.pool) {
        slot.model.visible = false;
        slot.active = false;
      }
      this.activeCount = 0;
      return;
    }

    const actualCount = Math.min(count, MAX_FAUNA);

    for (let i = 0; i < actualCount; i++) {
      const f = readFauna(view, i);
      const slot = this.pool[i];

      // Swap model if type changed
      if (slot.type !== f.type) {
        this.group.remove(slot.model);
        const newModel = buildFaunaModel(f.type, i);
        newModel.add(createFaunaGlow(f.type));
        slot.model = newModel;
        slot.type = f.type;
        this.group.add(newModel);
      }

      // Position: sim (x,y,z) Z-up → Three.js (x,z,y) Y-up
      slot.model.position.set(f.x, f.z, f.y);

      // Scale
      const size = FAUNA_SIZES[f.type] ?? 1.0;
      slot.model.scale.setScalar(size);

      // Face movement direction (basic: face toward camera-ish)
      // For now, gentle rotation based on position for variety
      slot.model.rotation.y = Math.atan2(f.y - 40, f.x - 40) + Math.PI;

      // Wing animations
      this.animateWings(slot.model, f.type, f.state, elapsedTime, i);

      // Leaving state: fade via scale shrink
      if (f.state === FaunaState.Leaving) {
        slot.model.scale.multiplyScalar(0.6);
      }

      // Acting state: slight bob
      if (f.state === FaunaState.Acting) {
        slot.model.position.y += Math.sin(elapsedTime * 8 + i) * 0.15;
      }

      slot.model.visible = true;
      slot.active = true;
    }

    // Hide unused slots
    for (let i = actualCount; i < MAX_FAUNA; i++) {
      this.pool[i].model.visible = false;
      this.pool[i].active = false;
    }

    this.activeCount = actualCount;
  }

  /** Animate wing meshes for flying creatures. */
  private animateWings(
    model: THREE.Group,
    type: number,
    state: number,
    time: number,
    index: number,
  ): void {
    const offset = index * 2.37; // per-creature phase offset

    if (type === FaunaType.Bee) {
      // Rapid wing flutter
      const flutter = Math.sin((time + offset) * 30) * 0.8;
      const wingL = model.getObjectByName('wing_l');
      const wingR = model.getObjectByName('wing_r');
      if (wingL) wingL.rotation.z = 0.5 + flutter;
      if (wingR) wingR.rotation.z = -0.5 - flutter;
      // Gentle body bob
      model.position.y += Math.sin((time + offset) * 5) * 0.2;

    } else if (type === FaunaType.Butterfly) {
      // Gentle wing flap
      const flap = Math.sin((time + offset) * 4) * 0.7;
      const names = ['wing_ul', 'wing_ur', 'wing_ll', 'wing_lr'];
      for (const name of names) {
        const w = model.getObjectByName(name);
        if (!w) continue;
        const sign = name.includes('_u') ? 1 : 0.7;
        const side = name.endsWith('l') ? 1 : -1;
        w.rotation.z = side * flap * sign;
      }
      // Float gently
      model.position.y += Math.sin((time + offset) * 2) * 0.3;

    } else if (type === FaunaType.Bird) {
      // Slow wing soar
      const soar = Math.sin((time + offset) * 2) * 0.4;
      const wingL = model.getObjectByName('wing_l');
      const wingR = model.getObjectByName('wing_r');
      if (wingL) wingL.rotation.z = 0.3 + soar;
      if (wingR) wingR.rotation.z = -0.3 - soar;
      // Gentle float
      model.position.y += Math.sin((time + offset) * 1.5) * 0.15;
    }
    // Worms and beetles don't have wing animations
  }

  /** Current active fauna count */
  get count(): number {
    return this.activeCount;
  }

  dispose(): void {
    for (const slot of this.pool) {
      slot.model.traverse((obj) => {
        if (obj instanceof THREE.Mesh) {
          obj.geometry.dispose();
          if (obj.material instanceof THREE.Material) obj.material.dispose();
        }
      });
    }
  }
}
