/**
 * Terrain renderer: converts greedy-meshed quads into Three.js geometry.
 *
 * Creates BufferGeometry per chunk with per-vertex:
 * - position (vec3)
 * - normal (vec3)
 * - color (vec3, from material type + AO darkening)
 *
 * Each chunk produces TWO meshes:
 * - solidMesh: roots, trunks, branches, etc. — always opaque
 * - soilMesh: soil + stone — can be made transparent for x-ray underground view
 */

import * as THREE from 'three';
import { Material, GROUND_LEVEL, GRID_X, GRID_Y, VOXEL_BYTES, type MaterialType } from '../bridge';
import type { MeshQuad } from '../mesher/greedy';
import type { ChunkMesh } from '../mesher/chunk';

/** Material base colors — earthy palette with good contrast under ACES tone mapping */
const MATERIAL_COLORS: Record<number, THREE.Color> = {
  [Material.Soil]: new THREE.Color(0.38, 0.26, 0.15),    // dark brown (reads as rich earth)
  [Material.Stone]: new THREE.Color(0.52, 0.50, 0.48),    // cool gray (distinct from soil)
  [Material.Water]: new THREE.Color(0.20, 0.50, 0.65),    // teal
  [Material.Root]: new THREE.Color(0.50, 0.34, 0.16),     // tan brown
  [Material.Seed]: new THREE.Color(0.75, 0.65, 0.25),     // bright golden (visible on soil)
  [Material.Trunk]: new THREE.Color(0.36, 0.25, 0.14),    // dark bark
  [Material.Branch]: new THREE.Color(0.40, 0.30, 0.18),   // medium brown
  [Material.Leaf]: new THREE.Color(0.30, 0.58, 0.20),     // bright green
  [Material.DeadWood]: new THREE.Color(0.48, 0.40, 0.28), // faded wood
};

/** Species-specific trunk colors (indexed by species_id in voxel byte 3).
 *  0=Oak, 1=Birch, 2=Willow, 3=Pine, 4=Fern, 5=Berry Bush, 6=Holly,
 *  7=Wildflower, 8=Daisy, 9=Moss, 10=Grass, 11=Clover */
const SPECIES_TRUNK: THREE.Color[] = [
  new THREE.Color(0.32, 0.22, 0.12),  // Oak: dark brown bark
  new THREE.Color(0.75, 0.72, 0.65),  // Birch: white/cream bark
  new THREE.Color(0.40, 0.38, 0.30),  // Willow: gray-green bark
  new THREE.Color(0.30, 0.18, 0.10),  // Pine: dark reddish-brown
  new THREE.Color(0.30, 0.22, 0.14),  // Fern: brown stem
  new THREE.Color(0.38, 0.28, 0.18),  // Berry Bush: medium brown
  new THREE.Color(0.28, 0.22, 0.16),  // Holly: dark bark
  new THREE.Color(0.35, 0.40, 0.20),  // Wildflower: green stem
  new THREE.Color(0.35, 0.38, 0.22),  // Daisy: green stem
  new THREE.Color(0.30, 0.25, 0.15),  // Moss: brown
  new THREE.Color(0.32, 0.35, 0.18),  // Grass: greenish
  new THREE.Color(0.30, 0.28, 0.16),  // Clover: brown-green
];

/** Grass-tinted soil for faces near/above ground level.
 *  Saturated green — the garden must read as a green meadow at first glance,
 *  even under warm golden-hour lighting that shifts everything amber. */
const SOIL_GRASS = new THREE.Color(0.18, 0.52, 0.14); // lush green

/** Wet soil colors — darker and bluer as moisture increases */
const SOIL_DAMP = new THREE.Color(0.28, 0.19, 0.12);  // dark moist earth
const SOIL_WET = new THREE.Color(0.20, 0.14, 0.10);   // saturated dark mud
const SOIL_GRASS_DAMP = new THREE.Color(0.22, 0.42, 0.16); // rich damp grass
const SOIL_GRASS_WET = new THREE.Color(0.18, 0.38, 0.14);  // deep wet grass

/** Nutrient-rich soil colors — warmer golden-brown tint shows fertile earth */
const SOIL_RICH = new THREE.Color(0.45, 0.32, 0.14);         // warm golden earth
const SOIL_GRASS_RICH = new THREE.Color(0.22, 0.58, 0.16);   // lush vibrant green

/** Species-specific root colors for x-ray differentiation.
 *  Saturated, distinct hues so root wars are immediately readable.
 *  Each tree/plant species gets a unique color family. */
