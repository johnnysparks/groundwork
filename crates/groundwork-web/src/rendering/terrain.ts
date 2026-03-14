/**
 * Terrain renderer: converts greedy-meshed quads into Three.js geometry.
 *
 * Creates BufferGeometry per chunk with per-vertex:
 * - position (vec3)
 * - normal (vec3)
 * - color (vec3, from material type + AO darkening)
 */

import * as THREE from 'three';
import { Material, type MaterialType } from '../bridge';
import type { MeshQuad } from '../mesher/greedy';
import type { ChunkMesh } from '../mesher/chunk';

/** Material base colors — warm, earthy palette (brightened for diorama lighting) */
const MATERIAL_COLORS: Record<number, THREE.Color> = {
  [Material.Soil]: new THREE.Color(0.55, 0.38, 0.22),    // warm brown
  [Material.Stone]: new THREE.Color(0.62, 0.60, 0.56),    // warm gray
  [Material.Water]: new THREE.Color(0.25, 0.55, 0.65),    // teal
  [Material.Root]: new THREE.Color(0.58, 0.42, 0.20),     // tan brown
  [Material.Seed]: new THREE.Color(0.62, 0.56, 0.30),     // golden
  [Material.Trunk]: new THREE.Color(0.48, 0.35, 0.20),    // warm brown
  [Material.Branch]: new THREE.Color(0.52, 0.38, 0.22),   // medium brown
  [Material.Leaf]: new THREE.Color(0.35, 0.62, 0.25),     // bright green
  [Material.DeadWood]: new THREE.Color(0.58, 0.48, 0.32), // amber
};

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

/**
 * Build a Three.js mesh for a chunk's greedy-meshed quads.
 * Accepts optional clipping planes for underground cutaway.
 */
export function buildChunkMesh(
  chunk: ChunkMesh,
  clippingPlanes?: THREE.Plane[],
): THREE.Mesh | null {
  const { quads } = chunk;
  if (quads.length === 0) return null;

  // Pre-allocate arrays (6 vertices per quad = 2 triangles)
  const vertCount = quads.length * 6;
  const positions = new Float32Array(vertCount * 3);
  const normals = new Float32Array(vertCount * 3);
  const colors = new Float32Array(vertCount * 3);

  let vi = 0; // vertex index

  for (const quad of quads) {
    const baseColor = MATERIAL_COLORS[quad.material] ?? DEFAULT_COLOR;
    const [nx, ny, nz] = FACE_NORMALS[quad.face];

    // Generate the 4 corner positions of the quad
    const corners = getQuadCorners(quad);
    const ao = quad.ao;

    // Determine triangle winding: flip if AO is asymmetric
    // This prevents visible seams at AO boundaries
    const flipWinding = ao[0] + ao[2] > ao[1] + ao[3];

    // Emit two triangles (6 vertices)
    // Winding is reversed from original because the Y↔Z swap changes handedness
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

  const material = new THREE.MeshLambertMaterial({
    vertexColors: true,
    clippingPlanes: clippingPlanes ?? [],
  });

  const mesh = new THREE.Mesh(geometry, material);
  mesh.name = `chunk_${chunk.cx}_${chunk.cy}_${chunk.cz}`;
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
