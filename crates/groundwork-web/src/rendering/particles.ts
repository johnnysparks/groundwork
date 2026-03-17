/**
 * Growth particle burst system.
 *
 * Emits green/golden sparkle bursts when plants advance a growth stage.
 * Uses a pooled particle system with Points geometry for performance.
 *
 * Each burst spawns a cluster of particles that rise, spread outward,
 * and fade — the visual "reward sting" for growth events.
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GRID_Z, VOXEL_BYTES, Material, materialIsFoliage as isFoliage } from '../bridge';

/** Maximum particles alive at once */
const MAX_PARTICLES = 2000;

/** Particles per growth burst — kept small so bursts are subtle sparkles */
const BURST_COUNT = 5;

/** Particle lifetime in seconds */
const PARTICLE_LIFE = 1.5;

/** Particle vertex shader */
const PARTICLE_VERT = /* glsl */ `
  attribute float aLife;
  attribute float aMaxLife;
  attribute vec3 aColor;

  varying float vLife;
  varying vec3 vColor;

  void main() {
    vLife = aLife / aMaxLife;
    vColor = aColor;

    vec4 mvPos = modelViewMatrix * vec4(position, 1.0);
    // Size: grows then shrinks over lifetime — small and gentle
    float sizeCurve = sin(vLife * 3.14159);
    gl_PointSize = sizeCurve * 4.0;
    gl_Position = projectionMatrix * mvPos;
  }
`;

/** Particle fragment shader */
const PARTICLE_FRAG = /* glsl */ `
  varying float vLife;
  varying vec3 vColor;

  void main() {
    // Soft circular particle
    float dist = length(gl_PointCoord - 0.5) * 2.0;
    if (dist > 1.0) discard;
    float alpha = (1.0 - dist * dist) * vLife;

    gl_FragColor = vec4(vColor, alpha);
  }
`;

/** Growth burst color palette — warm greens and golds matching the garden palette */
const BURST_COLORS = [
  new THREE.Color(0.35, 0.65, 0.25),  // forest green (matches foliage)
  new THREE.Color(0.45, 0.70, 0.30),  // warm green
  new THREE.Color(0.80, 0.70, 0.25),  // warm golden
  new THREE.Color(0.85, 0.75, 0.30),  // light gold
  new THREE.Color(0.40, 0.60, 0.28),  // earthy green
];

/** Stage transition burst colors — celebratory warm golds and greens */
const STAGE_COLORS = [
  new THREE.Color(0.95, 0.85, 0.30),  // bright gold
  new THREE.Color(0.85, 0.70, 0.25),  // warm golden
  new THREE.Color(0.40, 0.70, 0.30),  // vibrant green
  new THREE.Color(0.90, 0.80, 0.40),  // light gold
  new THREE.Color(0.50, 0.75, 0.35),  // fresh green
];

/** Seed sparkle colors — bright warm golds to mark where life is about to begin */
const SEED_COLORS = [
  new THREE.Color(0.95, 0.85, 0.40),  // bright gold
  new THREE.Color(0.90, 0.75, 0.30),  // amber gold
  new THREE.Color(1.00, 0.92, 0.55),  // pale gold
];

interface Particle {
  alive: boolean;
  life: number;
  maxLife: number;
  x: number; y: number; z: number;
  vx: number; vy: number; vz: number;
  color: THREE.Color;
}

export class GrowthParticles {
  readonly points: THREE.Points;

  private material: THREE.ShaderMaterial;
  private particles: Particle[];
  private positions: Float32Array;
  private lifes: Float32Array;
  private maxLifes: Float32Array;
  private colors: Float32Array;
  private geometry: THREE.BufferGeometry;

  /** Previous grid snapshot for detecting growth (material bytes only) */
  private prevLeafCount = 0;
  private prevLeafPositions: Set<number> = new Set();

  /** Seed positions for persistent sparkle effect */
  private seedPositions: { x: number; y: number; z: number }[] = [];
  private seedSparkleTimer = 0;

  constructor() {
    this.particles = new Array(MAX_PARTICLES);
    for (let i = 0; i < MAX_PARTICLES; i++) {
      this.particles[i] = {
        alive: false, life: 0, maxLife: PARTICLE_LIFE,
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
      vertexShader: PARTICLE_VERT,
      fragmentShader: PARTICLE_FRAG,
      transparent: true,
      depthWrite: false,
      // NormalBlending so particles show their actual warm green/gold colors
      // instead of washing out to white when stacked (additive was too bright).
    });

    this.points = new THREE.Points(this.geometry, this.material);
    this.points.name = 'growth-particles';
    this.points.frustumCulled = false;
  }