const SPECIES_ROOT: THREE.Color[] = [
  new THREE.Color(0.80, 0.50, 0.15),  // Oak: warm orange
  new THREE.Color(0.85, 0.80, 0.50),  // Birch: bright gold
  new THREE.Color(0.40, 0.65, 0.35),  // Willow: green
  new THREE.Color(0.55, 0.25, 0.20),  // Pine: deep red-brown
  new THREE.Color(0.20, 0.65, 0.45),  // Fern: teal
  new THREE.Color(0.70, 0.30, 0.50),  // Berry Bush: magenta-rose
  new THREE.Color(0.30, 0.45, 0.25),  // Holly: dark green
  new THREE.Color(0.75, 0.55, 0.70),  // Wildflower: pink-lavender
  new THREE.Color(0.80, 0.75, 0.25),  // Daisy: bright yellow
  new THREE.Color(0.35, 0.50, 0.25),  // Moss: olive
  new THREE.Color(0.45, 0.70, 0.30),  // Grass: bright green
  new THREE.Color(0.50, 0.70, 0.45),  // Clover: lime green
];

/** Species-specific leaf voxel colors — slightly muted vs billboard foliage
 *  so the voxel mesh reads as solid structure, not competing with soft sprites */
const SPECIES_LEAF: THREE.Color[] = [
  new THREE.Color(0.20, 0.45, 0.12),  // Oak: deep forest green
  new THREE.Color(0.42, 0.62, 0.20),  // Birch: bright lime green
  new THREE.Color(0.22, 0.48, 0.32),  // Willow: sage green
  new THREE.Color(0.10, 0.28, 0.16),  // Pine: dark blue-green
  new THREE.Color(0.14, 0.55, 0.30),  // Fern: emerald
  new THREE.Color(0.34, 0.48, 0.14),  // Berry Bush: olive
  new THREE.Color(0.12, 0.35, 0.12),  // Holly: dark green
  new THREE.Color(0.55, 0.38, 0.45),  // Wildflower: muted pink
  new THREE.Color(0.62, 0.58, 0.24),  // Daisy: warm gold
  new THREE.Color(0.16, 0.38, 0.14),  // Moss: dark olive
  new THREE.Color(0.26, 0.58, 0.12),  // Grass: fresh green
  new THREE.Color(0.34, 0.52, 0.16),  // Clover: yellow-green
];

const DEFAULT_COLOR = new THREE.Color(1, 0, 1); // magenta = unmapped material
const _scratchColor = new THREE.Color();

/** AO darkening factors: 0=no occlusion (bright), 3=full occlusion (dark) */
const AO_CURVE = [1.0, 0.82, 0.65, 0.50];

/** Simple integer hash for per-voxel color noise (returns 0.0–1.0) */
function voxelNoise(x: number, y: number, z: number): number {
  let h = (x * 73856093) ^ (y * 19349663) ^ (z * 83492791);
  h = ((h >> 16) ^ h) * 0x45d9f3b;
  h = ((h >> 16) ^ h) * 0x45d9f3b;
  h = (h >> 16) ^ h;
  return (h & 0xffff) / 65535.0;
}

/** Face normals indexed by face direction.
 *  Sim uses Z-up; Three.js uses Y-up. We swap Y↔Z here:
 *  sim +Y → Three.js +Z, sim +Z → Three.js +Y */
const FACE_NORMALS: [number, number, number][] = [
  [1, 0, 0],   // 0: +X
  [-1, 0, 0],  // 1: -X
  [0, 0, 1],   // 2: +Y (sim) → +Z (three)
  [0, 0, -1],  // 3: -Y (sim) → -Z (three)
  [0, 1, 0],   // 4: +Z (sim) → +Y (three)
  [0, -1, 0],  // 5: -Z (sim) → -Y (three)
];

/** Whether a material is soil/stone (becomes transparent in x-ray mode) */
function isSoilOrStone(mat: number): boolean {
  return mat === Material.Soil || mat === Material.Stone;
}

/** Shared material for solid terrain (trunk, branch, etc.) — tinted by day cycle */
let solidMaterial: THREE.MeshLambertMaterial | null = null;

