/**
 * Fauna renderer: billboard sprites for ecological creatures.
 *
 * Renders pollinators (bees, butterflies), birds, worms, and beetles as
 * animated billboard sprites. Each fauna type has its own visual identity:
 *
 * - Bees: small golden dots with rapid wing flutter
 * - Butterflies: larger, colorful with gentle wing flap
 * - Birds: dark silhouettes circling above canopy
 * - Worms: pinkish underground with subtle wiggle
 * - Beetles: dark brown on surface near dead wood
 *
 * Uses InstancedMesh for efficient rendering, similar to FoliageRenderer.
 * Positions are read from the WASM fauna export buffer each frame.
 */

import * as THREE from 'three';
import {
  GROUND_LEVEL,
  FaunaType,
  FaunaState,
  getFaunaCount,
  getFaunaView,
  readFauna,
} from '../bridge';

/** Maximum fauna instances (matches Rust MAX_FAUNA) */
const MAX_FAUNA = 128;

/** Fauna vertex shader — billboard with wing flutter animation */
const FAUNA_VERT = /* glsl */ `
  attribute vec3 instanceColor;
  attribute float aFaunaType;
  attribute float aFaunaState;

  uniform float uTime;

  varying vec3 vColor;
  varying vec2 vUv;
  varying float vType;
  varying float vState;

  void main() {
    vColor = instanceColor;
    vUv = uv;
    vType = aFaunaType;
    vState = aFaunaState;

    // Billboard: extract instance position from instance matrix
    vec4 worldPos = instanceMatrix * vec4(0.0, 0.0, 0.0, 1.0);

    // Wing flutter for flying creatures
    float flutter = 0.0;
    if (aFaunaType < 2.0) {
      // Bees and butterflies: rapid wing motion
      float wingSpeed = aFaunaType < 0.5 ? 15.0 : 6.0;
      flutter = sin(uTime * wingSpeed + worldPos.x * 3.0) * 0.15;
    } else if (aFaunaType < 2.5) {
      // Birds: gentle wing soar
      flutter = sin(uTime * 2.0 + worldPos.y * 2.0) * 0.1;
    }

    // Billboard orientation
    vec3 cameraRight = vec3(viewMatrix[0][0], viewMatrix[1][0], viewMatrix[2][0]);
    vec3 cameraUp = vec3(viewMatrix[0][1], viewMatrix[1][1], viewMatrix[2][1]);

    float scale = length(instanceMatrix[0].xyz);

    // Apply wing flutter to horizontal stretch
    float hStretch = 1.0 + flutter;

    vec3 vertexWorld = worldPos.xyz
      + cameraRight * position.x * scale * hStretch
      + cameraUp * position.y * scale;

    gl_Position = projectionMatrix * viewMatrix * vec4(vertexWorld, 1.0);
  }
`;

/** Fauna fragment shader — soft shapes with type-specific patterns */
const FAUNA_FRAG = /* glsl */ `
  varying vec3 vColor;
  varying vec2 vUv;
  varying float vType;
  varying float vState;

  void main() {
    vec2 centered = vUv - 0.5;
    float dist = length(centered) * 2.0;

    float alpha = 0.0;

    if (vType < 0.5) {
      // Bee: small solid circle with bright glow
      alpha = 1.0 - smoothstep(0.3, 0.7, dist);
    } else if (vType < 1.5) {
      // Butterfly: wing shape (wider than tall)
      float wingDist = length(vec2(centered.x * 0.7, centered.y));
      alpha = 1.0 - smoothstep(0.3, 0.6, wingDist * 2.0);
    } else if (vType < 2.5) {
      // Bird: V-shaped silhouette
      float birdShape = abs(centered.y - abs(centered.x) * 0.5);
      alpha = 1.0 - smoothstep(0.05, 0.15, birdShape);
      alpha *= step(dist, 0.9);
    } else if (vType < 3.5) {
      // Worm: elongated oval
      float wormDist = length(vec2(centered.x, centered.y * 2.5));
      alpha = 1.0 - smoothstep(0.2, 0.5, wormDist * 2.0);
    } else {
      // Beetle: compact oval
      float beetleDist = length(vec2(centered.x * 1.2, centered.y));
      alpha = 1.0 - smoothstep(0.25, 0.5, beetleDist * 2.0);
    }

    if (alpha < 0.05) discard;

    // Leaving state: fade out
    if (vState > 2.5) {
      alpha *= 0.5;
    }

    // Acting state: slight glow
    float glow = vState > 1.5 && vState < 2.5 ? 1.2 : 1.0;

    gl_FragColor = vec4(vColor * glow, alpha * 0.9);
  }
`;

