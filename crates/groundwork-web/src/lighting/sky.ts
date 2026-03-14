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
varying vec3 vWorldPosition;
void main() {
  // Normalize height to 0..1 (hemisphere: bottom at y=0, top at y=1 on unit sphere)
  float h = normalize(vWorldPosition).z * 0.5 + 0.5;
  // Smooth step for a softer blend near the horizon
  h = smoothstep(0.0, 1.0, h);
  gl_FragColor = vec4(mix(bottomColor, topColor, h), 1.0);
}
`;

export interface SkyUniforms {
  topColor: THREE.IUniform<THREE.Color>;
  bottomColor: THREE.IUniform<THREE.Color>;
  [key: string]: THREE.IUniform<unknown>;
}

/**
 * Create a sky dome (inverted sphere) with a vertical gradient.
 * Returns the mesh and the uniforms so the DayCycle can update colors each frame.
 */
export function createSkyGradient(scene: THREE.Scene): SkyUniforms {
  const uniforms: SkyUniforms = {
    topColor: { value: new THREE.Color(0x5577aa) },    // golden hour default
    bottomColor: { value: new THREE.Color(0xffddbb) },
  };

  const skyMat = new THREE.ShaderMaterial({
    vertexShader,
    fragmentShader,
    uniforms,
    side: THREE.BackSide,
    depthWrite: false,
  });

  const skyGeo = new THREE.SphereGeometry(400, 32, 16);
  const skyMesh = new THREE.Mesh(skyGeo, skyMat);
  skyMesh.renderOrder = -1; // render behind everything
  scene.add(skyMesh);

  // Remove the flat background color — the sky dome replaces it
  scene.background = null;

  return uniforms;
}
