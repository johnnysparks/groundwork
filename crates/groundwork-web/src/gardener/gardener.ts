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

    float scale = 4.0; // prominent gnome — visible at default zoom
    vec3 vert = worldPos.xyz
      + camRight * position.x * scale
      + camUp * position.y * scale;

    gl_Position = projectionMatrix * viewMatrix * vec4(vert, 1.0);
  }
`;

/** Gnome fragment shader — draws the gnome shape procedurally */
const GNOME_FRAG = /* glsl */ `
  varying vec2 vUv;

  // SDF helpers
  float sdCircle(vec2 p, vec2 c, float r) {
    return length(p - c) - r;
  }
  float sdEllipse(vec2 p, vec2 c, vec2 r) {
    vec2 d = (p - c) / r;
    return (length(d) - 1.0) * min(r.x, r.y);
  }

  void main() {
    vec2 p = vUv - 0.5;

    // --- Shapes ---

    // Body: squat ellipse (tunic)
    float body = sdEllipse(p, vec2(0.0, -0.04), vec2(0.16, 0.20));
    float bodyAlpha = 1.0 - smoothstep(-0.01, 0.01, body);

    // Head: circle sitting on body
    float head = sdCircle(p, vec2(0.0, 0.14), 0.10);
    float headAlpha = 1.0 - smoothstep(-0.01, 0.01, head);

    // Hat: pointy cone sitting on head
    float hatY = p.y - 0.22;
    float hatX = abs(p.x);
    float hatBase = 0.13;
    float hatHeight = 0.26;
    float hatSdf = hatX - (hatBase - hatY * (hatBase / hatHeight));
    float hatAlpha = step(hatSdf, 0.0) * step(0.0, hatY) * step(hatY, hatHeight);
    // Round the hat tip
    float tipDist = sdCircle(p, vec2(0.0, 0.22 + hatHeight - 0.03), 0.03);
    hatAlpha = max(hatAlpha, 1.0 - smoothstep(-0.01, 0.01, tipDist));

    // Boots: two small ellipses at bottom
    float bootL = sdEllipse(p, vec2(-0.08, -0.24), vec2(0.07, 0.04));
    float bootR = sdEllipse(p, vec2(0.08, -0.24), vec2(0.07, 0.04));
    float bootsAlpha = max(
      1.0 - smoothstep(-0.01, 0.01, bootL),
      1.0 - smoothstep(-0.01, 0.01, bootR)
    );

    // Belt: thin horizontal stripe across body center
    float beltAlpha = bodyAlpha
      * step(-0.06, p.y) * step(p.y, -0.02)
      * (1.0 - step(0.14, abs(p.x)));

    // Belt buckle: tiny square at center
    float buckleAlpha = step(-0.05, p.y) * step(p.y, -0.03)
      * step(abs(p.x), 0.025);

    // --- Combine alpha ---
    float alpha = max(max(max(bodyAlpha, headAlpha), hatAlpha), bootsAlpha);
    if (alpha < 0.1) discard;

    // --- Colors ---
    vec3 tunicColor = vec3(0.32, 0.45, 0.25);   // earthy green tunic
    vec3 hatColor = vec3(0.72, 0.18, 0.12);      // warm red hat
    vec3 skinColor = vec3(0.85, 0.70, 0.55);     // warm skin
    vec3 bootColor = vec3(0.35, 0.22, 0.12);     // dark brown boots
    vec3 beltColor = vec3(0.30, 0.20, 0.10);     // dark leather belt
    vec3 buckleColor = vec3(0.85, 0.75, 0.35);   // gold buckle
    vec3 beardColor = vec3(0.82, 0.78, 0.70);    // warm white beard

    // Layer colors bottom-to-top
    vec3 color = tunicColor;

    // Boots
    color = mix(color, bootColor, bootsAlpha);
    // Head (skin)
    color = mix(color, skinColor, headAlpha);
    // Beard: lower half of head
    float beardMask = headAlpha * step(p.y, 0.12) * step(0.04, p.y);
    color = mix(color, beardColor, beardMask);
    // Eyes: two tiny dark dots
    float eyeL = 1.0 - smoothstep(0.01, 0.02, length(p - vec2(-0.04, 0.16)));
    float eyeR = 1.0 - smoothstep(0.01, 0.02, length(p - vec2(0.04, 0.16)));
    color = mix(color, vec3(0.15, 0.10, 0.08), max(eyeL, eyeR));
    // Nose: tiny bump
    float nose = 1.0 - smoothstep(0.01, 0.02, length(p - vec2(0.0, 0.13)));
    color = mix(color, skinColor * 0.9, nose);
    // Hat over head
    color = mix(color, hatColor, hatAlpha);
    // Hat brim highlight
    float brimMask = hatAlpha * step(0.22, p.y) * step(p.y, 0.25);
    color = mix(color, hatColor * 0.8, brimMask);
    // Belt and buckle over body
    color = mix(color, beltColor, beltAlpha);
    color = mix(color, buckleColor, buckleAlpha * bodyAlpha);

    // Subtle shading: darker at edges
    float shade = 1.0 - length(p) * 0.3;
    color *= shade;

    // Shadow disc on the ground (below feet)
    float shadowDist = sdEllipse(p, vec2(0.0, -0.28), vec2(0.14, 0.03));
    float shadowAlpha = (1.0 - smoothstep(-0.01, 0.02, shadowDist)) * 0.3;

    float finalAlpha = max(alpha * 0.95, shadowAlpha);
    vec3 finalColor = mix(vec3(0.0), color, step(0.1, alpha));
    if (finalAlpha < 0.02) discard;

    gl_FragColor = vec4(finalColor, finalAlpha);
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
