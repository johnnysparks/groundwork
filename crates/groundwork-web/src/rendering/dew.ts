/**
 * Morning dew sparkle system — tiny white-blue sparkles on foliage at dawn.
 *
 * Dew appears during dawn (day cycle 0.15–0.35), twinkling on leaf surfaces
 * as the first light catches water droplets. Creates a magical, cozy dawn
 * atmosphere. Fades as the sun rises and evaporates the dew.
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GROUND_LEVEL } from '../bridge';

const MAX_DEW = 60;

/** Dew vertex shader — point sprites with twinkling shimmer */
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
    // Twinkle: sharp on/off sparkle
    float twinkle = step(0.7, sin(uTime * 4.0 + aPhase * 6.28));
    gl_PointSize = mix(1.0, 5.0, twinkle) * aActive * (200.0 / -mvPos.z);
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
    float alpha = (1.0 - dist * dist) * vActive;
    // Cool white-blue with slight shimmer
    float twinkle = step(0.7, sin(uTime * 4.0 + vPhase * 6.28));
    vec3 color = mix(vec3(0.7, 0.85, 1.0), vec3(1.0, 1.0, 1.0), twinkle);
    gl_FragColor = vec4(color, alpha * 0.8);
  }
`;

export class DewRenderer {
  readonly group: THREE.Group;
  private mesh: THREE.Points;
  private positions: Float32Array;
  private phases: Float32Array;
  private actives: Float32Array;
  private timeUniform: { value: number };
  private active = false;

  constructor() {
    this.group = new THREE.Group();
    this.group.name = 'morning-dew';

    this.positions = new Float32Array(MAX_DEW * 3);
    this.phases = new Float32Array(MAX_DEW);
    this.actives = new Float32Array(MAX_DEW);

    // Scatter dew drops across garden canopy area
    for (let i = 0; i < MAX_DEW; i++) {
      const x = 10 + Math.random() * (GRID_X - 20);
      const y = 10 + Math.random() * (GRID_Y - 20);
      const z = GROUND_LEVEL + 1 + Math.random() * 20; // on foliage
      this.positions[i * 3] = x;
      this.positions[i * 3 + 1] = z; // Y-up
      this.positions[i * 3 + 2] = y;
      this.phases[i] = Math.random();
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

  /** Set whether dew should be active (based on time of day). */
  setActive(dayTime: number): void {
    // Active during pre-dawn → early morning (0.15 – 0.35)
    this.active = dayTime >= 0.15 && dayTime <= 0.35;
  }

  /** Update dew shimmer. Call each frame. */
  update(dt: number, elapsedTime: number): void {
    this.timeUniform.value = elapsedTime;

    const targetActive = this.active ? 1 : 0;

    for (let i = 0; i < MAX_DEW; i++) {
      // Smooth fade in/out
      const current = this.actives[i];
      this.actives[i] += (targetActive - current) * Math.min(dt * 0.3, 1);
    }

    (this.mesh.geometry.getAttribute('aActive') as THREE.BufferAttribute).needsUpdate = true;
  }

  dispose(): void {
    this.mesh.geometry.dispose();
    (this.mesh.material as THREE.Material).dispose();
  }
}
