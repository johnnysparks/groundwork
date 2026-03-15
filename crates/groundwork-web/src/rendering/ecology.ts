/**
 * Ecological interaction indicator particles.
 *
 * Makes cause-and-effect chains VISIBLE in the garden:
 * - Pollination trails: golden pollen drifting between flowers (near fauna)
 * - Nutrient flow: warm orange particles rising from enriched soil (near worms)
 * - Water absorption: blue particles sinking into roots (near water+root contact)
 * - Decomposition: dark amber particles near dead wood being processed
 *
 * These are purely visual — they don't affect simulation. They read from
 * the voxel grid + fauna data to decide where to emit.
 */

import * as THREE from 'three';
import {
  GRID_X, GRID_Y, GRID_Z, VOXEL_BYTES, Material, GROUND_LEVEL,
  getFaunaCount, getFaunaView, readFauna, FaunaType,
} from '../bridge';

/** Maximum ecology particles */
const MAX_PARTICLES = 3000;

/** Particle vertex shader */
const ECO_VERT = /* glsl */ `
  attribute float aLife;
  attribute float aMaxLife;
  attribute vec3 aColor;

  varying float vLife;
  varying vec3 vColor;

  void main() {
    vLife = aLife / aMaxLife;
    vColor = aColor;

    vec4 mvPos = modelViewMatrix * vec4(position, 1.0);
    // Size: small, gentle glow
    float sizeCurve = sin(vLife * 3.14159) * 0.7 + 0.3;
    gl_PointSize = sizeCurve * 4.0;
    gl_Position = projectionMatrix * mvPos;
  }
`;

/** Particle fragment shader */
const ECO_FRAG = /* glsl */ `
  varying float vLife;
  varying vec3 vColor;

  void main() {
    float dist = length(gl_PointCoord - 0.5) * 2.0;
    if (dist > 1.0) discard;
    // Soft glow with life-based fade
    float alpha = (1.0 - dist * dist) * vLife * 0.8;
    gl_FragColor = vec4(vColor, alpha);
  }
`;

/** Interaction type colors */
const INTERACTION_COLORS = {
  pollination: [
    new THREE.Color(0.95, 0.85, 0.30),  // golden pollen
    new THREE.Color(0.90, 0.80, 0.40),  // warm yellow
  ],
  nutrient: [
    new THREE.Color(0.70, 0.45, 0.15),  // warm amber
    new THREE.Color(0.60, 0.50, 0.20),  // earthy gold
  ],
  waterAbsorb: [
    new THREE.Color(0.30, 0.60, 0.85),  // light blue
    new THREE.Color(0.20, 0.50, 0.75),  // deeper blue
  ],
  decomposition: [
    new THREE.Color(0.50, 0.35, 0.15),  // dark amber
    new THREE.Color(0.45, 0.30, 0.10),  // earthy brown
  ],
};

interface EcoParticle {
  alive: boolean;
  life: number;
  maxLife: number;
  x: number; y: number; z: number;
  vx: number; vy: number; vz: number;
  color: THREE.Color;
}

export class EcologyParticles {
  readonly points: THREE.Points;

  private material: THREE.ShaderMaterial;
  private particles: EcoParticle[];
  private positions: Float32Array;
  private lifes: Float32Array;
  private maxLifes: Float32Array;
  private colors: Float32Array;
  private geometry: THREE.BufferGeometry;
  private emitTimer = 0;

  constructor() {
    this.particles = new Array(MAX_PARTICLES);
    for (let i = 0; i < MAX_PARTICLES; i++) {
      this.particles[i] = {
        alive: false, life: 0, maxLife: 1,
        x: 0, y: 0, z: 0, vx: 0, vy: 0, vz: 0,
        color: new THREE.Color(),
      };
    }

    this.positions = new Float32Array(MAX_PARTICLES * 3);
    this.lifes = new Float32Array(MAX_PARTICLES);
    this.maxLifes = new Float32Array(MAX_PARTICLES);
    this.colors = new Float32Array(MAX_PARTICLES * 3);

    this.geometry = new THREE.BufferGeometry();
    this.geometry.setAttribute('position', new THREE.BufferAttribute(this.positions, 3));
    this.geometry.setAttribute('aLife', new THREE.BufferAttribute(this.lifes, 1));
    this.geometry.setAttribute('aMaxLife', new THREE.BufferAttribute(this.maxLifes, 1));
    this.geometry.setAttribute('aColor', new THREE.BufferAttribute(this.colors, 3));

    this.material = new THREE.ShaderMaterial({
      vertexShader: ECO_VERT,
      fragmentShader: ECO_FRAG,
      transparent: true,
      depthWrite: false,
      blending: THREE.AdditiveBlending,
    });

    this.points = new THREE.Points(this.geometry, this.material);
    this.points.name = 'ecology-particles';
    this.points.frustumCulled = false;
  }

