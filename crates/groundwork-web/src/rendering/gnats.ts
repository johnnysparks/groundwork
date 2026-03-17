/**
 * Ambient gnats/midges — tiny dark specks that swarm in lazy orbits
 * over dense vegetation during warm daylight hours.
 *
 * Each swarm orbits a center point with slight random perturbation,
 * creating the classic midge-column appearance. Active during daytime
 * (0.2–0.75) and only when foliage count exceeds a threshold.
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GROUND_LEVEL } from '../bridge';

/** Number of individual gnats across all swarms */
const MAX_GNATS = 40;
/** Number of swarm center points (gnats cluster around these) */
const NUM_SWARMS = 5;
/** Gnats per swarm */
const PER_SWARM = MAX_GNATS / NUM_SWARMS;

const VERT = /* glsl */ `
  attribute float aActive;
  varying float vActive;
  void main() {
    vActive = aActive;
    vec4 mvPos = modelViewMatrix * vec4(position, 1.0);
    gl_PointSize = aActive * 2.0 * (150.0 / -mvPos.z);
    gl_Position = projectionMatrix * mvPos;
  }
`;

const FRAG = /* glsl */ `
  varying float vActive;
  void main() {
    if (vActive < 0.01) discard;
    float dist = length(gl_PointCoord - 0.5) * 2.0;
    if (dist > 1.0) discard;
    float alpha = (1.0 - dist) * vActive * 0.7;
    // Dark specks — gnats are silhouettes against the sky
    gl_FragColor = vec4(0.15, 0.12, 0.1, alpha);
  }
`;

interface Gnat {
  /** Orbit angle around swarm center */
  angle: number;
  /** Orbit radius */
  radius: number;
  /** Vertical offset from swarm center */
  heightOff: number;
  /** Orbit speed (rad/sec) */
  speed: number;
  /** Vertical bob speed */
  bobSpeed: number;
  /** Which swarm this gnat belongs to */
  swarm: number;
}

interface SwarmCenter {
  x: number;
  y: number;
  z: number;
}

export class GnatRenderer {
  readonly group: THREE.Group;
  private mesh: THREE.Points;
  private positions: Float32Array;
  private actives: Float32Array;
  private gnats: Gnat[] = [];
  private swarmCenters: SwarmCenter[] = [];
  private active = false;
  private foliageThreshold = false;
  private relocateTimer = 0;

  constructor() {
    this.group = new THREE.Group();
    this.group.name = 'gnats';

    this.positions = new Float32Array(MAX_GNATS * 3);
    this.actives = new Float32Array(MAX_GNATS);

    // Initialize swarm centers at random garden positions
    for (let s = 0; s < NUM_SWARMS; s++) {
      this.swarmCenters.push({
        x: 20 + Math.random() * (GRID_X - 40),
        y: 20 + Math.random() * (GRID_Y - 40),
        z: GROUND_LEVEL + 4 + Math.random() * 6,
      });
    }

    // Create gnats distributed across swarms
    for (let i = 0; i < MAX_GNATS; i++) {
      const swarm = Math.floor(i / PER_SWARM);
      this.gnats.push({
        angle: Math.random() * Math.PI * 2,
        radius: 0.3 + Math.random() * 1.2,
        heightOff: (Math.random() - 0.5) * 2.0,
        speed: 1.5 + Math.random() * 3.0,
        bobSpeed: 2.0 + Math.random() * 2.0,
        swarm,
      });
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

  /** Set day-cycle visibility. Active during warm daylight (0.2–0.75). */
  setActive(dayTime: number): void {
    this.active = dayTime >= 0.2 && dayTime <= 0.75;
  }

  /** Set foliage threshold — gnats only appear over sufficiently green gardens. */
  setFoliageCount(count: number): void {
    this.foliageThreshold = count >= 200;
  }

  /** Update gnat positions. Call each frame. */
  update(dt: number, elapsedTime: number): void {
    const shouldShow = this.active && this.foliageThreshold;
    const targetActive = shouldShow ? 1 : 0;

    // Slowly relocate swarm centers to keep things dynamic
    this.relocateTimer += dt;
    if (this.relocateTimer > 15) {
      this.relocateTimer = 0;
      const s = Math.floor(Math.random() * NUM_SWARMS);
      this.swarmCenters[s].x = 20 + Math.random() * (GRID_X - 40);
      this.swarmCenters[s].y = 20 + Math.random() * (GRID_Y - 40);
      this.swarmCenters[s].z = GROUND_LEVEL + 4 + Math.random() * 6;
    }

    for (let i = 0; i < MAX_GNATS; i++) {
      const g = this.gnats[i];
      const current = this.actives[i];
      this.actives[i] += (targetActive - current) * Math.min(dt * 0.5, 1);

      if (this.actives[i] < 0.01) continue;

      // Advance orbit
      g.angle += g.speed * dt;

      const center = this.swarmCenters[g.swarm];

      // Orbit position + random jitter
      const x = center.x + Math.cos(g.angle) * g.radius;
      const y = center.y + Math.sin(g.angle) * g.radius;
      const z = center.z + g.heightOff + Math.sin(elapsedTime * g.bobSpeed + i) * 0.5;

      this.positions[i * 3] = x;
      this.positions[i * 3 + 1] = z; // Three.js Y-up
      this.positions[i * 3 + 2] = y;
    }

    (this.mesh.geometry.getAttribute('position') as THREE.BufferAttribute).needsUpdate = true;
    (this.mesh.geometry.getAttribute('aActive') as THREE.BufferAttribute).needsUpdate = true;
  }

  dispose(): void {
    this.mesh.geometry.dispose();
    (this.mesh.material as THREE.Material).dispose();
  }
}
