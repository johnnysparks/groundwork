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
import { Material, GROUND_LEVEL, type MaterialType } from '../bridge';
import type { MeshQuad } from '../mesher/greedy';
import type { ChunkMesh } from '../mesher/chunk';

/** Material base colors — earthy palette with good contrast under ACES tone mapping */
const MATERIAL_COLORS: Record<number, THREE.Color> = {
  [Material.Soil]: new THREE.Color(0.38, 0.26, 0.15),    // dark brown (reads as rich earth)
  [Material.Stone]: new THREE.Color(0.52, 0.50, 0.48),    // cool gray (distinct from soil)
  [Material.Water]: new THREE.Color(0.20, 0.50, 0.65),    // teal
  [Material.Root]: new THREE.Color(0.50, 0.34, 0.16),     // tan brown
  [Material.Seed]: new THREE.Color(0.55, 0.50, 0.25),     // golden
  [Material.Trunk]: new THREE.Color(0.36, 0.25, 0.14),    // dark bark
  [Material.Branch]: new THREE.Color(0.40, 0.30, 0.18),   // medium brown
  [Material.Leaf]: new THREE.Color(0.30, 0.58, 0.20),     // bright green
  [Material.DeadWood]: new THREE.Color(0.48, 0.40, 0.28), // faded wood
};

/** Grass-tinted soil for top faces near/above ground level */
const SOIL_GRASS = new THREE.Color(0.28, 0.40, 0.18); // mossy green-brown

const DEFAULT_COLOR = new THREE.Color(1, 0, 1); // magenta = unmapped material

/** AO darkening factors: 0=no occlusion (bright), 3=full occlusion (dark) */
const AO_CURVE = [1.0, 0.82, 0.65, 0.50];

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

/** Shared material for soil/stone meshes — toggled transparent for x-ray */
let soilMaterial: THREE.MeshLambertMaterial | null = null;

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
  const mat = getSoilMaterial();
  mat.opacity = active ? 0.12 : 1.0;
  mat.depthWrite = !active;
  mat.needsUpdate = true;
}

export interface ChunkMeshPair {
  solidMesh: THREE.Mesh | null;
  soilMesh: THREE.Mesh | null;
}

/**
 * Build Three.js meshes for a chunk's greedy-meshed quads.
 * Returns two meshes: solid (always opaque) and soil (toggleable transparency).
 */
export function buildChunkMesh(chunk: ChunkMesh): ChunkMeshPair {
  const { quads } = chunk;
  if (quads.length === 0) return { solidMesh: null, soilMesh: null };

  // Split quads into soil/stone and everything else
  const soilQuads: MeshQuad[] = [];
  const solidQuads: MeshQuad[] = [];
  for (const quad of quads) {
    if (isSoilOrStone(quad.material)) {
      soilQuads.push(quad);
    } else {
      solidQuads.push(quad);
    }
  }

  const solidMesh = buildMeshFromQuads(solidQuads, `chunk_${chunk.cx}_${chunk.cy}_${chunk.cz}`,
    new THREE.MeshLambertMaterial({ vertexColors: true }));
  const soilMesh = buildMeshFromQuads(soilQuads, `soil_${chunk.cx}_${chunk.cy}_${chunk.cz}`,
    getSoilMaterial());

  return { solidMesh, soilMesh };
}

function buildMeshFromQuads(quads: MeshQuad[], name: string, material: THREE.Material): THREE.Mesh | null {
  if (quads.length === 0) return null;

  const vertCount = quads.length * 6;
  const positions = new Float32Array(vertCount * 3);
  const normals = new Float32Array(vertCount * 3);
  const colors = new Float32Array(vertCount * 3);

  let vi = 0;

  for (const quad of quads) {
    // Top-facing soil near ground level gets a grass tint
    const isGrass = quad.material === Material.Soil && quad.face === 4 && quad.z >= GROUND_LEVEL - 1;
    const baseColor = isGrass ? SOIL_GRASS : (MATERIAL_COLORS[quad.material] ?? DEFAULT_COLOR);
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

      positions[vi * 3] = cx;
      positions[vi * 3 + 1] = cy;
      positions[vi * 3 + 2] = cz;

      normals[vi * 3] = nx;
      normals[vi * 3 + 1] = ny;
      normals[vi * 3 + 2] = nz;

      colors[vi * 3] = baseColor.r * aoFactor;
      colors[vi * 3 + 1] = baseColor.g * aoFactor;
      colors[vi * 3 + 2] = baseColor.b * aoFactor;

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
