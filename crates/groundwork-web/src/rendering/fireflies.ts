/**
 * Firefly particle system — soft golden lights that drift during dusk/night.
 *
 * Fireflies appear during golden hour and blue hour (day cycle time ~0.65–0.05),
 * drifting lazily near ground level. They pulse/blink with a soft warm glow,
 * making the idle garden feel like a living painting.
 *
 * Ecologically meaningful: fireflies indicate a healthy, thriving ecosystem.
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GROUND_LEVEL } from '../bridge';

const MAX_FIREFLIES = 40;

/** Firefly vertex shader — point sprites with pulsing glow */
const VERT = /* glsl */ `
  attribute float aPhase;
  attribute float aActive;
  varying float vPhase;
  varying float vActive;
  uniform float uTime;
  void main() {
    vPhase = aPhase;
    vActive = aActive;
    vec4 mvPos = modelViewMatrix * vec4(position, 1.0);
    // Pulse size with phase
    float pulse = 0.5 + 0.5 * sin(uTime * 2.0 + aPhase * 6.28);
    gl_PointSize = mix(2.0, 6.0, pulse) * aActive * (250.0 / -mvPos.z);
    gl_Position = projectionMatrix * mvPos;
  }
`;

const FRAG = /* glsl */ `
  varying float vPhase;
  varying float vActive;
  uniform float uTime;
  void main() {
    if (vActive < 0.01) discard;
    float dist = length(gl_PointCoord - 0.5) * 2.0;
    if (dist > 1.0) discard;
    // Soft radial falloff
    float alpha = (1.0 - dist * dist) * vActive;
    // Warm pulse: golden-green to warm yellow
    float pulse = 0.5 + 0.5 * sin(uTime * 2.0 + vPhase * 6.28);
    vec3 color = mix(vec3(0.6, 0.8, 0.2), vec3(1.0, 0.9, 0.4), pulse);
    gl_FragColor = vec4(color, alpha * 0.9);
  }
`;

interface Firefly {
  x: number;
  y: number;
  z: number;
  vx: number;
  vy: number;
  vz: number;
  phase: number;
  blinkTimer: number;
  blinkOn: boolean;
}

export class FireflyRenderer {
  readonly group: THREE.Group;
  private mesh: THREE.Points;
  private positions: Float32Array;
  private phases: Float32Array;
  private actives: Float32Array;
  private flies: Firefly[] = [];
  private timeUniform: { value: number };
  private active = false;

  constructor() {
    this.group = new THREE.Group();
    this.group.name = 'fireflies';

    this.positions = new Float32Array(MAX_FIREFLIES * 3);
    this.phases = new Float32Array(MAX_FIREFLIES);
    this.actives = new Float32Array(MAX_FIREFLIES);

    // Initialize fireflies with random positions in the garden
    for (let i = 0; i < MAX_FIREFLIES; i++) {
      this.flies.push({
        x: 20 + Math.random() * (GRID_X - 40),
        y: 20 + Math.random() * (GRID_Y - 40),
        z: GROUND_LEVEL + 2 + Math.random() * 10,
        vx: (Math.random() - 0.5) * 0.3,
        vy: (Math.random() - 0.5) * 0.3,
        vz: (Math.random() - 0.5) * 0.15,
        phase: Math.random(),
        blinkTimer: Math.random() * 4,
        blinkOn: Math.random() > 0.5,
      });
      this.positions[i * 3] = this.flies[i].x;
      this.positions[i * 3 + 1] = this.flies[i].z; // Y-up
      this.positions[i * 3 + 2] = this.flies[i].y;
      this.phases[i] = this.flies[i].phase;
      this.actives[i] = 0;
    }

    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.BufferAttribute(this.positions, 3));
    geo.setAttribute('aPhase', new THREE.BufferAttribute(this.phases, 1));
    geo.setAttribute('aActive', new THREE.BufferAttribute(this.actives, 1));

    this.timeUniform = { value: 0 };

    const mat = new THREE.ShaderMaterial({
      vertexShader: VERT,
      fragmentShader: FRAG,
      transparent: true,
      depthWrite: false,
      blending: THREE.AdditiveBlending,
      uniforms: { uTime: this.timeUniform },
    });

    this.mesh = new THREE.Points(geo, mat);
    this.mesh.frustumCulled = false;
    this.group.add(this.mesh);
  }

  /** Set whether fireflies should be active (based on time of day). */
  setActive(dayTime: number): void {
    // Active during golden hour → blue hour (0.65 – 0.05 wrapping through midnight)
    this.active = dayTime >= 0.65 || dayTime < 0.05;
  }

  /** Update firefly positions and blinking. Call each frame. */
  update(dt: number, elapsedTime: number): void {
    this.timeUniform.value = elapsedTime;

    const targetActive = this.active ? 1 : 0;

    for (let i = 0; i < MAX_FIREFLIES; i++) {
      const f = this.flies[i];

      // Fade in/out
      const current = this.actives[i];
      this.actives[i] += (targetActive - current) * Math.min(dt * 0.5, 1);

      if (this.actives[i] < 0.01) continue;

      // Blink cycle: on for 1-3s, off for 0.5-2s
      f.blinkTimer -= dt;
      if (f.blinkTimer <= 0) {
        f.blinkOn = !f.blinkOn;
        f.blinkTimer = f.blinkOn ? 1 + Math.random() * 2 : 0.5 + Math.random() * 1.5;
      }

      // Dim when blink is off
      if (!f.blinkOn) {
        this.actives[i] *= 0.15;
      }

      // Lazy drift
      f.x += f.vx * dt;
      f.y += f.vy * dt;
      f.z += f.vz * dt;

      // Gentle random direction change
      f.vx += (Math.random() - 0.5) * 0.1 * dt;
      f.vy += (Math.random() - 0.5) * 0.1 * dt;
      f.vz += (Math.random() - 0.5) * 0.05 * dt;

      // Damping
      f.vx *= 0.99;
      f.vy *= 0.99;
      f.vz *= 0.99;

      // Keep in bounds
      if (f.x < 15 || f.x > GRID_X - 15) f.vx *= -1;
      if (f.y < 15 || f.y > GRID_Y - 15) f.vy *= -1;
      if (f.z < GROUND_LEVEL + 1) { f.z = GROUND_LEVEL + 1; f.vz = Math.abs(f.vz); }
      if (f.z > GROUND_LEVEL + 15) { f.z = GROUND_LEVEL + 15; f.vz = -Math.abs(f.vz); }

      // Write to buffer (sim Z-up → Three.js Y-up)
      this.positions[i * 3] = f.x;
      this.positions[i * 3 + 1] = f.z;
      this.positions[i * 3 + 2] = f.y;
    }

    (this.mesh.geometry.getAttribute('position') as THREE.BufferAttribute).needsUpdate = true;
    (this.mesh.geometry.getAttribute('aActive') as THREE.BufferAttribute).needsUpdate = true;
  }

  dispose(): void {
    this.mesh.geometry.dispose();
    (this.mesh.material as THREE.Material).dispose();
  }
}