  /**
   * Emit a burst of particles at a world position.
   */
  emit(worldX: number, worldY: number, worldZ: number): void {
    for (let i = 0; i < BURST_COUNT; i++) {
      const p = this.findDeadParticle();
      if (!p) break;

      p.alive = true;
      p.life = PARTICLE_LIFE;
      p.maxLife = PARTICLE_LIFE * (0.7 + Math.random() * 0.6);

      // Spawn at voxel center with slight spread
      p.x = worldX + (Math.random() - 0.5) * 0.8;
      p.y = worldY + (Math.random() - 0.5) * 0.8;
      p.z = worldZ + (Math.random() - 0.5) * 0.8;

      // Velocity: gentle rise with outward spread (Three.js Y = up)
      const angle = Math.random() * Math.PI * 2;
      const speed = 0.2 + Math.random() * 0.3;
      p.vx = Math.cos(angle) * speed;
      p.vy = 0.4 + Math.random() * 0.4; // gentle rise, stays near canopy
      p.vz = Math.sin(angle) * speed;

      // Random color from palette
      const c = BURST_COLORS[Math.floor(Math.random() * BURST_COLORS.length)];
      p.color.copy(c);
    }
  }

  /**
   * Emit a large celebratory burst when a tree advances a growth stage.
   * Bigger, longer-lived, and wider spread than normal growth sparkles.
   */
  emitStageBurst(worldX: number, worldY: number, worldZ: number): void {
    const count = 20;
    for (let i = 0; i < count; i++) {
      const p = this.findDeadParticle();
      if (!p) break;

      p.alive = true;
      p.life = 2.0 + Math.random() * 1.0;
      p.maxLife = p.life;

      // Spawn around the tree position with wide spread
      p.x = worldX + (Math.random() - 0.5) * 3.0;
      p.y = worldY + Math.random() * 2.0;
      p.z = worldZ + (Math.random() - 0.5) * 3.0;

      // Outward and upward burst
      const angle = Math.random() * Math.PI * 2;
      const speed = 0.4 + Math.random() * 0.5;
      p.vx = Math.cos(angle) * speed;
      p.vy = 0.6 + Math.random() * 0.6;
      p.vz = Math.sin(angle) * speed;

      const c = STAGE_COLORS[Math.floor(Math.random() * STAGE_COLORS.length)];
      p.color.copy(c);
    }
  }

  /**
   * Emit a small dust puff at the gnome's feet (walking/working).
   * Earthy brown particles that puff outward and settle quickly.
   */
  emitDustPuff(worldX: number, worldY: number, worldZ: number): void {
    const count = 2;
    for (let i = 0; i < count; i++) {
      const p = this.findDeadParticle();
      if (!p) return;

      p.alive = true;
      p.life = 0.4 + Math.random() * 0.3;
      p.maxLife = p.life;

      p.x = worldX + (Math.random() - 0.5) * 0.6;
      p.y = worldY;
      p.z = worldZ + (Math.random() - 0.5) * 0.6;

      // Small outward puff that settles
      const angle = Math.random() * Math.PI * 2;
      p.vx = Math.cos(angle) * 0.3;
      p.vy = 0.1 + Math.random() * 0.2;
      p.vz = Math.sin(angle) * 0.3;

      // Earthy brown
      const t = Math.random();
      p.color.setRGB(0.45 + t * 0.15, 0.35 + t * 0.1, 0.2 + t * 0.1);
    }
  }

  /**
   * Emit blue sparkle particles at water flow frontier positions.
   * Call when water surface area increases (channel filling).
   */
  emitWaterFlow(positions: [number, number, number][]): void {
    for (const [x, y, z] of positions) {
      const count = 3;
      for (let i = 0; i < count; i++) {
        const p = this.findDeadParticle();
        if (!p) return;

        p.alive = true;
        p.life = 0.8 + Math.random() * 0.5;
        p.maxLife = p.life;

        // Sim coords → Three.js (x, z, y)
        p.x = x + 0.5 + (Math.random() - 0.5) * 0.5;
        p.y = z + 1.0 + Math.random() * 0.3;
        p.z = y + 0.5 + (Math.random() - 0.5) * 0.5;

        // Small upward splash
        p.vx = (Math.random() - 0.5) * 0.3;
        p.vy = 0.3 + Math.random() * 0.4;
        p.vz = (Math.random() - 0.5) * 0.3;

        // Blue-white water sparkle
        const t = Math.random();
        p.color.setRGB(0.4 + t * 0.4, 0.6 + t * 0.3, 0.8 + t * 0.2);
      }
    }
  }

