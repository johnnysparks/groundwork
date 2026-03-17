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
import { GRID_X, GRID_Y, GRID_Z, GROUND_LEVEL, VOXEL_BYTES, Material, materialIsFoliage as isFoliage } from '../bridge';

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

/** Flower petal colors by species_id. 7=Wildflower (pink-purple), 8=Daisy (warm yellow). */
const PETAL_COLORS: Record<number, THREE.Color[]> = {
  7: [ // Wildflower: pink-purple petals
    new THREE.Color(0.75, 0.40, 0.60),
    new THREE.Color(0.85, 0.50, 0.65),
    new THREE.Color(0.65, 0.35, 0.55),
  ],
  8: [ // Daisy: white-yellow petals
    new THREE.Color(0.90, 0.85, 0.55),
    new THREE.Color(0.95, 0.90, 0.70),
    new THREE.Color(0.85, 0.80, 0.45),
  ],
};

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

  /** Flower leaf positions for petal scatter on wind gusts */
  private flowerPositions: { x: number; y: number; z: number; species: number }[] = [];

  /** Previous tick seed position keys for detecting seed→plant sprout transitions */
  private prevSeedKeys = new Set<number>();

  /** Root voxel count — exposed so main can detect root growth */
  private _rootCount = 0;
  private _prevRootCount = 0;

  /** Dead wood positions for fungi spore emission */
  private deadWoodPositions: { x: number; y: number; z: number }[] = [];

  /** How many new root voxels appeared since last detectGrowth() */
  get rootGrowthDelta(): number { return Math.max(0, this._rootCount - this._prevRootCount); }

  /** Whether dead wood exists in the garden */
  get hasDeadWood(): boolean { return this.deadWoodPositions.length > 0; }

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
   * Emit flower petals scattered by a wind gust.
   * Picks random flower positions and emits colored petals that drift with wind.
   */
  emitPetalBurst(): void {
    if (this.flowerPositions.length === 0) return;
    // Pick up to 6 random flower sources
    const sources = Math.min(6, this.flowerPositions.length);
    for (let s = 0; s < sources; s++) {
      const idx = Math.floor(Math.random() * this.flowerPositions.length);
      const flower = this.flowerPositions[idx];
      const palette = PETAL_COLORS[flower.species];
      if (!palette) continue;

      // 2-3 petals per source
      const count = 2 + Math.floor(Math.random() * 2);
      for (let i = 0; i < count; i++) {
        const p = this.findDeadParticle();
        if (!p) return;

        p.alive = true;
        p.life = 2.0 + Math.random() * 1.5;  // long-lived drift
        p.maxLife = p.life;

        // Spawn at flower position (sim Z=up → Three.js Y=up)
        p.x = flower.x + 0.5 + (Math.random() - 0.5) * 0.8;
        p.y = flower.z + 0.5 + (Math.random() - 0.5) * 0.5;
        p.z = flower.y + 0.5 + (Math.random() - 0.5) * 0.8;

        // Wind-carried drift: strong horizontal + gentle tumble
        const windAngle = Math.random() * Math.PI * 0.5 - Math.PI * 0.25; // roughly downwind
        p.vx = Math.cos(windAngle) * (0.6 + Math.random() * 0.5);
        p.vy = 0.1 + Math.random() * 0.3;  // slight rise then gravity takes over
        p.vz = Math.sin(windAngle) * (0.3 + Math.random() * 0.3);

        const c = palette[Math.floor(Math.random() * palette.length)];
        p.color.copy(c);
      }
    }
  }

  /**
   * Emit a small spiral of dust during drought — a mini dust devil.
   */
  emitDustDevil(worldX: number, worldZ: number): void {
    const count = 6;
    const baseAngle = Math.random() * Math.PI * 2;
    for (let i = 0; i < count; i++) {
      const p = this.findDeadParticle();
      if (!p) return;

      p.alive = true;
      p.life = 1.0 + Math.random() * 0.8;
      p.maxLife = p.life;

      // Spiral placement
      const angle = baseAngle + (i / count) * Math.PI * 2;
      const r = 0.3 + (i / count) * 0.5;
      p.x = worldX + Math.cos(angle) * r;
      p.y = GROUND_LEVEL + 0.2 + (i / count) * 2;
      p.z = worldZ + Math.sin(angle) * r;

      // Spiral upward motion
      const tangent = angle + Math.PI / 2;
      p.vx = Math.cos(tangent) * 0.4 + (Math.random() - 0.5) * 0.1;
      p.vy = 0.5 + Math.random() * 0.3;
      p.vz = Math.sin(tangent) * 0.4 + (Math.random() - 0.5) * 0.1;

      // Dry dusty tan/brown
      const t = Math.random();
      p.color.setRGB(0.65 + t * 0.15, 0.55 + t * 0.1, 0.35 + t * 0.1);
    }
  }

  /**
   * Emit a celebratory sparkle burst when a new fauna arrives.
   * Bright warm particles spiral outward — the garden welcomes new life.
   */
  emitFaunaArrival(worldX: number, worldY: number, worldZ: number): void {
    const count = 12;
    for (let i = 0; i < count; i++) {
      const p = this.findDeadParticle();
      if (!p) return;

      p.alive = true;
      p.life = 1.2 + Math.random() * 0.8;
      p.maxLife = p.life;

      p.x = worldX + (Math.random() - 0.5) * 0.3;
      p.y = worldY + (Math.random() - 0.5) * 0.3;
      p.z = worldZ + (Math.random() - 0.5) * 0.3;

      // Spiral outward
      const angle = (i / count) * Math.PI * 2 + Math.random() * 0.3;
      const speed = 0.4 + Math.random() * 0.4;
      p.vx = Math.cos(angle) * speed;
      p.vy = 0.3 + Math.random() * 0.5;
      p.vz = Math.sin(angle) * speed;

      // Warm gold/white sparkle
      const t = Math.random();
      p.color.setRGB(0.95 + t * 0.05, 0.85 + t * 0.1, 0.5 + t * 0.3);
    }
  }

  /**
   * Emit a few leaf fragments near a world position (camera pan rustle).
   * Makes the canopy feel responsive to the player's movement.
   */
  emitCameraRustle(worldX: number, worldY: number, worldZ: number): void {
    const count = 2;
    for (let i = 0; i < count; i++) {
      const p = this.findDeadParticle();
      if (!p) return;

      p.alive = true;
      p.life = 0.8 + Math.random() * 0.6;
      p.maxLife = p.life;

      // Scattered near the position, slightly above ground
      p.x = worldX + (Math.random() - 0.5) * 6;
      p.y = Math.max(GROUND_LEVEL + 2, worldY) + Math.random() * 3;
      p.z = worldZ + (Math.random() - 0.5) * 6;

      // Tumble outward and down
      const angle = Math.random() * Math.PI * 2;
      p.vx = Math.cos(angle) * (0.3 + Math.random() * 0.3);
      p.vy = -0.1 + Math.random() * 0.2;
      p.vz = Math.sin(angle) * (0.3 + Math.random() * 0.3);

      // Leaf greens with autumn tint variation
      const t = Math.random();
      p.color.setRGB(0.3 + t * 0.25, 0.5 + t * 0.2, 0.15 + t * 0.1);
    }
  }

  /**
   * Emit a tiny rain splash at ground impact point.
   */
  emitRainSplash(worldX: number, worldY: number, worldZ: number): void {
    const p = this.findDeadParticle();
    if (!p) return;

    p.alive = true;
    p.life = 0.3 + Math.random() * 0.2;
    p.maxLife = p.life;

    p.x = worldX;
    p.y = worldY + 0.1;
    p.z = worldZ;

    // Tiny outward splash
    const angle = Math.random() * Math.PI * 2;
    p.vx = Math.cos(angle) * 0.2;
    p.vy = 0.2 + Math.random() * 0.15;
    p.vz = Math.sin(angle) * 0.2;

    // Light blue-white
    const t = Math.random();
    p.color.setRGB(0.6 + t * 0.3, 0.7 + t * 0.2, 0.85 + t * 0.15);
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
    const newSeedKeys = new Set<number>();
    const newFlowerPositions: { x: number; y: number; z: number; species: number }[] = [];
    const newDeadWoodPositions: { x: number; y: number; z: number }[] = [];
    let rootCount = 0;

    for (let z = 0; z < GRID_Z; z++) {
      for (let y = 0; y < GRID_Y; y++) {
        for (let x = 0; x < GRID_X; x++) {
          const idx = (x + y * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
          const mat = grid[idx];
          if (mat === Material.Root) rootCount++;
          if (mat === Material.DeadWood && newDeadWoodPositions.length < 20) {
            newDeadWoodPositions.push({ x, y, z });
          }

          if (isFoliage(mat) || mat === Material.Trunk || mat === Material.Branch) {
            const posKey = x + y * GRID_X + z * GRID_X * GRID_Y;
            newLeafPositions.add(posKey);

            // If this is a new vegetation voxel, emit particles
            if (!this.prevLeafPositions.has(posKey)) {
              // Sim Y↔Z swap: sim Z=up → Three.js Y=up
              this.emit(x + 0.5, z + 0.5, y + 0.5);

              // Seed→plant sprout pop: if this was a seed last tick, bigger burst
              if (this.prevSeedKeys.has(posKey)) {
                this.emitSproutPop(x + 0.5, z + 0.5, y + 0.5);
              }
            }
          } else if (mat === Material.Seed) {
            const seedKey = x + y * GRID_X + z * GRID_X * GRID_Y;
            newSeedPositions.push({ x, y, z });
            newSeedKeys.add(seedKey);

            // New seed appeared (wind/bird dispersal) — emit a falling trail
            if (!this.prevSeedKeys.has(seedKey)) {
              this.emitSeedLanding(x + 0.5, z + 0.5, y + 0.5);
            }
          }

          // Track flower leaf positions for petal scatter
          if (isFoliage(mat)) {
            const speciesId = grid[idx + 3];
            if (speciesId === 7 || speciesId === 8) {
              newFlowerPositions.push({ x, y, z, species: speciesId });
            }
          }
        }
      }
    }

    // Detect die-off: vegetation lost since last tick → emit wilting particles
    // Sample up to 5 lost positions to avoid performance hit
    let wiltCount = 0;
    for (const prevKey of this.prevLeafPositions) {
      if (wiltCount >= 5) break;
      if (!newLeafPositions.has(prevKey)) {
        // Decode position from key
        const x = prevKey % GRID_X;
        const y = Math.floor(prevKey / GRID_X) % GRID_Y;
        const z = Math.floor(prevKey / (GRID_X * GRID_Y));
        this.emitWilt(x + 0.5, z + 0.5, y + 0.5);
        wiltCount++;
      }
    }

    this.prevLeafPositions = newLeafPositions;
    this.seedPositions = newSeedPositions;
    this.prevSeedKeys = newSeedKeys;
    this.flowerPositions = newFlowerPositions;
    this._prevRootCount = this._rootCount;
    this._rootCount = rootCount;
    this.deadWoodPositions = newDeadWoodPositions;
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

  /**
   * Emit brown wilting particles when vegetation dies.
   * Amber-brown particles drift downward — visual "letting go."
   */
  private emitWilt(worldX: number, worldY: number, worldZ: number): void {
    const count = 3;
    for (let i = 0; i < count; i++) {
      const p = this.findDeadParticle();
      if (!p) return;

      p.alive = true;
      p.life = 1.0 + Math.random() * 0.8;
      p.maxLife = p.life;

      p.x = worldX + (Math.random() - 0.5) * 0.6;
      p.y = worldY + (Math.random() - 0.5) * 0.4;
      p.z = worldZ + (Math.random() - 0.5) * 0.6;

      // Drift downward and outward
      const angle = Math.random() * Math.PI * 2;
      p.vx = Math.cos(angle) * 0.15;
      p.vy = -0.3 - Math.random() * 0.2; // falls
      p.vz = Math.sin(angle) * 0.15;

      // Dry brown/amber
      const t = Math.random();
      p.color.setRGB(0.55 + t * 0.15, 0.40 + t * 0.1, 0.15 + t * 0.1);
    }
  }

  /**
   * Emit a descending golden trail at a newly-dispersed seed landing site.
   * Shows where wind or bird carried a seed.
   */
  private emitSeedLanding(worldX: number, worldY: number, worldZ: number): void {
    const count = 4;
    for (let i = 0; i < count; i++) {
      const p = this.findDeadParticle();
      if (!p) return;

      p.alive = true;
      p.life = 0.8 + Math.random() * 0.4;
      p.maxLife = p.life;

      // Trail descends from above the landing point
      p.x = worldX + (Math.random() - 0.5) * 0.3;
      p.y = worldY + 2 + Math.random() * 3; // starts above
      p.z = worldZ + (Math.random() - 0.5) * 0.3;

      // Falling + slight drift
      p.vx = (Math.random() - 0.5) * 0.15;
      p.vy = -1.5 - Math.random() * 0.5; // falls down
      p.vz = (Math.random() - 0.5) * 0.15;

      const c = SEED_COLORS[Math.floor(Math.random() * SEED_COLORS.length)];
      p.color.copy(c);
    }
  }

  /**
   * Emit a celebratory sprout pop when a seed becomes a plant.
   * Bright green outward burst — the moment of birth.
   */
  private emitSproutPop(worldX: number, worldY: number, worldZ: number): void {
    const count = 8;
    for (let i = 0; i < count; i++) {
      const p = this.findDeadParticle();
      if (!p) return;

      p.alive = true;
      p.life = 1.0 + Math.random() * 0.5;
      p.maxLife = p.life;

      p.x = worldX + (Math.random() - 0.5) * 0.4;
      p.y = worldY + Math.random() * 0.3;
      p.z = worldZ + (Math.random() - 0.5) * 0.4;

      // Outward pop in all directions
      const angle = Math.random() * Math.PI * 2;
      const speed = 0.3 + Math.random() * 0.4;
      p.vx = Math.cos(angle) * speed;
      p.vy = 0.5 + Math.random() * 0.5;
      p.vz = Math.sin(angle) * speed;

      // Bright spring green
      const t = Math.random();
      p.color.setRGB(0.3 + t * 0.2, 0.7 + t * 0.2, 0.2 + t * 0.15);
    }
  }

  /**
   * Emit a warm soil steam wisp at a random ground-level position.
   * Call during dawn to make bare earth feel alive as morning sun heats it.
   */
  emitSoilSteam(worldX: number, worldZ: number): void {
    const p = this.findDeadParticle();
    if (!p) return;

    p.alive = true;
    p.life = 1.5 + Math.random() * 1.0;
    p.maxLife = p.life;

    p.x = worldX + (Math.random() - 0.5) * 0.8;
    p.y = GROUND_LEVEL + 0.5;
    p.z = worldZ + (Math.random() - 0.5) * 0.8;

    // Slow rise with gentle drift
    p.vx = (Math.random() - 0.5) * 0.1;
    p.vy = 0.2 + Math.random() * 0.15;
    p.vz = (Math.random() - 0.5) * 0.1;

    // Warm translucent — pale amber/beige
    const t = Math.random();
    p.color.setRGB(0.75 + t * 0.15, 0.65 + t * 0.15, 0.50 + t * 0.1);
  }

  /**
   * Emit a single water drip from a random foliage position.
   * Call periodically after rain stops to make the garden feel "wet."
   */
  emitLeafDrip(): void {
    if (this.flowerPositions.length === 0 && this.prevLeafPositions.size === 0) return;

    // Pick a random leaf position from the prevLeafPositions set
    // We sample up to 3 keys to find one, then emit a drip
    const keys = this.prevLeafPositions;
    if (keys.size === 0) return;

    // Pick a random element from the set
    const idx = Math.floor(Math.random() * keys.size);
    let posKey = 0;
    let i = 0;
    for (const k of keys) {
      if (i === idx) { posKey = k; break; }
      i++;
    }

    // Decode position key (same encoding as detectGrowth: x * GRID_Z * GRID_Y + z * GRID_Y + y)
    const y = posKey % GRID_Y;
    const zSlice = Math.floor(posKey / GRID_Y);
    const z = zSlice % GRID_Z;
    const x = Math.floor(zSlice / GRID_Z);

    const p = this.findDeadParticle();
    if (!p) return;

    p.alive = true;
    p.life = 0.6 + Math.random() * 0.4;
    p.maxLife = p.life;

    // Three.js Y-up: sim (x, y, z) → Three.js (x, z, y)
    p.x = x + 0.5 + (Math.random() - 0.5) * 0.3;
    p.y = z + 0.3; // just below leaf
    p.z = y + 0.5 + (Math.random() - 0.5) * 0.3;

    // Falls straight down with tiny horizontal drift
    p.vx = (Math.random() - 0.5) * 0.05;
    p.vy = -1.0 - Math.random() * 0.5;
    p.vz = (Math.random() - 0.5) * 0.05;

    // Pale blue-white water drop
    const t = Math.random();
    p.color.setRGB(0.55 + t * 0.25, 0.7 + t * 0.2, 0.85 + t * 0.15);
  }

  /**
   * Emit a sunbeam shaft particle — bright golden motes streaming downward
   * through gaps in the canopy. Creates visible light columns.
   */
  emitSunbeam(worldX: number, worldZ: number, pollen = 0): void {
    const p = this.findDeadParticle();
    if (!p) return;

    p.alive = true;
    p.life = 1.5 + Math.random() * 1.0;
    p.maxLife = p.life;

    // High up in canopy, drifts slowly downward
    p.x = worldX + (Math.random() - 0.5) * 1.5;
    p.y = GROUND_LEVEL + 10 + Math.random() * 15;
    p.z = worldZ + (Math.random() - 0.5) * 1.5;

    // Slow downward drift with slight horizontal wander; pollen drifts more
    p.vx = (Math.random() - 0.5) * (0.05 + pollen * 0.08);
    p.vy = -0.3 - Math.random() * 0.2;
    p.vz = (Math.random() - 0.5) * (0.05 + pollen * 0.08);

    // Warm golden-white; shifts deeper gold when pollen is high
    const t = Math.random();
    p.color.setRGB(1.0, 0.92 + t * 0.08 - pollen * 0.1, 0.7 + t * 0.2 - pollen * 0.2);
  }

  /**
   * Emit a golden figure-8 particle for bee waggle dance during pollination.
   * Two particles trace opposing loops to suggest the iconic dance pattern.
   */
  emitBeeWaggle(worldX: number, worldZ: number): void {
    for (let k = 0; k < 2; k++) {
      const p = this.findDeadParticle();
      if (!p) return;

      p.alive = true;
      p.life = 0.6 + Math.random() * 0.4;
      p.maxLife = p.life;

      // Near bee position at ground level, opposing figure-8 halves
      const sign = k === 0 ? 1 : -1;
      p.x = worldX + sign * 0.15;
      p.y = GROUND_LEVEL + 1.5 + Math.random() * 0.3;
      p.z = worldZ;

      // Figure-8: horizontal loop + vertical bob
      p.vx = sign * (0.4 + Math.random() * 0.2);
      p.vy = 0.1 * (Math.random() - 0.5);
      p.vz = (Math.random() - 0.5) * 0.3;

      // Warm honey-gold
      p.color.setRGB(1.0, 0.82 + Math.random() * 0.1, 0.3 + Math.random() * 0.15);
    }
  }

  /**
   * Emit a pale moth particle that flutters near a light source.
   * Creates night-time insect activity around fireflies.
   */
  emitMothFlutter(worldX: number, worldY: number, worldZ: number): void {
    const p = this.findDeadParticle();
    if (!p) return;

    p.alive = true;
    p.life = 0.8 + Math.random() * 0.6;
    p.maxLife = p.life;

    // Near the light source, slightly offset
    const angle = Math.random() * Math.PI * 2;
    const dist = 0.5 + Math.random() * 1.0;
    p.x = worldX + Math.cos(angle) * dist;
    p.y = worldY + (Math.random() - 0.5) * 0.5;
    p.z = worldZ + Math.sin(angle) * dist;

    // Flutter in circular motion toward light
    p.vx = Math.cos(angle + Math.PI / 2) * 0.6;
    p.vy = (Math.random() - 0.3) * 0.2;
    p.vz = Math.sin(angle + Math.PI / 2) * 0.6;

    // Pale cream-white moth color
    const t = Math.random();
    p.color.setRGB(0.85 + t * 0.1, 0.82 + t * 0.1, 0.7 + t * 0.15);
  }

  /**
   * Emit fungi/decomposition spore particles near dead wood.
   * Warm brown-orange particles drift upward slowly.
   */
  emitFungiSpore(): void {
    if (this.deadWoodPositions.length === 0) return;
    const dw = this.deadWoodPositions[Math.floor(Math.random() * this.deadWoodPositions.length)];

    const p = this.findDeadParticle();
    if (!p) return;

    p.alive = true;
    p.life = 1.0 + Math.random() * 1.0;
    p.maxLife = p.life;

    // Near the dead wood, sim Z→Three.js Y
    p.x = dw.x + (Math.random() - 0.5) * 1.5;
    p.y = dw.z + 0.5 + Math.random() * 0.5;
    p.z = dw.y + (Math.random() - 0.5) * 1.5;

    // Slow upward drift
    p.vx = (Math.random() - 0.5) * 0.08;
    p.vy = 0.1 + Math.random() * 0.1;
    p.vz = (Math.random() - 0.5) * 0.08;

    // Mushroom brown-orange tones
    const t = Math.random();
    p.color.setRGB(0.6 + t * 0.15, 0.4 + t * 0.15, 0.2 + t * 0.1);
  }

  /**
   * Emit tiny dust puffs behind a squirrel — footprints in dirt.
   * Shows ground-level scurrying activity.
   */
  emitSquirrelPrints(worldX: number, worldZ: number): void {
    const p = this.findDeadParticle();
    if (!p) return;

    p.alive = true;
    p.life = 0.3 + Math.random() * 0.2;
    p.maxLife = p.life;

    p.x = worldX + (Math.random() - 0.5) * 0.3;
    p.y = GROUND_LEVEL + 0.05;
    p.z = worldZ + (Math.random() - 0.5) * 0.3;

    // Tiny upward puff
    p.vx = (Math.random() - 0.5) * 0.1;
    p.vy = 0.15 + Math.random() * 0.1;
    p.vz = (Math.random() - 0.5) * 0.1;

    // Light dust brown
    const t = Math.random();
    p.color.setRGB(0.55 + t * 0.15, 0.45 + t * 0.1, 0.3 + t * 0.1);
  }

  /**
   * Emit a soft golden glow at water surface below a firefly.
   * Creates the illusion of firefly light reflecting off water.
   */
  emitFireflyReflection(worldX: number, worldZ: number): void {
    const p = this.findDeadParticle();
    if (!p) return;

    p.alive = true;
    p.life = 0.4 + Math.random() * 0.3;
    p.maxLife = p.life;

    // At water surface level
    p.x = worldX + (Math.random() - 0.5) * 0.5;
    p.y = GROUND_LEVEL + 0.1;
    p.z = worldZ + (Math.random() - 0.5) * 0.5;

    // Stationary — reflection stays on surface
    p.vx = 0;
    p.vy = 0;
    p.vz = 0;

    // Warm golden glow matching firefly color
    const t = Math.random();
    p.color.setRGB(0.7 + t * 0.3, 0.75 + t * 0.15, 0.25 + t * 0.15);
  }

  /**
   * Emit a wind streak — a fast-moving horizontal particle that shows
   * the wind direction. Called during gusts for visible air movement.
   */
  emitWindStreak(windAngle: number, strength: number): void {
    const p = this.findDeadParticle();
    if (!p) return;

    p.alive = true;
    p.life = 0.4 + Math.random() * 0.3;
    p.maxLife = p.life;

    // Random position above the garden
    p.x = 10 + Math.random() * (GRID_X - 20);
    p.y = GROUND_LEVEL + 2 + Math.random() * 15;
    p.z = 10 + Math.random() * (GRID_Y - 20);

    // Fast horizontal movement in wind direction
    const speed = (3 + Math.random() * 2) * strength;
    p.vx = Math.cos(windAngle) * speed;
    p.vy = (Math.random() - 0.5) * 0.3;
    p.vz = Math.sin(windAngle) * speed;

    // Nearly invisible white — just visible enough to show direction
    p.color.setRGB(0.8, 0.82, 0.85);
  }

  /**
   * Emit a faint iridescent shimmer behind a beetle — shows their path
   * through the garden as they decompose organic matter.
   */
  emitBeetleTrail(worldX: number, worldZ: number): void {
    const p = this.findDeadParticle();
    if (!p) return;

    p.alive = true;
    p.life = 0.6 + Math.random() * 0.4;
    p.maxLife = p.life;

    // Just above ground behind beetle
    p.x = worldX + (Math.random() - 0.5) * 0.4;
    p.y = GROUND_LEVEL + 0.15;
    p.z = worldZ + (Math.random() - 0.5) * 0.4;

    // Tiny drift
    p.vx = (Math.random() - 0.5) * 0.05;
    p.vy = 0.02;
    p.vz = (Math.random() - 0.5) * 0.05;

    // Iridescent green-blue (moss/beetle wing shimmer)
    const t = Math.random();
    p.color.setRGB(0.3 + t * 0.15, 0.5 + t * 0.2, 0.35 + t * 0.25);
  }

  /**
   * Emit a tiny pollen mote trailing behind a butterfly in flight.
   * Soft yellow-white particles that drift slowly downward.
   */
  emitButterflyPollen(worldX: number, worldZ: number): void {
    const p = this.findDeadParticle();
    if (!p) return;

    p.alive = true;
    p.life = 0.8 + Math.random() * 0.6;
    p.maxLife = p.life;

    // Near butterfly flight height with slight scatter
    p.x = worldX + (Math.random() - 0.5) * 0.6;
    p.y = GROUND_LEVEL + 1.5 + Math.random() * 1.5;
    p.z = worldZ + (Math.random() - 0.5) * 0.6;

    // Gentle float downward with wandering drift
    p.vx = (Math.random() - 0.5) * 0.15;
    p.vy = -0.1 - Math.random() * 0.08;
    p.vz = (Math.random() - 0.5) * 0.15;

    // Soft warm yellow (pollen)
    const t = Math.random();
    p.color.setRGB(1.0, 0.9 + t * 0.1, 0.5 + t * 0.2);
  }

  /**
   * Emit a tiny twig/fiber particle from a perched bird — suggests nesting
   * activity. Brown-tan particles tumble slowly downward from canopy.
   */
  emitBirdNesting(worldX: number, worldZ: number): void {
    const p = this.findDeadParticle();
    if (!p) return;

    p.alive = true;
    p.life = 1.0 + Math.random() * 0.8;
    p.maxLife = p.life;

    // From canopy height, near perch position
    p.x = worldX + (Math.random() - 0.5) * 0.8;
    p.y = GROUND_LEVEL + 6 + Math.random() * 4;
    p.z = worldZ + (Math.random() - 0.5) * 0.8;

    // Tumble downward with wobble
    p.vx = (Math.random() - 0.5) * 0.2;
    p.vy = -0.15 - Math.random() * 0.1;
    p.vz = (Math.random() - 0.5) * 0.2;

    // Brown-tan twig color
    const t = Math.random();
    p.color.setRGB(0.55 + t * 0.15, 0.4 + t * 0.1, 0.2 + t * 0.1);
  }

  /**
   * Emit a ground-level wet shimmer — creates the impression of puddles
   * forming on flat ground during rain. Short-lived reflective sparkle.
   */
  emitPuddleShimmer(worldX: number, worldZ: number): void {
    const p = this.findDeadParticle();
    if (!p) return;

    p.alive = true;
    p.life = 1.5 + Math.random() * 1.5;
    p.maxLife = p.life;

    // Right at ground level, on the surface
    p.x = worldX + (Math.random() - 0.5) * 2;
    p.y = GROUND_LEVEL + 0.05;
    p.z = worldZ + (Math.random() - 0.5) * 2;

    // No movement — puddles are stationary
    p.vx = 0;
    p.vy = 0;
    p.vz = 0;

    // Cool blue-white reflective tint
    const t = Math.random();
    p.color.setRGB(0.55 + t * 0.2, 0.7 + t * 0.15, 0.85 + t * 0.15);
  }

  /**
   * Emit small soil disturbance particles around a worm's position.
   * Shows underground activity — tiny earthy crumbs puffing up.
   */
  emitWormTrail(worldX: number, worldZ: number): void {
    const count = 2;
    for (let i = 0; i < count; i++) {
      const p = this.findDeadParticle();
      if (!p) return;

      p.alive = true;
      p.life = 0.5 + Math.random() * 0.4;
      p.maxLife = p.life;

      // Near-surface, scattered around worm position
      p.x = worldX + (Math.random() - 0.5) * 0.8;
      p.y = GROUND_LEVEL + 0.3 + Math.random() * 0.3;
      p.z = worldZ + (Math.random() - 0.5) * 0.8;

      // Tiny upward puff
      p.vx = (Math.random() - 0.5) * 0.15;
      p.vy = 0.2 + Math.random() * 0.15;
      p.vz = (Math.random() - 0.5) * 0.15;

      // Earthy brown tones
      const t = Math.random();
      p.color.setRGB(0.45 + t * 0.15, 0.35 + t * 0.1, 0.2 + t * 0.1);
    }
  }

  /**
   * Emit an expanding ring of particles on the water surface.
   * Called when flying fauna passes over water — creates a gentle ripple.
   */
  emitWaterRipple(worldX: number, waterY: number, worldZ: number): void {
    const count = 8;
    const baseAngle = Math.random() * Math.PI * 2;
    for (let i = 0; i < count; i++) {
      const p = this.findDeadParticle();
      if (!p) return;

      p.alive = true;
      p.life = 0.6 + Math.random() * 0.3;
      p.maxLife = p.life;

      // Start at center, expand outward in a ring
      const angle = baseAngle + (i / count) * Math.PI * 2;
      p.x = worldX + Math.cos(angle) * 0.15;
      p.y = waterY + 0.1;
      p.z = worldZ + Math.sin(angle) * 0.15;

      const speed = 0.6 + Math.random() * 0.3;
      p.vx = Math.cos(angle) * speed;
      p.vy = 0; // stays on water surface
      p.vz = Math.sin(angle) * speed;

      // Pale blue-white water ring
      const t = Math.random();
      p.color.setRGB(0.6 + t * 0.2, 0.75 + t * 0.15, 0.9 + t * 0.1);
    }
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
