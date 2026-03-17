/**
 * Cloud shadow ground plane — drifting soft shadow patches on the terrain.
 *
 * A large transparent plane just above ground level. Its shader computes
 * the same FBM noise as the sky clouds to create matching shadow patterns
 * that slowly drift across the garden. Uses multiply blending for natural
 * darkening without adding geometry.
 *
 * Weather-driven: shadows appear with clouds, vanish on clear/drought.
 * Night: shadows fade out (no sunlight = no shadows).
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GROUND_LEVEL } from '../bridge';

const SHADOW_VERT = /* glsl */ `
varying vec2 vUv;
void main() {
  vUv = uv;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}
`;

const SHADOW_FRAG = /* glsl */ `
uniform float uTime;
uniform float uCloudDensity;
uniform float uDayAmount;

varying vec2 vUv;

float hash(vec2 p) {
  return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

float vnoise(vec2 p) {
  vec2 i = floor(p);
  vec2 f = fract(p);
  f = f * f * (3.0 - 2.0 * f);
  float a = hash(i);
  float b = hash(i + vec2(1.0, 0.0));
  float c = hash(i + vec2(0.0, 1.0));
  float d = hash(i + vec2(1.0, 1.0));
  return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

float fbm(vec2 p) {
  float f = 0.0;
  f += 0.50 * vnoise(p); p *= 2.03;
  f += 0.25 * vnoise(p); p *= 2.01;
  f += 0.125 * vnoise(p); p *= 2.02;
  f += 0.0625 * vnoise(p);
  return f / 0.9375;
}

void main() {
  // Scale UVs to match sky cloud projection scale
  vec2 cloudUV = vUv * 3.0;
  // Same drift speed as sky clouds for coherence
  cloudUV += vec2(uTime * 0.008, uTime * 0.002);

  float n = fbm(cloudUV);
  float n2 = fbm(cloudUV * 0.7 + vec2(uTime * 0.005, 3.7));

  float shadow = smoothstep(0.50 - uCloudDensity * 0.12, 0.68, n) * 0.7
               + smoothstep(0.45 - uCloudDensity * 0.10, 0.62, n2) * 0.3;

  // Shadow darkness: subtle darkening, not black
  float darkness = shadow * uCloudDensity * uDayAmount * 0.25;

  // Edge fade: soften at plane boundaries
  vec2 edge = smoothstep(vec2(0.0), vec2(0.05), vUv) * smoothstep(vec2(1.0), vec2(0.95), vUv);
  darkness *= edge.x * edge.y;

  gl_FragColor = vec4(0.0, 0.0, 0.0, darkness);
}
`;

export class CloudShadow {
  readonly mesh: THREE.Mesh;
  private material: THREE.ShaderMaterial;

  constructor() {
    this.material = new THREE.ShaderMaterial({
      vertexShader: SHADOW_VERT,
      fragmentShader: SHADOW_FRAG,
      uniforms: {
        uTime: { value: 0.0 },
        uCloudDensity: { value: 0.4 },
        uDayAmount: { value: 1.0 },
      },
      transparent: true,
      depthWrite: false,
    });

    // Plane covers the full garden, sitting just above ground level
    const geo = new THREE.PlaneGeometry(GRID_X, GRID_Y);
    this.mesh = new THREE.Mesh(geo, this.material);
    // Plane in XZ: rotate from XY to XZ (Three.js Y-up)
    this.mesh.rotation.x = -Math.PI / 2;
    this.mesh.position.set(GRID_X / 2, GROUND_LEVEL + 0.05, GRID_Y / 2);
    this.mesh.renderOrder = 5; // after terrain, before particles
    this.mesh.name = 'cloud-shadow';
  }

  update(time: number, cloudDensity: number, dayAmount: number): void {
    this.material.uniforms.uTime.value = time;
    this.material.uniforms.uCloudDensity.value = cloudDensity;
    this.material.uniforms.uDayAmount.value = dayAmount;
  }

  dispose(): void {
    this.mesh.geometry.dispose();
    this.material.dispose();
  }
}
