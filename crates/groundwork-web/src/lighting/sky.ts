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
varying vec3 vWorldPosition;
void main() {
  float h = normalize(vWorldPosition).y; // -1 (nadir) to +1 (zenith)

  // Three-band gradient: bottom → horizon → top
  // Horizon band sits at h=0 (eye level), blends with fog for seamless depth
  vec3 color;
  if (h > 0.0) {
    // Above horizon: horizon → sky top
    float t = smoothstep(0.0, 0.6, h);
    color = mix(horizonColor, topColor, t);
  } else {
    // Below horizon: ground bottom → horizon
    float t = smoothstep(-0.5, 0.0, h);
    color = mix(bottomColor, horizonColor, t);
  }

  gl_FragColor = vec4(color, 1.0);
}
`;

export interface SkyUniforms {
  topColor: THREE.IUniform<THREE.Color>;
  bottomColor: THREE.IUniform<THREE.Color>;
  horizonColor: THREE.IUniform<THREE.Color>;
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