/** Get or create the shared solid terrain material */
export function getSolidMaterial(): THREE.MeshLambertMaterial {
  if (!solidMaterial) {
    solidMaterial = new THREE.MeshLambertMaterial({
      vertexColors: true,
    });
    // Inject wind sway into vertex shader for above-ground plant voxels
    solidMaterial.onBeforeCompile = (shader) => {
      shader.uniforms.uWindStrength = windStrengthUniform;
      shader.uniforms.uWindTime = windTimeUniform;
      // Add uniforms before main
      shader.vertexShader = shader.vertexShader.replace(
        'void main() {',
        `uniform float uWindStrength;
uniform float uWindTime;
void main() {`,
      );
      // Displace above-ground vertices (Y > GROUND_LEVEL in Three.js Y-up)
      // Only sway above ground; amplitude scales with height for natural top-heavy motion
      shader.vertexShader = shader.vertexShader.replace(
        '#include <begin_vertex>',
        `#include <begin_vertex>
float heightAboveGround = position.y - ${GROUND_LEVEL.toFixed(1)};
if (heightAboveGround > 1.0 && uWindStrength > 0.01) {
  float swayAmount = heightAboveGround * 0.008 * uWindStrength;
  float phase = position.x * 0.3 + position.z * 0.2 + uWindTime * 1.5;
  transformed.x += sin(phase) * swayAmount;
  transformed.z += cos(phase * 0.7 + 1.3) * swayAmount * 0.5;
}`,
      );
    };
  }
  return solidMaterial;
}

/** Wind sway uniforms — shared across all solid terrain meshes */
const windStrengthUniform = { value: 0.0 };
const windTimeUniform = { value: 0.0 };

/** Update terrain wind sway. Call each frame with dt in seconds. */
export function updateTerrainWind(dt: number, windStrength: number): void {
  windTimeUniform.value += dt;
  windStrengthUniform.value = windStrength;
}

/** Set terrain day-night tint (multiplied against vertex colors) */
export function setTerrainDayTint(r: number, g: number, b: number): void {
  if (solidMaterial) solidMaterial.color.setRGB(r, g, b);
  if (soilMaterial) soilMaterial.color.setRGB(r, g, b);
}

/** Shared material for soil/stone meshes — toggled transparent for x-ray */
let soilMaterial: THREE.MeshLambertMaterial | null = null;

/** X-ray state — tracks cutaway depth and mode */
let xrayState = {
  active: false,
  /** Cutaway depth level (0 = surface, negative = deeper underground) */
  cutawayY: GROUND_LEVEL,
};

/** Get or create the shared soil material */
export function getSoilMaterial(): THREE.MeshLambertMaterial {
  if (!soilMaterial) {
    soilMaterial = new THREE.MeshLambertMaterial({
      vertexColors: true,
      transparent: true,
      opacity: 1.0,
      depthWrite: true,
    });
  }
  return soilMaterial;
}

/** Set x-ray mode: make soil/stone transparent to see roots underground */
export function setXrayMode(active: boolean): void {
  xrayState.active = active;
  if (!active) {
    xrayState.cutawayY = GROUND_LEVEL;
  }
  const mat = getSoilMaterial();
  mat.opacity = active ? 0.12 : 1.0;
  mat.depthWrite = !active;
  mat.needsUpdate = true;

  // Update clipping plane on root glow material
  updateRootGlowClip(active);
}

/** Adjust cutaway depth (for scroll-based depth exploration in x-ray mode) */
export function adjustCutawayDepth(delta: number): void {
  if (!xrayState.active) return;
  xrayState.cutawayY = Math.max(0, Math.min(GROUND_LEVEL + 5, xrayState.cutawayY + delta));
  const mat = getSoilMaterial();
  // Soil below cutaway is more transparent; soil above is less
  // This creates a "slicing" effect
  mat.opacity = 0.08 + (xrayState.cutawayY / GROUND_LEVEL) * 0.08;
  mat.needsUpdate = true;
}

/** Get current x-ray state */
export function getXrayState() {
  return xrayState;
}

// --- Root glow material ---
// When x-ray is active, roots get a warm emissive glow to make them
// visible through transparent soil — the signature "X-Ray Garden" effect.

let rootGlowMaterial: THREE.MeshLambertMaterial | null = null;

/** Get or create the root glow material */
export function getRootGlowMaterial(): THREE.MeshLambertMaterial {
  if (!rootGlowMaterial) {
    rootGlowMaterial = new THREE.MeshLambertMaterial({
      vertexColors: true,
      emissive: new THREE.Color(0.0, 0.0, 0.0),
      emissiveIntensity: 0.0,
    });
  }
  return rootGlowMaterial;
}

/** Update root material when x-ray mode changes — no glow, just natural colors */
function updateRootGlowClip(_xrayActive: boolean): void {
  // Roots use their natural brown vertex colors in all modes.
  // No emissive glow — the greyscale scene + transparent soil is enough
  // to make roots readable without artificial highlighting.
}

