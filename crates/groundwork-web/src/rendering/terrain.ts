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

/** Material base colors — warm, earthy palette */
const MATERIAL_COLORS: Record<number, THREE.Color> = {
  [Material.Soil]: new THREE.Color(0.45, 0.30, 0.18),    // rich chocolate brown
  [Material.Stone]: new THREE.Color(0.55, 0.53, 0.50),    // warm gray
  [Material.Water]: new THREE.Color(0.20, 0.50, 0.60),    // teal
  [Material.Root]: new THREE.Color(0.50, 0.35, 0.15),     // tan brown
  [Material.Seed]: new THREE.Color(0.55, 0.50, 0.25),     // golden
  [Material.Trunk]: new THREE.Color(0.40, 0.28, 0.15),    // dark brown
  [Material.Branch]: new THREE.Color(0.45, 0.32, 0.18),   // medium brown
  [Material.Leaf]: new THREE.Color(0.30, 0.55, 0.20),     // warm green
  [Material.DeadWood]: new THREE.Color(0.50, 0.40, 0.28), // amber
};

const DEFAULT_COLOR = new THREE.Color(1, 0, 1); // magenta = unmapped material

/** AO darkening factors: 0=no occlusion (bright), 3=full occlusion (dark) */
const AO_CURVE = [1.0, 0.75, 0.55, 0.35];

/** Face normals indexed by face direction */
const FACE_NORMALS: [number, number, number][] = [
  [1, 0, 0],   // 0: +X
  [-1, 0, 0],  // 1: -X
  [0, 1, 0],   // 2: +Y
  [0, -1, 0],  // 3: -Y
  [0, 0, 1],   // 4: +Z
  [0, 0, -1],  // 5: -Z
];

/**
 * Build a Three.js mesh for a chunk's greedy-meshed quads.
 */
export function buildChunkMesh(chunk: ChunkMesh): THREE.Mesh | null {
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
    const indices = flipWinding
      ? [0, 1, 2, 2, 3, 0]
      : [0, 1, 3, 1, 2, 3];

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
    // Water is semi-transparent
  });

  const mesh = new THREE.Mesh(geometry, material);
  mesh.name = `chunk_${chunk.cx}_${chunk.cy}_${chunk.cz}`;
  return mesh;
}

/**
 * Compute the 4 corner positions of a quad in world space.
 * Returns corners in order: [bottom-left, bottom-right, top-right, top-left]
 */
function getQuadCorners(quad: MeshQuad): [number, number, number][] {
  const { x, y, z, w, h, face } = quad;

  switch (face) {
    case 0: // +X face: at x+1, spans y..y+w, z..z+h
      return [
        [x + 1, y, z],
        [x + 1, y + w, z],
        [x + 1, y + w, z + h],
        [x + 1, y, z + h],
      ];
    case 1: // -X face: at x, spans y..y+w, z..z+h
      return [
        [x, y + w, z],
        [x, y, z],
        [x, y, z + h],
        [x, y + w, z + h],
      ];
    case 2: // +Y face: at y+1, spans x..x+w, z..z+h
      return [
        [x + w, y + 1, z],
        [x, y + 1, z],
        [x, y + 1, z + h],
        [x + w, y + 1, z + h],
      ];
    case 3: // -Y face: at y, spans x..x+w, z..z+h
      return [
        [x, y, z],
        [x + w, y, z],
        [x + w, y, z + h],
        [x, y, z + h],
      ];
    case 4: // +Z face: at z+1, spans x..x+w, y..y+h
      return [
        [x, y, z + 1],
        [x + w, y, z + 1],
        [x + w, y + h, z + 1],
        [x, y + h, z + 1],
      ];
    case 5: // -Z face: at z, spans x..x+w, y..y+h
      return [
        [x + w, y, z],
        [x, y, z],
        [x, y + h, z],
        [x + w, y + h, z],
      ];
    default:
      return [[0, 0, 0], [0, 0, 0], [0, 0, 0], [0, 0, 0]];
  }
}
