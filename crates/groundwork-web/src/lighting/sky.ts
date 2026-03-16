/**
 * Sky gradient background — a large sphere with a vertical color gradient.
 * Top color and bottom color are updated each frame by the DayCycle.
 *
 * Uses a custom ShaderMaterial so the gradient is smooth and resolution-independent.
 */

import * as THREE from 'three';

const vertexShader = /* glsl */ `
varying vec3 vWorldPosition;
void main() {
  vec4 worldPos = modelMatrix * vec4(position, 1.0);
  vWorldPosition = worldPos.xyz;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}
`;

const fragmentShader = /* glsl */ `
uniform vec3 topColor;
uniform vec3 bottomColor;
uniform vec3 horizonColor;
uniform float uNightAmount;
varying vec3 vWorldPosition;

// Simple pseudo-random hash for star placement
float hash(vec2 p) {
  return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

void main() {
  vec3 dir = normalize(vWorldPosition);
  float h = dir.y; // -1 (nadir) to +1 (zenith)

  // Three-band gradient: bottom → horizon → top
  vec3 color;
  if (h > 0.0) {
    float t = smoothstep(0.0, 0.6, h);
    color = mix(horizonColor, topColor, t);
  } else {
    float t = smoothstep(-0.5, 0.0, h);
    color = mix(bottomColor, horizonColor, t);
  }

  // Stars: faint points above horizon, visible only at night
  if (h > 0.05 && uNightAmount > 0.0) {
    // Grid-based star field using direction as UV
    vec2 starUV = dir.xz / max(h, 0.01) * 8.0;
    vec2 cell = floor(starUV);
    float starVal = hash(cell);
    // ~5% of cells have a star
    if (starVal > 0.95) {
      // Star brightness varies and twinkles slightly
      vec2 cellCenter = cell + 0.5;
      float dist = length(fract(starUV) - 0.5);
      float star = smoothstep(0.15, 0.0, dist) * (0.5 + 0.5 * starVal);
      // Fade with height (more stars at zenith) and night amount
      float heightFade = smoothstep(0.05, 0.4, h);
      color += vec3(star * uNightAmount * heightFade * 0.7);
    }
  }

  gl_FragColor = vec4(color, 1.0);
}
`;

export interface SkyUniforms {
  topColor: THREE.IUniform<THREE.Color>;
  bottomColor: THREE.IUniform<THREE.Color>;
  horizonColor: THREE.IUniform<THREE.Color>;
  uNightAmount: THREE.IUniform<number>;
  [key: string]: THREE.IUniform<unknown>;
}

/**
 * Create a sky dome (inverted sphere) with a vertical gradient.
 * Returns the mesh and the uniforms so the DayCycle can update colors each frame.
 */
export function createSkyGradient(scene: THREE.Scene): SkyUniforms {
  const uniforms: SkyUniforms = {
    topColor: { value: new THREE.Color(0x5599dd) },      // bright sky blue
    horizonColor: { value: new THREE.Color(0xccddcc) },   // haze — matches fog color
    bottomColor: { value: new THREE.Color(0x3a3228) },    // dark earth (below-horizon, rarely seen)
    uNightAmount: { value: 0.0 },                          // 0=day, 1=full night (drives star visibility)
  };

  const skyMat = new THREE.ShaderMaterial({
    vertexShader,
    fragmentShader,
    uniforms,
    side: THREE.BackSide,
    depthWrite: false,
    fog: false, // Sky dome must not be affected by scene fog
  });

  const skyGeo = new THREE.SphereGeometry(500, 32, 16);
  const skyMesh = new THREE.Mesh(skyGeo, skyMat);
  skyMesh.renderOrder = -1; // render behind everything
  scene.add(skyMesh);

  // Remove the flat background color — the sky dome replaces it
  scene.background = null;

  return uniforms;
}
