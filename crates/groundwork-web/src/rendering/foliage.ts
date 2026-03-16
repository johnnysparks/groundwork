/**
 * Foliage billboard renderer with wind sway animation.
 *
 * Replaces Leaf voxel cubes with billboard sprites that always face the
 * camera and gently sway in simulated wind. Creates a lush, organic look
 * instead of blocky cubes.
 *
 * Uses InstancedMesh for efficient rendering of many leaf sprites.
 * Custom ShaderMaterial provides:
 * - Camera-facing billboard rotation
 * - Sine-wave wind sway (offset by world position for natural variation)
 * - Soft circular alpha cutout
 * - Per-instance color variation for visual richness
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GRID_Z, VOXEL_BYTES, Material, GROUND_LEVEL, materialIsFoliage as isFoliage } from '../bridge';

/** Maximum number of foliage instances (pre-allocated) */
const MAX_FOLIAGE = 50_000;

/** Foliage sprite size (larger than a voxel for lush canopy overlap) */
const SPRITE_SIZE = 1.4;

/** Wind sway vertex shader */
const FOLIAGE_VERT = /* glsl */ `
  attribute vec3 instanceColor;

  uniform float uTime;
  uniform float uWindStrength;

  varying vec3 vColor;
  varying vec2 vUv;

  void main() {
    vColor = instanceColor;
    vUv = uv;

    // Billboard: extract instance position from instance matrix
    vec4 worldPos = instanceMatrix * vec4(0.0, 0.0, 0.0, 1.0);

    // Wind sway: sine wave offset by world position for natural variation
    // Height above ground increases sway amplitude (Three.js Y = up)
    float heightFactor = max(0.0, (worldPos.y - ${GROUND_LEVEL.toFixed(1)}) * 0.04);
    float swayX = sin(uTime * 1.2 + worldPos.x * 0.7 + worldPos.z * 0.3) * uWindStrength * heightFactor;
    float swayY = cos(uTime * 0.9 + worldPos.z * 0.5 + worldPos.x * 0.4) * uWindStrength * heightFactor * 0.6;
    float swayZ = sin(uTime * 0.7 + worldPos.x * 0.3 + worldPos.z * 0.6) * uWindStrength * heightFactor * 0.2;

    // Billboard: orient quad to face camera
    // Keep local vertex position (quad corners), but orient in camera space
    vec3 cameraRight = vec3(viewMatrix[0][0], viewMatrix[1][0], viewMatrix[2][0]);
    vec3 cameraUp = vec3(viewMatrix[0][1], viewMatrix[1][1], viewMatrix[2][1]);

    // Scale from instance matrix (uniform scale assumed)
    float scale = length(instanceMatrix[0].xyz);

    vec3 vertexWorld = worldPos.xyz
      + cameraRight * position.x * scale
      + cameraUp * position.y * scale
      + vec3(swayX, swayY, swayZ);

    gl_Position = projectionMatrix * viewMatrix * vec4(vertexWorld, 1.0);
  }
`;

/** Foliage fragment shader with soft circular cutout */
const FOLIAGE_FRAG = /* glsl */ `
  varying vec3 vColor;
  varying vec2 vUv;

  void main() {
    // Soft circular alpha — creates a leaf-like blob instead of hard square
    float dist = length(vUv - 0.5) * 2.0;
    float alpha = 1.0 - smoothstep(0.5, 1.0, dist);
    if (alpha < 0.05) discard;

    // Slight shading: darker toward edges
    float shade = 1.0 - dist * 0.2;

    gl_FragColor = vec4(vColor * shade, alpha * 0.96);
  }
`;

/** Base leaf colors with warm green variation */
const LEAF_COLORS = [
  new THREE.Color(0.30, 0.55, 0.20), // warm green (base)
  new THREE.Color(0.35, 0.58, 0.22), // slightly lighter
  new THREE.Color(0.25, 0.50, 0.18), // slightly darker
  new THREE.Color(0.32, 0.52, 0.25), // yellow-green tint
  new THREE.Color(0.28, 0.48, 0.22), // deeper green
];

