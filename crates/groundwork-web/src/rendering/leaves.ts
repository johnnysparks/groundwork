/**
 * Falling leaves — ambient particles drifting down from the canopy.
 *
 * A few leaves constantly flutter down through the garden, adding
 * gentle ambient motion that makes the trees feel alive. Leaves
 * spawn at random canopy positions and sway side-to-side as they fall.
 *
 * Serves principle #7: "Idle time must be rewarding."
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GROUND_LEVEL } from '../bridge';

const MAX_LEAVES = 20;
const FALL_SPEED = 1.5; // voxels per second — gentle drift

const VERT = /* glsl */ `
  attribute float aPhase;
  attribute vec3 aColor;
  varying float vPhase;
  varying vec3 vColor;
  uniform float uTime;
  void main() {
    vPhase = aPhase;
    vColor = aColor;
    vec4 mvPos = modelViewMatrix * vec4(position, 1.0);
    gl_PointSize = 4.0 * (200.0 / -mvPos.z);
    gl_Position = projectionMatrix * mvPos;
  }
`;

const FRAG = /* glsl */ `
  varying float vPhase;
  varying vec3 vColor;
  uniform float uTime;
  void main() {
    float dist = length(gl_PointCoord - 0.5) * 2.0;
    if (dist > 1.0) discard;
    // Leaf shape: slightly elongated (wider than tall)
    vec2 uv = gl_PointCoord - 0.5;
    float leaf = 1.0 - smoothstep(0.3, 0.5, length(uv * vec2(1.0, 1.5)));
    gl_FragColor = vec4(vColor, leaf * 0.85);
  }
`;

/** Fallback autumn/green leaf colors (used when no species data) */
const LEAF_COLORS = [
  [0.45, 0.55, 0.20], // olive green
  [0.55, 0.65, 0.25], // spring green
  [0.65, 0.50, 0.15], // golden
  [0.70, 0.40, 0.15], // amber
  [0.50, 0.60, 0.20], // fresh green
  [0.40, 0.50, 0.15], // dark green
];

/** Species-tinted falling leaf colors — foliage palette shifted toward autumn.
 *  Index matches species_id: 0=Oak, 1=Birch, 2=Willow, 3=Pine. */
const SPECIES_LEAF_COLORS: number[][] = [
  [0.40, 0.45, 0.12], // Oak: deep warm olive → autumn brown-green
  [0.60, 0.65, 0.20], // Birch: bright → golden yellow
  [0.35, 0.50, 0.30], // Willow: sage → muted silver-green
  [0.15, 0.30, 0.15], // Pine: dark blue-green (needles don't turn)
];

interface Leaf {
  x: number;
  y: number;
  z: number;
  phase: number;
  swaySpeed: number;
  swayAmplitude: number;
  fallSpeed: number;
}

export class FallingLeaves {
  readonly group: THREE.Group;
  private mesh: THREE.Points;
  private positions: Float32Array;
  private phases: Float32Array;
  private colors: Float32Array;
  private leaves: Leaf[] = [];
  private timeUniform: { value: number };
  private plantCount = 0;
  private windStrength = 0.35;
  private windDirX = 1;
  private windDirZ = 0;
  private treeSpecies: number[] = []; // species IDs of trees in garden

  constructor() {
    this.group = new THREE.Group();
    this.group.name = 'fallingLeaves';

    this.positions = new Float32Array(MAX_LEAVES * 3);
    this.phases = new Float32Array(MAX_LEAVES);
    this.colors = new Float32Array(MAX_LEAVES * 3);

    for (let i = 0; i < MAX_LEAVES; i++) {
      const leaf = this.spawnLeaf();
      this.leaves.push(leaf);
      this.writeLeaf(i, leaf);
    }

    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.BufferAttribute(this.positions, 3));
    geo.setAttribute('aPhase', new THREE.BufferAttribute(this.phases, 1));
    geo.setAttribute('aColor', new THREE.BufferAttribute(this.colors, 3));

    this.timeUniform = { value: 0 };

    const mat = new THREE.ShaderMaterial({
      vertexShader: VERT,
      fragmentShader: FRAG,
      transparent: true,
      depthWrite: false,
      uniforms: { uTime: this.timeUniform },
    });

