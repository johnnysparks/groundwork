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

/** Particles per growth burst */
const BURST_COUNT = 12;

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
    // Size: grows then shrinks over lifetime
    float sizeCurve = sin(vLife * 3.14159);
    gl_PointSize = sizeCurve * 6.0;
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

/** Growth burst color palette — greens and golds */
const BURST_COLORS = [
  new THREE.Color(0.4, 0.8, 0.3),   // bright green
  new THREE.Color(0.5, 0.9, 0.4),   // light green
  new THREE.Color(0.8, 0.75, 0.3),  // golden
  new THREE.Color(0.9, 0.85, 0.4),  // light gold
  new THREE.Color(0.6, 0.9, 0.5),   // spring green
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
      blending: THREE.AdditiveBlending,
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

      // Velocity: rise upward with outward spread
      const angle = Math.random() * Math.PI * 2;
      const speed = 0.3 + Math.random() * 0.5;
      p.vx = Math.cos(angle) * speed;
      p.vy = Math.sin(angle) * speed;
      p.vz = 0.8 + Math.random() * 0.6; // mostly upward

      // Random color from palette
      const c = BURST_COLORS[Math.floor(Math.random() * BURST_COLORS.length)];
      p.color.copy(c);
    }
  }

  /**
   * Detect new Leaf voxels in the grid and emit growth bursts.
   * Call after each sim tick with the current grid.
   */
  detectGrowth(grid: Uint8Array): void {
    const newLeafPositions = new Set<number>();

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
              this.emit(x + 0.5, y + 0.5, z + 0.5);
            }
          }
        }
      }
    }

    this.prevLeafPositions = newLeafPositions;
  }

  /**
   * Update particle simulation. Call each frame with delta time in seconds.
   */
  update(dt: number): void {
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

      // Gravity-lite: particles slow down and drift
      p.vz -= 0.3 * dt; // gentle gravity
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