/** Species-specific foliage color palettes.
 *  Index matches species_id stored in voxel nutrient_level byte.
 *  0=Oak, 1=Birch, 2=Willow, 3=Pine, 4=Fern, 5=Berry Bush, 6=Holly,
 *  7=Wildflower, 8=Daisy, 9=Moss, 10=Grass, 11=Clover
 *
 *  Colors are pushed wide apart so each species reads as visually
 *  distinct even under warm golden-hour lighting. Hue varies from
 *  warm olive (oak) through cool sage (willow) to vibrant emerald (fern). */
const SPECIES_FOLIAGE: THREE.Color[] = [
  new THREE.Color(0.22, 0.48, 0.12),  // Oak: deep warm forest green
  new THREE.Color(0.48, 0.72, 0.22),  // Birch: bright spring yellow-green
  new THREE.Color(0.25, 0.52, 0.38),  // Willow: silver-sage, distinctly blue
  new THREE.Color(0.10, 0.30, 0.18),  // Pine: very dark blue-green
  new THREE.Color(0.16, 0.62, 0.35),  // Fern: vibrant emerald
  new THREE.Color(0.38, 0.52, 0.16),  // Berry Bush: warm olive
  new THREE.Color(0.12, 0.38, 0.12),  // Holly: dark pure green
  new THREE.Color(0.65, 0.45, 0.55),  // Wildflower: pink-purple
  new THREE.Color(0.72, 0.68, 0.28),  // Daisy: warm yellow
  new THREE.Color(0.18, 0.40, 0.16),  // Moss: dark muted olive
  new THREE.Color(0.30, 0.65, 0.14),  // Grass: bright fresh green
  new THREE.Color(0.38, 0.60, 0.18),  // Clover: yellow-green
];

export class FoliageRenderer {
  readonly group: THREE.Group;

  private mesh: THREE.InstancedMesh;
  private material: THREE.ShaderMaterial;
  private colorAttr: THREE.InstancedBufferAttribute;
  private instanceCount = 0;
  private dummy = new THREE.Object3D();

  constructor() {
    this.group = new THREE.Group();
    this.group.name = 'foliage';

    // Quad geometry for billboard sprite
    const geo = new THREE.PlaneGeometry(SPRITE_SIZE, SPRITE_SIZE);

    // Custom shader material with wind sway + billboard
    this.material = new THREE.ShaderMaterial({
      vertexShader: FOLIAGE_VERT,
      fragmentShader: FOLIAGE_FRAG,
      uniforms: {
        uTime: { value: 0 },
        uWindStrength: { value: 0.35 },
      },
      transparent: true,
      depthWrite: false,
      side: THREE.DoubleSide,
    });

    // Per-instance colors
    const colorArray = new Float32Array(MAX_FOLIAGE * 3);
    this.colorAttr = new THREE.InstancedBufferAttribute(colorArray, 3);

    this.mesh = new THREE.InstancedMesh(geo, this.material, MAX_FOLIAGE);
    this.mesh.instanceColor = null; // we use custom attribute instead
    this.mesh.geometry.setAttribute('instanceColor', this.colorAttr);
    this.mesh.frustumCulled = false; // billboards change size, let them all render
    this.mesh.count = 0;

    this.group.add(this.mesh);
  }

