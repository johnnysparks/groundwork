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
  getFaunaCount, getFaunaView, readFauna, FaunaType, FaunaState,
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
    gl_PointSize = sizeCurve * 8.0;
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
  nitrogen: [
    new THREE.Color(0.30, 0.75, 0.25),  // fresh green
    new THREE.Color(0.40, 0.65, 0.20),  // olive green
  ],
  canopyShade: [
    new THREE.Color(0.35, 0.55, 0.70),  // cool dappled blue
    new THREE.Color(0.30, 0.50, 0.55),  // teal shade
  ],
  birdSeedDrop: [
    new THREE.Color(0.90, 0.75, 0.30),  // golden seed
    new THREE.Color(0.80, 0.65, 0.25),  // amber seed
    new THREE.Color(0.70, 0.55, 0.20),  // earthy gold
  ],
  soilDisturbance: [
    new THREE.Color(0.42, 0.30, 0.18),  // dark earth
    new THREE.Color(0.50, 0.38, 0.22),  // medium soil
    new THREE.Color(0.36, 0.26, 0.14),  // deep brown
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
  private emitInterval = 0.3;

  constructor(opts?: { mobile?: boolean }) {
    if (opts?.mobile) {
      this.emitInterval = 1.0; // scan less frequently on mobile
    }
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
          // Golden pollen trail behind pollinator — stronger when actively pollinating
          const count = f.state === FaunaState.Acting ? 10 : 5;
          this.emitTrail(f.x, f.z, f.y, INTERACTION_COLORS.pollination, count);
        } else if (f.type === FaunaType.Worm) {
          // Nutrient particles rising from worm activity (underground)
          this.emitTrail(f.x, f.z, f.y, INTERACTION_COLORS.nutrient, 3);
          // Surface soil disturbance: earthy puffs at ground level above the worm
          this.emitSoilDisturbance(f.x, f.y);
        } else if (f.type === FaunaType.Beetle) {
          // Decomposition particles near beetles
          this.emitTrail(f.x, f.z, f.y, INTERACTION_COLORS.decomposition, 3);
        } else if (f.type === FaunaType.Bird && f.state === FaunaState.Acting) {
          // Bird seed-drop: golden seeds falling from the bird
          this.emitSeedDrop(f.x, f.z, f.y);
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
            this.emitTrail(sx + 0.5, sz + 0.5, sy + 0.5, INTERACTION_COLORS.waterAbsorb, 3);
          }
        }
      }
    }

    // Nitrogen handshake: detect Trunk voxels near ground-level Leaf voxels
    // When groundcover (clover/moss/grass) is near a tree trunk, emit green
    // nitrogen particles rising from the ground — makes the 1.5x boost visible.
    const nStep = 12; // sparse scan (don't check every voxel)
    for (let sy = 0; sy < GRID_Y; sy += nStep) {
      for (let sx = 0; sx < GRID_X; sx += nStep) {
        // Check for trunk at this position (at or just above ground)
        let hasTrunk = false;
        for (let sz = GROUND_LEVEL; sz <= GROUND_LEVEL + 3; sz++) {
          const idx = (sx + sy * GRID_X + sz * GRID_X * GRID_Y) * VOXEL_BYTES;
          if (grid[idx] === Material.Trunk) { hasTrunk = true; break; }
        }
        if (!hasTrunk) continue;

        // Check for ground-level Leaf voxels within 5 voxels (nitrogen radius)
        let groundLeafCount = 0;
        const radius = 5;
        for (let dy = -radius; dy <= radius; dy += 2) {
          for (let dx = -radius; dx <= radius; dx += 2) {
            const nx = sx + dx;
            const ny = sy + dy;
            if (nx < 0 || nx >= GRID_X || ny < 0 || ny >= GRID_Y) continue;
            for (let nz = GROUND_LEVEL; nz <= GROUND_LEVEL + 2; nz++) {
              const nIdx = (nx + ny * GRID_X + nz * GRID_X * GRID_Y) * VOXEL_BYTES;
              if (grid[nIdx] === Material.Leaf) groundLeafCount++;
            }
          }
        }

        // Need 3+ leaf voxels for boost (matches sim threshold)
        if (groundLeafCount >= 3) {
          // Emit green nitrogen particles rising from ground near the trunk
          this.emitTrail(
            sx + 0.5,
            GROUND_LEVEL + 1.5, // Y-up position (just above ground)
            sy + 0.5,
            INTERACTION_COLORS.nitrogen,
            4,
          );
        }
      }
    }

    // Canopy Effect: shade-tolerant ground plants near tall trunks
    // Detect Leaf voxels at ground level with low light (shaded) near Trunk voxels above
    const cStep = 16;
    for (let sy = 0; sy < GRID_Y; sy += cStep) {
      for (let sx = 0; sx < GRID_X; sx += cStep) {
        // Check for ground-level leaf with low light (shaded by canopy above)
        const gIdx = (sx + sy * GRID_X + (GROUND_LEVEL + 1) * GRID_X * GRID_Y) * VOXEL_BYTES;
        if (grid[gIdx] !== Material.Leaf) continue;
        const lightLevel = grid[gIdx + 2]; // byte 2 = light_level
        if (lightLevel > 30 || lightLevel < 5) continue; // moderate shade = 5-30

        // Check for tall trunk above (canopy source)
        let hasTrunkAbove = false;
        for (let tz = GROUND_LEVEL + 5; tz < GROUND_LEVEL + 30; tz += 3) {
          const tIdx = (sx + sy * GRID_X + tz * GRID_X * GRID_Y) * VOXEL_BYTES;
          if (grid[tIdx] === Material.Trunk || grid[tIdx] === Material.Branch) {
            hasTrunkAbove = true;
            break;
          }
        }
        if (!hasTrunkAbove) continue;

        // Dappled shade particles — cool blue-green, drifting slowly
        this.emitTrail(
          sx + 0.5,
          GROUND_LEVEL + 2,
          sy + 0.5,
          INTERACTION_COLORS.canopyShade,
          3,
        );
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

  /** Emit small soil disturbance particles at the surface above a worm.
   *  Makes underground worm activity visible from the default camera view. */
  private emitSoilDisturbance(simX: number, simY: number): void {
    const count = 2;
    for (let i = 0; i < count; i++) {
      const p = this.findDead();
      if (!p) return;

      p.alive = true;
      p.maxLife = 0.6 + Math.random() * 0.4;
      p.life = p.maxLife;

      // Emit at ground level (Three.js Y = sim Z) above the worm's XY position
      p.x = simX + 0.5 + (Math.random() - 0.5) * 0.8;
      p.y = GROUND_LEVEL + 0.2 + Math.random() * 0.3; // just above surface
      p.z = simY + 0.5 + (Math.random() - 0.5) * 0.8;

      // Small upward puff that settles
      p.vx = (Math.random() - 0.5) * 0.2;
      p.vy = 0.15 + Math.random() * 0.15;
      p.vz = (Math.random() - 0.5) * 0.2;

      const c = INTERACTION_COLORS.soilDisturbance[
        Math.floor(Math.random() * INTERACTION_COLORS.soilDisturbance.length)
      ];
      p.color.copy(c);
    }
  }

  /** Emit seed particles falling from a bird's position */
  private emitSeedDrop(worldX: number, worldY: number, worldZ: number): void {
    const count = 5;
    for (let i = 0; i < count; i++) {
      const p = this.findDead();
      if (!p) return;

      p.alive = true;
      p.maxLife = 1.2 + Math.random() * 0.8;
      p.life = p.maxLife;

      p.x = worldX + (Math.random() - 0.5) * 1.0;
      p.y = worldY + (Math.random() - 0.5) * 0.5;
      p.z = worldZ + (Math.random() - 0.5) * 1.0;

      // Seeds fall downward with gentle drift
      p.vx = (Math.random() - 0.5) * 0.2;
      p.vy = -0.4 - Math.random() * 0.3;  // downward
      p.vz = (Math.random() - 0.5) * 0.2;

      const c = INTERACTION_COLORS.birdSeedDrop[
        Math.floor(Math.random() * INTERACTION_COLORS.birdSeedDrop.length)
      ];
      p.color.copy(c);
    }
  }

  /** Update particles. Call each frame. */
  update(dt: number, grid: Uint8Array): void {
    // Periodically emit new interaction particles
    this.emitTimer += dt;
    if (this.emitTimer > this.emitInterval) {
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