  /**
   * Scan the grid and fauna for interaction hotspots and emit particles.
   * Call periodically (not every frame — every ~0.5s is fine).
   */
  emitFromInteractions(grid: Uint8Array): void {
    // Emit pollination particles near pollinators
    const faunaCount = getFaunaCount();
    const faunaView = getFaunaView();
    if (faunaView) {
      for (let i = 0; i < faunaCount; i++) {
        const f = readFauna(faunaView, i);
        if (f.type === FaunaType.Bee || f.type === FaunaType.Butterfly) {
          // Golden pollen trail behind pollinator
          this.emitTrail(f.x, f.z, f.y, INTERACTION_COLORS.pollination, 2);
        } else if (f.type === FaunaType.Worm) {
          // Nutrient particles rising from worm activity (underground)
          this.emitTrail(f.x, f.z, f.y, INTERACTION_COLORS.nutrient, 1);
        } else if (f.type === FaunaType.Beetle) {
          // Decomposition particles near beetles
          this.emitTrail(f.x, f.z, f.y, INTERACTION_COLORS.decomposition, 1);
        }
      }
    }

    // Scan for water-root contact zones (sample sparse points)
    const step = 8;
    for (let sy = 0; sy < GRID_Y; sy += step) {
      for (let sx = 0; sx < GRID_X; sx += step) {
        for (let sz = GROUND_LEVEL - 10; sz < GROUND_LEVEL; sz += 2) {
          const idx = (sx + sy * GRID_X + sz * GRID_X * GRID_Y) * VOXEL_BYTES;
          const mat = grid[idx];
          const water = grid[idx + 1];
          if (mat === Material.Root && water > 30) {
            // Water being absorbed by roots — blue particles
            // Coordinate swap: sim (x,y,z) Z-up → Three.js (x,z,y) Y-up
            this.emitTrail(sx + 0.5, sz + 0.5, sy + 0.5, INTERACTION_COLORS.waterAbsorb, 1);
          }
        }
      }
    }
  }

  /** Emit a small trail of particles at a position */
  private emitTrail(
    worldX: number, worldY: number, worldZ: number,
    palette: THREE.Color[], count: number,
  ): void {
    for (let i = 0; i < count; i++) {
      const p = this.findDead();
      if (!p) return;

      p.alive = true;
      p.maxLife = 1.0 + Math.random() * 1.0;
      p.life = p.maxLife;

      p.x = worldX + (Math.random() - 0.5) * 0.5;
      p.y = worldY + (Math.random() - 0.5) * 0.5;
      p.z = worldZ + (Math.random() - 0.5) * 0.5;

      // Gentle drift
      p.vx = (Math.random() - 0.5) * 0.3;
      p.vy = 0.2 + Math.random() * 0.3;  // Y-up drift
      p.vz = (Math.random() - 0.5) * 0.3;

      const c = palette[Math.floor(Math.random() * palette.length)];
      p.color.copy(c);
    }
  }

  /** Update particles. Call each frame. */
  update(dt: number, grid: Uint8Array): void {
    // Periodically emit new interaction particles
    this.emitTimer += dt;
    if (this.emitTimer > 0.3) {
      this.emitTimer = 0;
      this.emitFromInteractions(grid);
    }

    // Simulate particles
    for (let i = 0; i < MAX_PARTICLES; i++) {
      const p = this.particles[i];
      if (!p.alive) {
        this.positions[i * 3] = 0;
        this.positions[i * 3 + 1] = -1000;
        this.positions[i * 3 + 2] = 0;
        this.lifes[i] = 0;
        this.maxLifes[i] = 1;
        continue;
      }

      p.life -= dt;
      if (p.life <= 0) {
        p.alive = false;
        continue;
      }

      // Gentle physics
      p.vy -= 0.05 * dt; // very light gravity
      p.vx *= 0.97;
      p.vy *= 0.97;
      p.vz *= 0.97;

      p.x += p.vx * dt;
      p.y += p.vy * dt;
      p.z += p.vz * dt;

      this.positions[i * 3] = p.x;
      this.positions[i * 3 + 1] = p.y;
      this.positions[i * 3 + 2] = p.z;
      this.lifes[i] = p.life;
      this.maxLifes[i] = p.maxLife;
      this.colors[i * 3] = p.color.r;
      this.colors[i * 3 + 1] = p.color.g;
      this.colors[i * 3 + 2] = p.color.b;
    }

    // Update GPU buffers
    (this.geometry.getAttribute('position') as THREE.BufferAttribute).needsUpdate = true;
    (this.geometry.getAttribute('aLife') as THREE.BufferAttribute).needsUpdate = true;
    (this.geometry.getAttribute('aMaxLife') as THREE.BufferAttribute).needsUpdate = true;
    (this.geometry.getAttribute('aColor') as THREE.BufferAttribute).needsUpdate = true;
    this.geometry.setDrawRange(0, MAX_PARTICLES);
  }

  private findDead(): EcoParticle | null {
    for (let i = 0; i < MAX_PARTICLES; i++) {
      if (!this.particles[i].alive) return this.particles[i];
    }
    return null;
  }

  dispose(): void {
    this.geometry.dispose();
    this.material.dispose();
  }
}