  /**
   * Scan the voxel grid and build foliage instances for all Leaf voxels.
   * Call after grid changes (tick, tool placement).
   */
  rebuild(grid: Uint8Array): void {
    let count = 0;

    for (let z = 0; z < GRID_Z && count < MAX_FOLIAGE; z++) {
      for (let y = 0; y < GRID_Y && count < MAX_FOLIAGE; y++) {
        for (let x = 0; x < GRID_X && count < MAX_FOLIAGE; x++) {
          const idx = (x + y * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
          const mat = grid[idx];

          if (!isFoliage(mat)) continue;

          // Position at voxel center (sim Y↔Z swap: sim Z=up → Three.js Y=up)
          this.dummy.position.set(x + 0.5, z + 0.5, y + 0.5);

          // Count neighboring leaf voxels for density-based sizing.
          // Interior canopy leaves (many neighbors) get larger → cohesive mass.
          // Edge leaves (few neighbors) stay smaller → natural fringe.
          let neighbors = 0;
          for (let dz = -1; dz <= 1; dz++) {
            for (let dy = -1; dy <= 1; dy++) {
              for (let dx = -1; dx <= 1; dx++) {
                if (dx === 0 && dy === 0 && dz === 0) continue;
                const nx = x + dx, ny = y + dy, nz = z + dz;
                if (nx >= 0 && nx < GRID_X && ny >= 0 && ny < GRID_Y && nz >= 0 && nz < GRID_Z) {
                  const ni = (nx + ny * GRID_X + nz * GRID_X * GRID_Y) * VOXEL_BYTES;
                  if (isFoliage(grid[ni])) neighbors++;
                }
              }
            }
          }
          // Scale: edge leaves 0.8, dense interior 1.4 — plus hash noise
          const hash = (x * 73856093 ^ y * 19349663 ^ z * 83492791) & 0xffff;
          const densityScale = 0.8 + (neighbors / 26.0) * 0.6;
          const noiseScale = ((hash & 0xff) / 255.0 - 0.5) * 0.15;
          const scaleVar = densityScale + noiseScale;
          this.dummy.scale.setScalar(scaleVar);

          // Random rotation around Z for visual variety (won't matter much with billboard)
          this.dummy.rotation.set(0, 0, (hash >> 8) / 255.0 * Math.PI);
          this.dummy.updateMatrix();
          this.mesh.setMatrixAt(count, this.dummy.matrix);

          // Pick color based on species (nutrient_level byte = species_id)
          const speciesId = grid[idx + 3];
          const baseColor = speciesId < SPECIES_FOLIAGE.length
            ? SPECIES_FOLIAGE[speciesId]
            : LEAF_COLORS[hash % LEAF_COLORS.length];

          // Health-based stress tint: water_level byte MAY store health.
          // However, leaf voxels in the canopy often have water_level=0 (no
          // water above ground), which is NOT "dead" — it means "no data".
          // Treat 0 as fully healthy. Only values 1–59 indicate actual stress.
          const health = grid[idx + 1];
          const healthFrac = health === 0 ? 1.0 : Math.min(health / 60, 1);
          const stressTint = Math.pow(Math.max(0, 1 - healthFrac), 2);

          // Per-instance brightness variation for visual richness
          const brightness = 0.9 + ((hash >> 4) & 0xf) / 15.0 * 0.2;
          const r = baseColor.r * brightness + stressTint * 0.2;
          const g = baseColor.g * brightness * (1 - stressTint * 0.3);
          const b = baseColor.b * brightness * (1 - stressTint * 0.5);
          this.colorAttr.setXYZ(count, r, g, b);

          count++;
        }
      }
    }

    this.instanceCount = count;
    this.mesh.count = count;
    this.mesh.instanceMatrix.needsUpdate = true;
    this.colorAttr.needsUpdate = true;
  }

  /**
   * Update wind animation. Call each frame with elapsed time.
   */
  update(elapsedTime: number): void {
    this.material.uniforms.uTime.value = elapsedTime;
  }

  /** Get current wind strength */
  getWindStrength(): number {
    return this.material.uniforms.uWindStrength.value as number;
  }

  /** Set wind strength (0 = still, 1 = gusty) */
  setWindStrength(strength: number): void {
    this.material.uniforms.uWindStrength.value = strength;
  }

  /** Current number of active foliage sprites */
  get count(): number {
    return this.instanceCount;
  }

  dispose(): void {
    this.mesh.geometry.dispose();
    this.material.dispose();
  }
}
