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

/** Warm autumn/green leaf colors */
const LEAF_COLORS = [
  [0.45, 0.55, 0.20], // olive green
  [0.55, 0.65, 0.25], // spring green
  [0.65, 0.50, 0.15], // golden
  [0.70, 0.40, 0.15], // amber
  [0.50, 0.60, 0.20], // fresh green
  [0.40, 0.50, 0.15], // dark green
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
    const c = LEAF_COLORS[Math.floor(Math.random() * LEAF_COLORS.length)];
    this.colors[i * 3] = c[0];
    this.colors[i * 3 + 1] = c[1];
    this.colors[i * 3 + 2] = c[2];
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

      // Fall
      leaf.z -= leaf.fallSpeed * dt;

      // Sway side to side
      leaf.x += Math.sin(elapsedTime * leaf.swaySpeed + leaf.phase) * leaf.swayAmplitude * dt;
      leaf.y += Math.cos(elapsedTime * leaf.swaySpeed * 0.7 + leaf.phase) * leaf.swayAmplitude * 0.5 * dt;

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