export interface ChunkMeshResult {
  solidMesh: THREE.Mesh | null;
  soilMesh: THREE.Mesh | null;
  rootMesh: THREE.Mesh | null;
}

// Keep backwards-compatible export
export type ChunkMeshPair = ChunkMeshResult;

/**
 * Build Three.js meshes for a chunk's greedy-meshed quads.
 * Returns three meshes:
 * - solid: trunks, branches, dead wood — always opaque
 * - soil: soil + stone — toggleable transparency for x-ray
 * - root: root voxels — get emissive glow in x-ray mode
 */
export function buildChunkMesh(chunk: ChunkMesh, grid?: Uint8Array): ChunkMeshResult {
  const { quads } = chunk;
  if (quads.length === 0) return { solidMesh: null, soilMesh: null, rootMesh: null };

  // Split quads into soil/stone, roots, and everything else
  const soilQuads: MeshQuad[] = [];
  const rootQuads: MeshQuad[] = [];
  const solidQuads: MeshQuad[] = [];
  for (const quad of quads) {
    if (isSoilOrStone(quad.material)) {
      soilQuads.push(quad);
    } else if (quad.material === Material.Root) {
      rootQuads.push(quad);
    } else {
      solidQuads.push(quad);
    }
  }

  const solidMesh = buildMeshFromQuads(solidQuads, `chunk_${chunk.cx}_${chunk.cy}_${chunk.cz}`,
    getSolidMaterial(), grid);
  const soilMesh = buildMeshFromQuads(soilQuads, `soil_${chunk.cx}_${chunk.cy}_${chunk.cz}`,
    getSoilMaterial());
  const rootMesh = buildMeshFromQuads(rootQuads, `root_${chunk.cx}_${chunk.cy}_${chunk.cz}`,
    getRootGlowMaterial(), grid);

  return { solidMesh, soilMesh, rootMesh };
}

function buildMeshFromQuads(quads: MeshQuad[], name: string, material: THREE.Material, grid?: Uint8Array): THREE.Mesh | null {
  if (quads.length === 0) return null;

  const vertCount = quads.length * 6;
  const positions = new Float32Array(vertCount * 3);
  const normals = new Float32Array(vertCount * 3);
  const colors = new Float32Array(vertCount * 3);

  let vi = 0;

  for (const quad of quads) {
    // Soil color depends on wetness and whether it's a top face (grass)
    let baseColor: THREE.Color;
    if (quad.material === Material.Soil) {
      // Grass on ALL faces near the surface — the garden reads as a green
      // meadow, not brown dirt. Extends 6 voxels below ground level to cover
      // the rolling terrain contour steps that otherwise show brown soil sides.
      const isGrass = quad.z >= GROUND_LEVEL - 6;
      if (quad.wetness >= 2) {
        baseColor = isGrass ? SOIL_GRASS_WET : SOIL_WET;
      } else if (quad.wetness >= 1) {
        baseColor = isGrass ? SOIL_GRASS_DAMP : SOIL_DAMP;
      } else {
        baseColor = isGrass ? SOIL_GRASS : MATERIAL_COLORS[Material.Soil];
      }
      // Nutrient-rich soil: blend toward warm golden tint. The richer the
      // soil, the more visible the warm glow — teaches players where
      // nitrogen handshake, worm activity, and decomposition enriched soil.
      if (quad.nutrient >= 2) {
        const richTarget = isGrass ? SOIL_GRASS_RICH : SOIL_RICH;
        baseColor = _scratchColor.copy(baseColor).lerp(richTarget, 0.5);
      } else if (quad.nutrient >= 1) {
        const richTarget = isGrass ? SOIL_GRASS_RICH : SOIL_RICH;
        baseColor = _scratchColor.copy(baseColor).lerp(richTarget, 0.2);
      }
    } else if (grid && (quad.material === Material.Trunk || quad.material === Material.Branch || quad.material === Material.Root || quad.material === Material.Leaf)) {
      // Look up species_id from voxel byte 3 for species-specific colors
      const vIdx = (quad.x + quad.y * GRID_X + quad.z * GRID_X * GRID_Y) * VOXEL_BYTES;
      const speciesId = grid[vIdx + 3] ?? 0;
      if (quad.material === Material.Root) {
        baseColor = SPECIES_ROOT[speciesId] ?? MATERIAL_COLORS[Material.Root];
      } else if (quad.material === Material.Leaf) {
        baseColor = SPECIES_LEAF[speciesId] ?? MATERIAL_COLORS[Material.Leaf];
      } else {
        const speciesColor = SPECIES_TRUNK[speciesId];
        if (speciesColor) {
          baseColor = quad.material === Material.Branch
            ? _scratchColor.copy(speciesColor).multiplyScalar(1.15)
            : speciesColor;
        } else {
          baseColor = MATERIAL_COLORS[quad.material] ?? DEFAULT_COLOR;
        }
      }
    } else {
      baseColor = MATERIAL_COLORS[quad.material] ?? DEFAULT_COLOR;
    }
    // Depth-based darkening for soil/stone: deeper = darker, breaks contour bands
    let depthFactor = 1.0;
    let noiseFactor = 0.0;
    if (quad.material === Material.Soil || quad.material === Material.Stone) {
      // Darken by 0–15% based on depth below surface
      const depthBelow = Math.max(0, GROUND_LEVEL - quad.z);
      depthFactor = 1.0 - Math.min(depthBelow / GROUND_LEVEL, 1.0) * 0.15;
      // Per-voxel noise: ±5% brightness variation to break uniformity
      noiseFactor = (voxelNoise(quad.x, quad.y, quad.z) - 0.5) * 0.10;
    }

    const [nx, ny, nz] = FACE_NORMALS[quad.face];
    const corners = getQuadCorners(quad);
    const ao = quad.ao;
    const flipWinding = ao[0] + ao[2] > ao[1] + ao[3];
    const indices = flipWinding
      ? [0, 2, 1, 2, 0, 3]
      : [0, 3, 1, 1, 3, 2];

    for (const ci of indices) {
      const [cx, cy, cz] = corners[ci];
      const aoFactor = AO_CURVE[ao[ci]];
      const colorScale = aoFactor * depthFactor + noiseFactor;

      positions[vi * 3] = cx;
      positions[vi * 3 + 1] = cy;
      positions[vi * 3 + 2] = cz;

      normals[vi * 3] = nx;
      normals[vi * 3 + 1] = ny;
      normals[vi * 3 + 2] = nz;

      colors[vi * 3] = Math.max(0, baseColor.r * colorScale);
      colors[vi * 3 + 1] = Math.max(0, baseColor.g * colorScale);
      colors[vi * 3 + 2] = Math.max(0, baseColor.b * colorScale);

      vi++;
    }
  }

  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3));
  geometry.setAttribute('normal', new THREE.BufferAttribute(normals, 3));
  geometry.setAttribute('color', new THREE.BufferAttribute(colors, 3));

  const mesh = new THREE.Mesh(geometry, material);
  mesh.name = name;
  return mesh;
}