  /**
   * Detect new Leaf voxels in the grid and emit growth bursts.
   * Call after each sim tick with the current grid.
   */
  detectGrowth(grid: Uint8Array): void {
    const newLeafPositions = new Set<number>();
    const newSeedPositions: { x: number; y: number; z: number }[] = [];

    for (let z = 0; z < GRID_Z; z++) {
      for (let y = 0; y < GRID_Y; y++) {
        for (let x = 0; x < GRID_X; x++) {
          const idx = (x + y * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
          const mat = grid[idx];

          if (isFoliage(mat) || mat === Material.Trunk || mat === Material.Branch) {
            const posKey = x + y * GRID_X + z * GRID_X * GRID_Y;
            newLeafPositions.add(posKey);

            // If this is a new vegetation voxel, emit particles
            if (!this.prevLeafPositions.has(posKey)) {
              // Sim Y↔Z swap: sim Z=up → Three.js Y=up
              this.emit(x + 0.5, z + 0.5, y + 0.5);
            }
          } else if (mat === Material.Seed) {
            newSeedPositions.push({ x, y, z });
          }
        }
      }
    }

    this.prevLeafPositions = newLeafPositions;
    this.seedPositions = newSeedPositions;
  }

  /**
   * Update particle simulation. Call each frame with delta time in seconds.
   */
  update(dt: number): void {
    // Emit seed sparkles: 2-3 random seeds sparkle each ~0.3 seconds
    this.seedSparkleTimer += dt;
    if (this.seedSparkleTimer >= 0.3 && this.seedPositions.length > 0) {
      this.seedSparkleTimer = 0;
      const count = Math.min(3, this.seedPositions.length);
      for (let s = 0; s < count; s++) {
        const idx = Math.floor(Math.random() * this.seedPositions.length);
        const seed = this.seedPositions[idx];
        this.emitSeedSparkle(seed.x + 0.5, seed.z + 0.5, seed.y + 0.5);
      }
    }

    let activeCount = 0;

    for (let i = 0; i < MAX_PARTICLES; i++) {
      const p = this.particles[i];
      if (!p.alive) {
        // Park dead particles off-screen
        this.positions[i * 3] = 0;
        this.positions[i * 3 + 1] = 0;
        this.positions[i * 3 + 2] = -1000;
        this.lifes[i] = 0;
        this.maxLifes[i] = 1;
        continue;
      }

      p.life -= dt;
      if (p.life <= 0) {
        p.alive = false;
        continue;
      }

      // Gravity-lite: particles slow down and drift (Three.js Y = up)
      p.vy -= 0.3 * dt; // gentle gravity
      p.vx *= 0.98;
      p.vy *= 0.98;

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

      activeCount++;
    }

    // Update GPU buffers
    const posAttr = this.geometry.getAttribute('position') as THREE.BufferAttribute;
    const lifeAttr = this.geometry.getAttribute('aLife') as THREE.BufferAttribute;
    const maxLifeAttr = this.geometry.getAttribute('aMaxLife') as THREE.BufferAttribute;
    const colorAttr = this.geometry.getAttribute('aColor') as THREE.BufferAttribute;
    posAttr.needsUpdate = true;
    lifeAttr.needsUpdate = true;
    maxLifeAttr.needsUpdate = true;
    colorAttr.needsUpdate = true;

    // Set draw range to avoid processing unused particles
    this.geometry.setDrawRange(0, MAX_PARTICLES);
  }

  /**
   * Emit a single gentle golden sparkle above a seed — marks where life will begin.
   */
  private emitSeedSparkle(worldX: number, worldY: number, worldZ: number): void {
    const p = this.findDeadParticle();
    if (!p) return;

    p.alive = true;
    p.life = 0.8 + Math.random() * 0.4;
    p.maxLife = p.life;

    // Spawn just above the seed with tiny offset
    p.x = worldX + (Math.random() - 0.5) * 0.3;
    p.y = worldY + 0.3 + Math.random() * 0.3;
    p.z = worldZ + (Math.random() - 0.5) * 0.3;

    // Gentle upward drift
    p.vx = (Math.random() - 0.5) * 0.1;
    p.vy = 0.3 + Math.random() * 0.2;
    p.vz = (Math.random() - 0.5) * 0.1;

    const c = SEED_COLORS[Math.floor(Math.random() * SEED_COLORS.length)];
    p.color.copy(c);
  }

  private findDeadParticle(): Particle | null {
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
