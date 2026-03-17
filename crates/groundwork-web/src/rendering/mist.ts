/**
 * Dawn mist: soft, diffuse wisps hovering low over the garden at dawn.
 *
 * Large, slow-drifting translucent particles that fade as the sun rises.
 * Active during early dawn (0.10–0.30 day cycle), creating a distinct
 * morning atmosphere separate from dew sparkles.
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GROUND_LEVEL } from '../bridge';

const MAX_WISPS = 30;

const VERT = /* glsl */ `
  attribute float aActive;
  varying float vActive;
  void main() {
    vActive = aActive;
    vec4 mvPos = modelViewMatrix * vec4(position, 1.0);
    gl_PointSize = aActive * 30.0 * (200.0 / -mvPos.z);
    gl_Position = projectionMatrix * mvPos;
  }
`;

const FRAG = /* glsl */ `
  varying float vActive;
  void main() {
    if (vActive < 0.01) discard;
    float dist = length(gl_PointCoord - 0.5) * 2.0;
    if (dist > 1.0) discard;
    // Very soft gaussian-like falloff
    float alpha = exp(-dist * dist * 3.0) * vActive * 0.3;
    // Cool white-blue mist
    gl_FragColor = vec4(0.8, 0.85, 0.95, alpha);
  }
`;

interface MistWisp {
  x: number;
  y: number;
  z: number;
  vx: number;
  vz: number;
}

export class MistRenderer {
  readonly group: THREE.Group;
  private mesh: THREE.Points;
  private positions: Float32Array;
  private actives: Float32Array;
  private wisps: MistWisp[] = [];
  private active = false;
  private density = 1.0; // 0.3 (dry garden) – 1.5 (lots of water)
  private windDirX = 0;
  private windDirZ = 0;
  private windStrength = 0;

  constructor() {
    this.group = new THREE.Group();
    this.group.name = 'dawn-mist';

    this.positions = new Float32Array(MAX_WISPS * 3);
    this.actives = new Float32Array(MAX_WISPS);

    for (let i = 0; i < MAX_WISPS; i++) {
      const wisp: MistWisp = {
        x: 5 + Math.random() * (GRID_X - 10),
        y: GROUND_LEVEL + 0.5 + Math.random() * 4,
        z: 5 + Math.random() * (GRID_Y - 10),
        vx: (Math.random() - 0.5) * 0.2,
        vz: (Math.random() - 0.5) * 0.2,
      };
      this.wisps.push(wisp);
      this.positions[i * 3] = wisp.x;
      this.positions[i * 3 + 1] = wisp.y;
      this.positions[i * 3 + 2] = wisp.z;
      this.actives[i] = 0;
    }

    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.BufferAttribute(this.positions, 3));
    geo.setAttribute('aActive', new THREE.BufferAttribute(this.actives, 1));

    const mat = new THREE.ShaderMaterial({
      vertexShader: VERT,
      fragmentShader: FRAG,
      transparent: true,
      depthWrite: false,
    });

    this.mesh = new THREE.Points(geo, mat);
    this.mesh.frustumCulled = false;
    this.group.add(this.mesh);
  }

  /** Set whether mist should be active (based on time of day). */
  setActive(dayTime: number): void {
    this.active = dayTime >= 0.10 && dayTime <= 0.30;
  }

  private rainBoost = 0; // decays over time, boosts density after rain

  /** Scale mist density with water volume — wetter gardens get thicker morning mist. */
  setWaterInfluence(waterCount: number): void {
    // 0 water → density 0.3 (thin), 100+ water → density 1.5 (thick)
    this.density = 0.3 + Math.min(1.2, waterCount / 80) + this.rainBoost;
  }

  /** Boost next dawn mist after rain. Call when rain stops. */
  setRainBoost(): void {
    this.rainBoost = 0.8;
  }

  /** Set wind parameters so mist drifts with the wind */
  setWind(windAngle: number, strength: number): void {
    this.windDirX = Math.cos(windAngle);
    this.windDirZ = Math.sin(windAngle);
    this.windStrength = strength;
  }

  /** Update mist drift. Call each frame. */
  update(dt: number): void {
    // Rain boost decays slowly (~60s)
    if (this.rainBoost > 0) this.rainBoost = Math.max(0, this.rainBoost - dt / 60);
    const targetActive = this.active ? this.density : 0;

    for (let i = 0; i < MAX_WISPS; i++) {
      const w = this.wisps[i];

      // Smooth fade in/out (slower than dew — mist lingers)
      this.actives[i] += (targetActive - this.actives[i]) * Math.min(dt * 0.15, 1);

      // Slow drift + wind influence
      w.x += (w.vx + this.windDirX * this.windStrength * 0.4) * dt;
      w.z += (w.vz + this.windDirZ * this.windStrength * 0.4) * dt;

      // Gentle random direction wobble
      w.vx += (Math.random() - 0.5) * 0.02 * dt;
      w.vz += (Math.random() - 0.5) * 0.02 * dt;

      // Keep in bounds
      if (w.x < 0 || w.x > GRID_X) w.vx *= -1;
      if (w.z < 0 || w.z > GRID_Y) w.vz *= -1;

      this.positions[i * 3] = w.x;
      this.positions[i * 3 + 1] = w.y;
      this.positions[i * 3 + 2] = w.z;
    }

    (this.mesh.geometry.getAttribute('position') as THREE.BufferAttribute).needsUpdate = true;
    (this.mesh.geometry.getAttribute('aActive') as THREE.BufferAttribute).needsUpdate = true;
  }

  dispose(): void {
    this.mesh.geometry.dispose();
    (this.mesh.material as THREE.Material).dispose();
  }
}