/**
 * Compute the 4 corner positions of a quad in world space.
 * Returns corners in order: [bottom-left, bottom-right, top-right, top-left]
 *
 * COORDINATE SWAP: sim (x,y,z) where Z=up → Three.js (x,z,y) where Y=up.
 * Every output triple is [sim_x, sim_z, sim_y].
 */
function getQuadCorners(quad: MeshQuad): [number, number, number][] {
  const { x, y, z, w, h, face } = quad;

  switch (face) {
    case 0: // +X face: at x+1, spans simY..simY+w, simZ..simZ+h
      return [
        [x + 1, z, y],
        [x + 1, z, y + w],
        [x + 1, z + h, y + w],
        [x + 1, z + h, y],
      ];
    case 1: // -X face: at x, spans simY..simY+w, simZ..simZ+h
      return [
        [x, z, y + w],
        [x, z, y],
        [x, z + h, y],
        [x, z + h, y + w],
      ];
    case 2: // +simY face: at y+1, spans x..x+w, simZ..simZ+h
      return [
        [x + w, z, y + 1],
        [x, z, y + 1],
        [x, z + h, y + 1],
        [x + w, z + h, y + 1],
      ];
    case 3: // -simY face: at y, spans x..x+w, simZ..simZ+h
      return [
        [x, z, y],
        [x + w, z, y],
        [x + w, z + h, y],
        [x, z + h, y],
      ];
    case 4: // +simZ face (top): at z+1, spans x..x+w, simY..simY+h
      return [
        [x, z + 1, y],
        [x + w, z + 1, y],
        [x + w, z + 1, y + h],
        [x, z + 1, y + h],
      ];
    case 5: // -simZ face (bottom): at z, spans x..x+w, simY..simY+h
      return [
        [x + w, z, y],
        [x, z, y],
        [x, z, y + h],
        [x + w, z, y + h],
      ];
    default:
      return [[0, 0, 0], [0, 0, 0], [0, 0, 0], [0, 0, 0]];
  }
}
