/**
 * Dust motes in sunbeams — tiny warm particles that drift through dappled light.
 *
 * Appear during bright daylight hours (day cycle 0.3–0.65), drifting lazily
 * through the air above the garden. They catch the sunlight and glow warm gold,
 * creating visible sunbeam shafts through the canopy.
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GROUND_LEVEL } from '../bridge';

const MAX_MOTES = 50;

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
    // Gentle size pulse
    float pulse = 0.6 + 0.4 * sin(uTime * 1.5 + aPhase * 6.28);
    gl_PointSize = mix(1.0, 3.5, pulse) * aActive * (180.0 / -mvPos.z);
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
    float alpha = (1.0 - dist * dist) * vActive * 0.6;
    // Warm golden dust
    float pulse = 0.5 + 0.5 * sin(uTime * 1.5 + vPhase * 6.28);
    vec3 color = mix(vec3(0.9, 0.8, 0.5), vec3(1.0, 0.95, 0.7), pulse);
    gl_FragColor = vec4(color, alpha);
  }
`;

interface Mote {
  x: number;
  y: number;
  z: number;
  vx: number;
  vy: number;
  vz: number;
  phase: number;
}

export class DustMoteRenderer {
  readonly group: THREE.Group;
  private mesh: THREE.Points;
  private positions: Float32Array;
  private phases: Float32Array;
  private actives: Float32Array;
  private motes: Mote[] = [];
  private timeUniform: { value: number };
  private active = false;

  constructor() {
    this.group = new THREE.Group();
    this.group.name = 'dust-motes';

    this.positions = new Float32Array(MAX_MOTES * 3);
    this.phases = new Float32Array(MAX_MOTES);
    this.actives = new Float32Array(MAX_MOTES);

    for (let i = 0; i < MAX_MOTES; i++) {
      const mote: Mote = {
        x: 15 + Math.random() * (GRID_X - 30),
        y: 15 + Math.random() * (GRID_Y - 30),
        z: GROUND_LEVEL + 3 + Math.random() * 25,
        vx: (Math.random() - 0.5) * 0.1,
        vy: (Math.random() - 0.5) * 0.1,
        vz: (Math.random() - 0.5) * 0.05,
        phase: Math.random(),
      };
      this.motes.push(mote);
      this.positions[i * 3] = mote.x;
      this.positions[i * 3 + 1] = mote.z;
      this.positions[i * 3 + 2] = mote.y;
      this.phases[i] = mote.phase;
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

  /** Set whether dust motes should be active (based on time of day).
   *  Active during bright daylight (0.3–0.65) AND golden hour (0.65–0.80). */
  setActive(dayTime: number): void {
    this.active = dayTime >= 0.3 && dayTime <= 0.80;
  }

  /** Update mote positions. Call each frame. */
  update(dt: number, elapsedTime: number): void {
    this.timeUniform.value = elapsedTime;

    const targetActive = this.active ? 1 : 0;

    for (let i = 0; i < MAX_MOTES; i++) {
      const m = this.motes[i];
      const current = this.actives[i];
      this.actives[i] += (targetActive - current) * Math.min(dt * 0.4, 1);

      if (this.actives[i] < 0.01) continue;

      // Very slow lazy drift
      m.x += m.vx * dt;
      m.y += m.vy * dt;
      m.z += m.vz * dt;

      // Gentle random direction change
      m.vx += (Math.random() - 0.5) * 0.02 * dt;
      m.vy += (Math.random() - 0.5) * 0.02 * dt;
      m.vz += (Math.random() - 0.5) * 0.01 * dt;

      // Damping
      m.vx *= 0.995;
      m.vy *= 0.995;
      m.vz *= 0.995;

      // Keep in bounds
      if (m.x < 10 || m.x > GRID_X - 10) m.vx *= -1;
      if (m.y < 10 || m.y > GRID_Y - 10) m.vy *= -1;
      if (m.z < GROUND_LEVEL + 2) { m.z = GROUND_LEVEL + 2; m.vz = Math.abs(m.vz); }
      if (m.z > GROUND_LEVEL + 30) { m.z = GROUND_LEVEL + 30; m.vz = -Math.abs(m.vz); }

      this.positions[i * 3] = m.x;
      this.positions[i * 3 + 1] = m.z;
      this.positions[i * 3 + 2] = m.y;
    }

    (this.mesh.geometry.getAttribute('position') as THREE.BufferAttribute).needsUpdate = true;
    (this.mesh.geometry.getAttribute('aActive') as THREE.BufferAttribute).needsUpdate = true;
  }

  dispose(): void {
    this.mesh.geometry.dispose();
    (this.mesh.material as THREE.Material).dispose();
  }
}
