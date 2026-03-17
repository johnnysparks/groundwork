/**
 * Rain particle renderer: falling droplets during Rain weather state.
 *
 * Uses a pooled Points geometry with a simple shader. Droplets fall from
 * the sky, hit the ground, and respawn at the top. Only active during
 * WeatherState.Rain — idle otherwise.
 *
 * The effect is cozy and gentle: soft streaks, not a storm. The garden
 * gets a visible "it's raining" event that makes weather feel alive.
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GROUND_LEVEL } from '../bridge';

/** Max rain droplets */
const MAX_DROPS = 800;

/** Rain area: covers the garden bed with some margin */
const RAIN_MIN_X = -10;
const RAIN_MAX_X = GRID_X + 10;
const RAIN_MIN_Z = -10;  // Three.js Z = sim Y
const RAIN_MAX_Z = GRID_Y + 10;
const RAIN_MIN_Y = GROUND_LEVEL;      // ground level (Three.js Y = sim Z)
const RAIN_MAX_Y = GROUND_LEVEL + 40; // sky spawn height

/** Fall speed (voxels per second) */
const FALL_SPEED = 60;

const RAIN_VERT = /* glsl */ `
  attribute float aLife;

  varying float vLife;

  void main() {
    vLife = aLife;
    vec4 mvPos = modelViewMatrix * vec4(position, 1.0);
    // Elongated drop: size scales with life (full when alive)
    gl_PointSize = mix(1.0, 3.0, aLife);
    gl_Position = projectionMatrix * mvPos;
  }
`;

const RAIN_FRAG = /* glsl */ `
  uniform vec3 uColor;

  varying float vLife;

  void main() {
    // Soft elongated streak
    vec2 uv = gl_PointCoord - 0.5;
    float dist = length(uv) * 2.0;
    if (dist > 1.0) discard;
    float alpha = (1.0 - dist) * vLife * 0.4;

    gl_FragColor = vec4(uColor, alpha);
  }
`;

interface Raindrop {
  x: number;
  y: number;
  z: number;
  speed: number;
}

export class RainRenderer {
  readonly points: THREE.Points;

  private drops: Raindrop[];
  private positions: Float32Array;
  private lifes: Float32Array;
  private geometry: THREE.BufferGeometry;
  private material: THREE.ShaderMaterial;
  private active = false;
  private intensity = 0;  // 0-1 smooth ramp
  private windDirX = 0;
  private windDirZ = 0;
  private windStrength = 0;
  private splashCallback: ((x: number, y: number, z: number) => void) | null = null;

  constructor() {
    this.drops = new Array(MAX_DROPS);
    for (let i = 0; i < MAX_DROPS; i++) {
      this.drops[i] = { x: 0, y: -1000, z: 0, speed: FALL_SPEED };
    }

    this.positions = new Float32Array(MAX_DROPS * 3);
    this.lifes = new Float32Array(MAX_DROPS);

    this.geometry = new THREE.BufferGeometry();
    this.geometry.setAttribute('position', new THREE.BufferAttribute(this.positions, 3));
    this.geometry.setAttribute('aLife', new THREE.BufferAttribute(this.lifes, 1));

    this.material = new THREE.ShaderMaterial({
      vertexShader: RAIN_VERT,
      fragmentShader: RAIN_FRAG,
      uniforms: {
        uColor: { value: new THREE.Color(0.7, 0.8, 0.9) }, // soft blue-white
      },
      transparent: true,
      depthWrite: false,
    });

    this.points = new THREE.Points(this.geometry, this.material);
    this.points.name = 'rain';
    this.points.frustumCulled = false;
    this.points.visible = false;
  }

  /** Register a callback for splash particles at raindrop impact points. */
  onSplash(cb: (x: number, y: number, z: number) => void): void {
    this.splashCallback = cb;
  }

  /** Set whether rain is active (call when weather state changes) */
  setActive(raining: boolean): void {
    if (raining && !this.active) {
      // Starting rain: initialize drops
      for (let i = 0; i < MAX_DROPS; i++) {
        this.respawn(this.drops[i]);
        this.drops[i].y = RAIN_MIN_Y + Math.random() * (RAIN_MAX_Y - RAIN_MIN_Y);
      }
    }
    this.active = raining;
  }

  /** Set wind parameters for angled rain during gusts */
  setWind(windAngle: number, strength: number): void {
    this.windDirX = Math.cos(windAngle);
    this.windDirZ = Math.sin(windAngle);
    this.windStrength = strength;
  }

  /** Current rain intensity (0-1), smooth-ramped */
  getIntensity(): number {
    return this.intensity;
  }

  /** Update rain simulation. Call each frame with delta time. */
  update(dt: number): void {
    // Smooth intensity ramp: 3s ramp-up, 2s fade-down
    const target = this.active ? 1 : 0;
    const rampSpeed = this.active ? dt / 3.0 : dt / 2.0;
    this.intensity += (target - this.intensity) * Math.min(rampSpeed * 3, 1);
    this.intensity = Math.max(0, Math.min(1, this.intensity));

    // Show mesh if any intensity
    this.points.visible = this.intensity > 0.01;
    if (!this.points.visible) return;

    // Number of active drops scales with intensity
    const activeCount = Math.floor(MAX_DROPS * this.intensity);

    for (let i = 0; i < MAX_DROPS; i++) {
      const drop = this.drops[i];

      if (i < activeCount) {
        // Fall + wind-driven horizontal drift
        drop.y -= drop.speed * dt;
        const drift = this.windStrength * 15 * dt; // stronger wind = more angled rain
        drop.x += this.windDirX * drift;
        drop.z += this.windDirZ * drift;

        // Hit the ground? Emit splash and respawn
        if (drop.y < RAIN_MIN_Y) {
          if (this.splashCallback && Math.random() < 0.08) {
            this.splashCallback(drop.x, RAIN_MIN_Y, drop.z);
          }
          this.respawn(drop);
        }

        // Life = how far from ground, scaled by intensity for opacity fade
        const life = (drop.y - RAIN_MIN_Y) / (RAIN_MAX_Y - RAIN_MIN_Y);
        this.positions[i * 3] = drop.x;
        this.positions[i * 3 + 1] = drop.y;
        this.positions[i * 3 + 2] = drop.z;
        this.lifes[i] = Math.max(0, life) * this.intensity;
      } else {
        // Inactive drops: park offscreen
        this.positions[i * 3 + 1] = -1000;
        this.lifes[i] = 0;
      }
    }

    const posAttr = this.geometry.getAttribute('position') as THREE.BufferAttribute;
    const lifeAttr = this.geometry.getAttribute('aLife') as THREE.BufferAttribute;
    posAttr.needsUpdate = true;
    lifeAttr.needsUpdate = true;
  }

  private respawn(drop: Raindrop): void {
    drop.x = RAIN_MIN_X + Math.random() * (RAIN_MAX_X - RAIN_MIN_X);
    drop.z = RAIN_MIN_Z + Math.random() * (RAIN_MAX_Z - RAIN_MIN_Z);
    drop.y = RAIN_MAX_Y + Math.random() * 10; // slight overshoot for stagger
    drop.speed = FALL_SPEED * (0.8 + Math.random() * 0.4); // vary speed
  }

  dispose(): void {
    this.geometry.dispose();
    this.material.dispose();
  }
}
