/**
 * Garden gnome billboard sprite.
 *
 * A charming little character that walks to task locations and
 * executes garden work. Uses a simple shader-drawn silhouette
 * (no texture needed — the shape IS the character).
 *
 * Position: reads from task queue, walks toward next task.
 * Animation: waddle bounce while walking, work motion at task.
 */

import * as THREE from 'three';
import { GROUND_LEVEL, GRID_X, GRID_Y } from '../bridge';
import type { TaskQueue, GardenTask } from './queue';

/** Gnome state */
enum GnomeState {
  Idle,
  Walking,
  Working,
}

/** Gnome vertex shader — billboard with waddle bounce */
const GNOME_VERT = /* glsl */ `
  uniform float uTime;
  uniform float uState; // 0=idle, 1=walking, 2=working

  varying vec2 vUv;

  void main() {
    vUv = uv;

    vec4 worldPos = modelMatrix * vec4(0.0, 0.0, 0.0, 1.0);

    // Waddle bounce while walking
    float bounce = 0.0;
    if (uState > 0.5 && uState < 1.5) {
      bounce = abs(sin(uTime * 8.0)) * 0.15;
    }
    // Work bob
    if (uState > 1.5) {
      bounce = sin(uTime * 6.0) * 0.1;
    }

    worldPos.y += bounce;

    // Billboard
    vec3 camRight = vec3(viewMatrix[0][0], viewMatrix[1][0], viewMatrix[2][0]);
    vec3 camUp = vec3(viewMatrix[0][1], viewMatrix[1][1], viewMatrix[2][1]);

    float scale = 2.5; // ~2.5 voxels tall
    vec3 vert = worldPos.xyz
      + camRight * position.x * scale
      + camUp * position.y * scale;

    gl_Position = projectionMatrix * viewMatrix * vec4(vert, 1.0);
  }
`;

/** Gnome fragment shader — draws the gnome shape procedurally */
const GNOME_FRAG = /* glsl */ `
  varying vec2 vUv;

  void main() {
    vec2 p = vUv - 0.5;

    // Body: round bottom half
    float body = length(vec2(p.x * 1.3, (p.y + 0.1) * 1.0));
    float bodyAlpha = 1.0 - smoothstep(0.2, 0.22, body);

    // Hat: triangle top
    float hatY = p.y - 0.15;
    float hatX = abs(p.x);
    float hat = hatX - (0.18 - hatY * 0.6);
    float hatAlpha = step(hat, 0.0) * step(0.0, hatY) * step(hatY, 0.35);

    // Combine
    float alpha = max(bodyAlpha, hatAlpha);
    if (alpha < 0.1) discard;

    // Colors: brown body, red hat, white beard hint
    vec3 bodyColor = vec3(0.45, 0.32, 0.20); // brown tunic
    vec3 hatColor = vec3(0.75, 0.15, 0.10);  // red hat
    vec3 beardColor = vec3(0.85, 0.82, 0.75); // white beard

    // Beard: small white area below center
    float beardMask = step(-0.15, -p.y) * step(p.y, -0.02) * step(abs(p.x), 0.12);

    vec3 color = mix(bodyColor, hatColor, hatAlpha);
    color = mix(color, beardColor, beardMask * bodyAlpha);

    gl_FragColor = vec4(color, alpha * 0.95);
  }
`;

export class GardenerSprite {
  readonly group: THREE.Group;
  private mesh: THREE.Mesh;
  private material: THREE.ShaderMaterial;
  private state = GnomeState.Idle;

  /** Current position in sim coordinates */
  x: number;
  y: number;
  z: number;

  /** Current task being walked toward */
  private currentTask: GardenTask | null = null;
  private workTimer = 0;

  constructor() {
    this.group = new THREE.Group();
    this.group.name = 'gardener';

    // Start at center of garden
    this.x = GRID_X / 2;
    this.y = GRID_Y / 2;
    this.z = GROUND_LEVEL + 1;

    const geo = new THREE.PlaneGeometry(1, 1);
    this.material = new THREE.ShaderMaterial({
      vertexShader: GNOME_VERT,
      fragmentShader: GNOME_FRAG,
      uniforms: {
        uTime: { value: 0 },
        uState: { value: 0 },
      },
      transparent: true,
      depthWrite: false,
      side: THREE.DoubleSide,
    });

    this.mesh = new THREE.Mesh(geo, this.material);
    this.mesh.frustumCulled = false;
    this.group.add(this.mesh);

    this.updatePosition();
  }

  /** Update gnome each frame. Returns a completed task or null. */
  update(dt: number, elapsed: number, queue: TaskQueue): GardenTask | null {
    this.material.uniforms.uTime.value = elapsed;
    this.material.uniforms.uState.value = this.state;

    // Pick task if idle
    if (!this.currentTask && queue.length > 0) {
      this.currentTask = queue.peek();
      this.state = GnomeState.Walking;
    }

    if (!this.currentTask) {
      this.state = GnomeState.Idle;
      this.updatePosition();
      return null;
    }

    const tx = this.currentTask.x;
    const ty = this.currentTask.y;

    if (this.state === GnomeState.Walking) {
      // Walk toward task at 1.5 voxels/tick (~15 voxels/sec at 10 ticks/sec)
      const speed = 8.0 * dt; // voxels per second in real time
      const dx = tx - this.x;
      const dy = ty - this.y;
      const dist = Math.sqrt(dx * dx + dy * dy);

      if (dist < 1.0) {
        // Arrived
        this.x = tx;
        this.y = ty;
        this.state = GnomeState.Working;
        this.workTimer = 0.3; // work for 0.3 seconds
      } else {
        // Move toward
        this.x += (dx / dist) * Math.min(speed, dist);
        this.y += (dy / dist) * Math.min(speed, dist);
      }
    }

    if (this.state === GnomeState.Working) {
      this.workTimer -= dt;
      if (this.workTimer <= 0) {
        // Task complete — dequeue and return it
        const completed = queue.dequeue();
        this.currentTask = null;
        this.state = GnomeState.Idle;
        this.updatePosition();
        return completed;
      }
    }

    this.updatePosition();
    return null;
  }

  private updatePosition(): void {
    // Sim (x,y,z) Z-up → Three.js (x,z,y) Y-up
    this.mesh.position.set(this.x + 0.5, this.z + 1.5, this.y + 0.5);
  }

  dispose(): void {
    this.mesh.geometry.dispose();
    this.material.dispose();
  }
}