/** Fauna type colors */
const FAUNA_COLORS: Record<number, THREE.Color> = {
  [FaunaType.Bee]: new THREE.Color(0.95, 0.80, 0.20),       // golden yellow
  [FaunaType.Butterfly]: new THREE.Color(0.85, 0.45, 0.65),  // warm pink-purple
  [FaunaType.Bird]: new THREE.Color(0.25, 0.22, 0.30),       // dark silhouette
  [FaunaType.Worm]: new THREE.Color(0.75, 0.55, 0.50),       // pinkish brown
  [FaunaType.Beetle]: new THREE.Color(0.35, 0.25, 0.15),     // dark brown
};

/** Fauna sprite sizes */
const FAUNA_SIZES: Record<number, number> = {
  [FaunaType.Bee]: 0.5,
  [FaunaType.Butterfly]: 0.8,
  [FaunaType.Bird]: 1.2,
  [FaunaType.Worm]: 0.4,
  [FaunaType.Beetle]: 0.45,
};

export class FaunaRenderer {
  readonly group: THREE.Group;

  private mesh: THREE.InstancedMesh;
  private material: THREE.ShaderMaterial;
  private colorAttr: THREE.InstancedBufferAttribute;
  private typeAttr: THREE.InstancedBufferAttribute;
  private stateAttr: THREE.InstancedBufferAttribute;
  private dummy = new THREE.Object3D();
  private activeCount = 0;

  constructor() {
    this.group = new THREE.Group();
    this.group.name = 'fauna';

    // Quad geometry for billboard
    const geo = new THREE.PlaneGeometry(1, 1);

    this.material = new THREE.ShaderMaterial({
      vertexShader: FAUNA_VERT,
      fragmentShader: FAUNA_FRAG,
      uniforms: {
        uTime: { value: 0 },
      },
      transparent: true,
      depthWrite: false,
      side: THREE.DoubleSide,
    });

    // Per-instance attributes
    const colorArray = new Float32Array(MAX_FAUNA * 3);
    this.colorAttr = new THREE.InstancedBufferAttribute(colorArray, 3);

    const typeArray = new Float32Array(MAX_FAUNA);
    this.typeAttr = new THREE.InstancedBufferAttribute(typeArray, 1);

    const stateArray = new Float32Array(MAX_FAUNA);
    this.stateAttr = new THREE.InstancedBufferAttribute(stateArray, 1);

    this.mesh = new THREE.InstancedMesh(geo, this.material, MAX_FAUNA);
    this.mesh.instanceColor = null;
    this.mesh.geometry.setAttribute('instanceColor', this.colorAttr);
    this.mesh.geometry.setAttribute('aFaunaType', this.typeAttr);
    this.mesh.geometry.setAttribute('aFaunaState', this.stateAttr);
    this.mesh.frustumCulled = false;
    this.mesh.count = 0;

    this.group.add(this.mesh);
  }

  /**
   * Update fauna positions from the WASM bridge data.
   * Call each frame after sim tick.
   */
  rebuild(): void {
    const count = getFaunaCount();
    const view = getFaunaView();

    if (!view || count === 0) {
      this.mesh.count = 0;
      this.activeCount = 0;
      return;
    }

    const actualCount = Math.min(count, MAX_FAUNA);

    for (let i = 0; i < actualCount; i++) {
      const f = readFauna(view, i);

      // Coordinate swap: sim (x,y,z) Z-up → Three.js (x,z,y) Y-up
      this.dummy.position.set(f.x, f.z, f.y);

      const size = FAUNA_SIZES[f.type] ?? 0.5;
      this.dummy.scale.setScalar(size);
      this.dummy.updateMatrix();
      this.mesh.setMatrixAt(i, this.dummy.matrix);

      // Color with slight per-instance variation
      const baseColor = FAUNA_COLORS[f.type] ?? new THREE.Color(1, 0, 1);
      const variation = 0.9 + (i * 73856093 & 0xff) / 255.0 * 0.2;
      this.colorAttr.setXYZ(
        i,
        baseColor.r * variation,
        baseColor.g * variation,
        baseColor.b * variation,
      );

      // Type and state for shader
      this.typeAttr.setX(i, f.type);
      this.stateAttr.setX(i, f.state);
    }

    this.activeCount = actualCount;
    this.mesh.count = actualCount;
    this.mesh.instanceMatrix.needsUpdate = true;
    this.colorAttr.needsUpdate = true;
    this.typeAttr.needsUpdate = true;
    this.stateAttr.needsUpdate = true;
  }

  /**
   * Update animation time. Call each frame.
   */
  update(elapsedTime: number): void {
    this.material.uniforms.uTime.value = elapsedTime;
    // Rebuild positions every frame (fauna move continuously)
    this.rebuild();
  }

  /** Current active fauna count */
  get count(): number {
    return this.activeCount;
  }

  dispose(): void {
    this.mesh.geometry.dispose();
    this.material.dispose();
  }
}