    this.mesh = new THREE.Points(geo, mat);
    this.mesh.frustumCulled = false;
    this.group.add(this.mesh);
  }

  /** Set current plant count — leaves only fall if there are trees */
  setPlantCount(count: number): void {
    this.plantCount = count;
  }

  /** Set wind strength (0–1) from weather system. Affects fall speed and lateral drift. */
  setWind(strength: number, windAngle?: number): void {
    this.windStrength = strength;
    if (windAngle !== undefined) {
      this.windDirX = Math.cos(windAngle);
      this.windDirZ = Math.sin(windAngle);
    }
  }

  /** Set tree species present in the garden (species IDs 0-3 for trees). */
  setTreeSpecies(speciesIds: number[]): void {
    this.treeSpecies = speciesIds.filter(id => id <= 3); // only trees
  }

  private spawnLeaf(): Leaf {
    return {
      x: 15 + Math.random() * (GRID_X - 30),
      y: 15 + Math.random() * (GRID_Y - 30),
      z: GROUND_LEVEL + 15 + Math.random() * 25,
      phase: Math.random() * Math.PI * 2,
      swaySpeed: 1.5 + Math.random() * 2,
      swayAmplitude: 0.3 + Math.random() * 0.5,
      fallSpeed: FALL_SPEED * (0.7 + Math.random() * 0.6),
    };
  }

  private writeLeaf(i: number, leaf: Leaf): void {
    // Sim Z-up → Three.js Y-up
    this.positions[i * 3] = leaf.x;
    this.positions[i * 3 + 1] = leaf.z;
    this.positions[i * 3 + 2] = leaf.y;
    this.phases[i] = leaf.phase;

    // Pick color from species if trees exist, else fallback palette
    let c: number[];
    if (this.treeSpecies.length > 0) {
      const sid = this.treeSpecies[Math.floor(Math.random() * this.treeSpecies.length)];
      c = SPECIES_LEAF_COLORS[sid] ?? LEAF_COLORS[Math.floor(Math.random() * LEAF_COLORS.length)];
    } else {
      c = LEAF_COLORS[Math.floor(Math.random() * LEAF_COLORS.length)];
    }
    // Slight per-leaf brightness variation
    const v = 0.9 + Math.random() * 0.2;
    this.colors[i * 3] = c[0] * v;
    this.colors[i * 3 + 1] = c[1] * v;
    this.colors[i * 3 + 2] = c[2] * v;
  }

  /** Burst: respawn all leaves at the canopy top for a visible gust surge. */
  emitGustBurst(): void {
    for (let i = 0; i < MAX_LEAVES; i++) {
      const leaf = this.spawnLeaf();
      // Cluster near top of canopy for a visible cascade
      leaf.z = GROUND_LEVEL + 25 + Math.random() * 15;
      leaf.fallSpeed *= 1.3; // fall a bit faster during gust
      this.leaves[i] = leaf;
      this.writeLeaf(i, leaf);
    }
  }

  update(dt: number, elapsedTime: number): void {
    this.timeUniform.value = elapsedTime;

    // No leaves fall if garden has few plants
    if (this.plantCount < 200) {
      for (let i = 0; i < MAX_LEAVES; i++) {
        this.positions[i * 3 + 1] = -1000; // park off-screen
      }
      (this.mesh.geometry.getAttribute('position') as THREE.BufferAttribute).needsUpdate = true;
      return;
    }

    for (let i = 0; i < MAX_LEAVES; i++) {
      const leaf = this.leaves[i];

      // Fall — faster in high wind (rain), slower in low wind (drought)
      const windFactor = 0.5 + this.windStrength;
      leaf.z -= leaf.fallSpeed * windFactor * dt;

      // Sway side to side — stronger in wind
      const swayMul = 0.5 + this.windStrength * 1.5;
      leaf.x += Math.sin(elapsedTime * leaf.swaySpeed + leaf.phase) * leaf.swayAmplitude * swayMul * dt;
      leaf.y += Math.cos(elapsedTime * leaf.swaySpeed * 0.7 + leaf.phase) * leaf.swayAmplitude * 0.5 * swayMul * dt;

      // Wind pushes leaves in the current wind direction during gusts
      leaf.x += this.windDirX * this.windStrength * 0.5 * dt;
      leaf.y += this.windDirZ * this.windStrength * 0.5 * dt;

      // Hit ground — respawn at top
      if (leaf.z <= GROUND_LEVEL) {
        const newLeaf = this.spawnLeaf();
        this.leaves[i] = newLeaf;
        this.writeLeaf(i, newLeaf);
      } else {
        // Update position (Z-up → Y-up)
        this.positions[i * 3] = leaf.x;
        this.positions[i * 3 + 1] = leaf.z;
        this.positions[i * 3 + 2] = leaf.y;
      }
    }

    (this.mesh.geometry.getAttribute('position') as THREE.BufferAttribute).needsUpdate = true;
  }

  dispose(): void {
    this.mesh.geometry.dispose();
    (this.mesh.material as THREE.Material).dispose();
  }
}
