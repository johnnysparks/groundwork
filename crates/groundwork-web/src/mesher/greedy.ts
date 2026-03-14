/**
 * Greedy meshing for voxel grids.
 *
 * Generates optimized geometry by merging coplanar, same-material faces
 * into larger quads. Operates on a single chunk at a time.
 *
 * References:
 * - "Meshing in a Minecraft Game" (0fps.net)
 * - "Voxel Meshing in Exile" (thenumb.at)
 */

import { GRID_X, GRID_Y, GRID_Z, GROUND_LEVEL, VOXEL_BYTES, Material, type MaterialType } from '../bridge';

/** A single face quad emitted by the greedy mesher */
export interface MeshQuad {
  /** Bottom-left corner of the quad in world coordinates */
  x: number;
  y: number;
  z: number;
  /** Width and height of the quad (in the face plane) */
  w: number;
  h: number;
  /** Face normal direction: 0=+x, 1=-x, 2=+y, 3=-y, 4=+z, 5=-z */
  face: number;
  /** Material type of this face */
  material: MaterialType;
  /** Average ambient occlusion at the 4 corners (0-3 each, packed) */
  ao: [number, number, number, number];
}

/** Chunk dimensions */
export const CHUNK_SIZE = 16;
export const CHUNKS_X = Math.ceil(GRID_X / CHUNK_SIZE); // 8
export const CHUNKS_Y = Math.ceil(GRID_Y / CHUNK_SIZE); // 8
export const CHUNKS_Z = Math.ceil(GRID_Z / CHUNK_SIZE); // 4

/**
 * Check if a voxel is solid (not air, water, or foliage).
 * Air voxels don't generate faces. Water is rendered separately
 * with a custom shader. Foliage (Leaf) is rendered as billboard sprites.
 */
function isSolid(mat: number): boolean {
  return mat !== Material.Air && mat !== Material.Water && mat !== Material.Leaf;
}

/**
 * Check if a material is vegetation that should be rendered as billboards.
 */
export function isFoliage(mat: number): boolean {
  return mat === Material.Leaf;
}

/**
 * Read material from the grid at absolute coordinates.
 * Returns Air for out-of-bounds.
 */
function readMaterial(grid: Uint8Array, x: number, y: number, z: number): number {
  if (x < 0 || x >= GRID_X || y < 0 || y >= GRID_Y || z < 0 || z >= GRID_Z) {
    return Material.Air;
  }
  const idx = (x + y * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
  return grid[idx];
}

/**
 * Compute per-vertex ambient occlusion for a face vertex.
 * Counts how many of the 3 neighboring voxels are solid.
 * Returns 0 (no occlusion) to 3 (fully occluded corner).
 */
function vertexAO(side1: boolean, side2: boolean, corner: boolean): number {
  if (side1 && side2) return 3;
  return (side1 ? 1 : 0) + (side2 ? 1 : 0) + (corner ? 1 : 0);
}

/**
 * Compute AO for the 4 corners of a +Z face at (x, y, z+1).
 * For other face directions, callers rotate the neighbor lookups.
 */
function computeFaceAO(
  grid: Uint8Array,
  x: number, y: number, z: number,
  face: number,
): [number, number, number, number] {
  // For each face direction, define the tangent/bitangent axes and the
  // normal direction to sample neighbors for AO.
  // face: 0=+x, 1=-x, 2=+y, 3=-y, 4=+z, 5=-z
  let nx: number, ny: number, nz: number;
  let tx: number, ty: number, tz: number;
  let bx: number, by: number, bz: number;

  switch (face) {
    case 0: // +X
      nx = 1; ny = 0; nz = 0;
      tx = 0; ty = 1; tz = 0;
      bx = 0; by = 0; bz = 1;
      break;
    case 1: // -X
      nx = -1; ny = 0; nz = 0;
      tx = 0; ty = -1; tz = 0;
      bx = 0; by = 0; bz = 1;
      break;
    case 2: // +Y
      nx = 0; ny = 1; nz = 0;
      tx = -1; ty = 0; tz = 0;
      bx = 0; by = 0; bz = 1;
      break;
    case 3: // -Y
      nx = 0; ny = -1; nz = 0;
      tx = 1; ty = 0; tz = 0;
      bx = 0; by = 0; bz = 1;
      break;
    case 4: // +Z
      nx = 0; ny = 0; nz = 1;
      tx = 1; ty = 0; tz = 0;
      bx = 0; by = 1; bz = 0;
      break;
    case 5: // -Z
      nx = 0; ny = 0; nz = -1;
      tx = -1; ty = 0; tz = 0;
      bx = 0; by = 1; bz = 0;
      break;
    default:
      return [0, 0, 0, 0];
  }

  // Sample the 8 neighbors on the face plane (in the normal direction)
  const fx = x + nx, fy = y + ny, fz = z + nz;
  const s = (dx: number, dy: number, dz: number) =>
    isSolid(readMaterial(grid, fx + dx, fy + dy, fz + dz));

  // 4 corners of the quad, with their side and corner neighbors
  const a0 = vertexAO(s(-tx, -ty, -tz), s(-bx, -by, -bz), s(-tx - bx, -ty - by, -tz - bz));
  const a1 = vertexAO(s(tx, ty, tz), s(-bx, -by, -bz), s(tx - bx, ty - by, tz - bz));
  const a2 = vertexAO(s(tx, ty, tz), s(bx, by, bz), s(tx + bx, ty + by, tz + bz));
  const a3 = vertexAO(s(-tx, -ty, -tz), s(bx, by, bz), s(-tx + bx, -ty + by, -tz + bz));

  return [a0, a1, a2, a3];
}

/**
 * Generate greedy-meshed quads for a single chunk.
 *
 * @param grid Raw voxel grid data (full grid, not chunk-local)
 * @param cx Chunk X index (0..CHUNKS_X-1)
 * @param cy Chunk Y index
 * @param cz Chunk Z index
 * @returns Array of merged face quads
 */
export function meshChunk(
  grid: Uint8Array,
  cx: number,
  cy: number,
  cz: number,
): MeshQuad[] {
  const quads: MeshQuad[] = [];
  const x0 = cx * CHUNK_SIZE;
  const y0 = cy * CHUNK_SIZE;
  const z0 = cz * CHUNK_SIZE;
  const x1 = Math.min(x0 + CHUNK_SIZE, GRID_X);
  const y1 = Math.min(y0 + CHUNK_SIZE, GRID_Y);
  const z1 = Math.min(z0 + CHUNK_SIZE, GRID_Z);

  // For each face direction, sweep slices and run greedy merge
  // face 0: +X, face 1: -X, face 2: +Y, face 3: -Y, face 4: +Z, face 5: -Z
  for (let face = 0; face < 6; face++) {
    // Determine sweep axis and the two perpendicular axes
    let aStart: number, aEnd: number;
    let bStart: number, bEnd: number;
    let sStart: number, sEnd: number;

    // s = slice axis (normal direction), a/b = face plane axes
    switch (face) {
      case 0: case 1: // +/-X: sweep X, face plane is Y×Z
        sStart = x0; sEnd = x1;
        aStart = y0; aEnd = y1;
        bStart = z0; bEnd = z1;
        break;
      case 2: case 3: // +/-Y: sweep Y, face plane is X×Z
        sStart = y0; sEnd = y1;
        aStart = x0; aEnd = x1;
        bStart = z0; bEnd = z1;
        break;
      default: // +/-Z: sweep Z, face plane is X×Y
        sStart = z0; sEnd = z1;
        aStart = x0; aEnd = x1;
        bStart = y0; bEnd = y1;
        break;
    }

    const aSize = aEnd - aStart;
    const bSize = bEnd - bStart;

    // Mask: which cells in this slice need a face
    const mask = new Int8Array(aSize * bSize);   // material (0 = no face)
    const aoMask = new Int32Array(aSize * bSize); // packed AO for greedy compare

    for (let s = sStart; s < sEnd; s++) {
      // Fill mask for this slice
      for (let b = bStart; b < bEnd; b++) {
        for (let a = aStart; a < aEnd; a++) {
          const mi = (a - aStart) + (b - bStart) * aSize;
          // Convert (s, a, b) back to (x, y, z) based on face direction
          let x: number, y: number, z: number;
          let nx: number, ny: number, nz: number;
          switch (face) {
            case 0: x = s; y = a; z = b; nx = 1; ny = 0; nz = 0; break;
            case 1: x = s; y = a; z = b; nx = -1; ny = 0; nz = 0; break;
            case 2: x = a; y = s; z = b; nx = 0; ny = 1; nz = 0; break;
            case 3: x = a; y = s; z = b; nx = 0; ny = -1; nz = 0; break;
            case 4: x = a; y = b; z = s; nx = 0; ny = 0; nz = 1; break;
            default: x = a; y = b; z = s; nx = 0; ny = 0; nz = -1; break;
          }

          const mat = readMaterial(grid, x, y, z);
          const neighborMat = readMaterial(grid, x + nx, y + ny, z + nz);

          // Emit face if this voxel is solid and neighbor in normal direction is air
          if (isSolid(mat) && !isSolid(neighborMat)) {
            mask[mi] = mat;
            const ao = computeFaceAO(grid, x, y, z, face);
            aoMask[mi] = ao[0] | (ao[1] << 4) | (ao[2] << 8) | (ao[3] << 12);
          } else {
            mask[mi] = 0;
            aoMask[mi] = 0;
          }
        }
      }

      // Greedy merge: sweep rows, expand quads
      for (let b = 0; b < bSize; b++) {
        let a = 0;
        while (a < aSize) {
          const idx = a + b * aSize;
          const mat = mask[idx];
          if (mat === 0) { a++; continue; }

          const ao = aoMask[idx];

          // Expand width (along a-axis)
          let w = 1;
          while (a + w < aSize) {
            const ni = (a + w) + b * aSize;
            if (mask[ni] === mat && aoMask[ni] === ao) {
              w++;
            } else {
              break;
            }
          }

          // Expand height (along b-axis)
          let h = 1;
          outer:
          while (b + h < bSize) {
            for (let da = 0; da < w; da++) {
              const ni = (a + da) + (b + h) * aSize;
              if (mask[ni] !== mat || aoMask[ni] !== ao) {
                break outer;
              }
            }
            h++;
          }

          // Clear the merged region from the mask
          for (let db = 0; db < h; db++) {
            for (let da = 0; da < w; da++) {
              mask[(a + da) + (b + db) * aSize] = 0;
            }
          }

          // Convert back to world coordinates
          let qx: number, qy: number, qz: number;
          let qw: number, qh: number;
          switch (face) {
            case 0: case 1:
              qx = s; qy = aStart + a; qz = bStart + b; qw = w; qh = h; break;
            case 2: case 3:
              qx = aStart + a; qy = s; qz = bStart + b; qw = w; qh = h; break;
            default:
              qx = aStart + a; qy = bStart + b; qz = s; qw = w; qh = h; break;
          }

          const unpackedAO: [number, number, number, number] = [
            ao & 0xf,
            (ao >> 4) & 0xf,
            (ao >> 8) & 0xf,
            (ao >> 12) & 0xf,
          ];

          quads.push({
            x: qx, y: qy, z: qz,
            w: qw, h: qh,
            face,
            material: mat as MaterialType,
            ao: unpackedAO,
          });

          a += w;
        }
      }
    }
  }

  return quads;
}

/**
 * Generate a mock voxel grid for testing the mesher without WASM.
 * Creates flat ground at GROUND_LEVEL with a water pool and small hill.
 */
export function createMockGrid(): Uint8Array {
  const size = GRID_X * GRID_Y * GRID_Z * VOXEL_BYTES;
  const grid = new Uint8Array(size);

  for (let z = 0; z < GRID_Z; z++) {
    for (let y = 0; y < GRID_Y; y++) {
      for (let x = 0; x < GRID_X; x++) {
        const idx = (x + y * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
        if (z < 10) {
          grid[idx] = Material.Stone;
        } else if (z <= GROUND_LEVEL) {
          grid[idx] = Material.Soil;
        } else {
          grid[idx] = Material.Air;
        }
      }
    }
  }

  // Water pond at center — carved basin with varying depth
  const cx = GRID_X / 2, cy = GRID_Y / 2;
  const pondRadius = 8;
  for (let dy = -pondRadius; dy <= pondRadius; dy++) {
    for (let dx = -pondRadius; dx <= pondRadius; dx++) {
      const dist = Math.sqrt(dx * dx + dy * dy);
      if (dist > pondRadius) continue;

      // Depth increases toward center: 1 at edges, up to 4 at center
      const depthFactor = 1.0 - dist / pondRadius;
      const waterDepth = Math.max(1, Math.round(depthFactor * 4));

      // Carve basin into soil and fill with water
      for (let d = 0; d < waterDepth; d++) {
        const z = GROUND_LEVEL - d;
        const idx = ((cx + dx) + (cy + dy) * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
        grid[idx] = Material.Water;
        grid[idx + 1] = 255; // water_level
      }
    }
  }

  // Small stream flowing from pond (shallow water, 1 deep)
  for (let sx = cx + pondRadius; sx < cx + pondRadius + 12; sx++) {
    for (let sy = cy - 1; sy <= cy + 1; sy++) {
      if (sx < GRID_X) {
        const idx = (sx + sy * GRID_X + GROUND_LEVEL * GRID_X * GRID_Y) * VOXEL_BYTES;
        grid[idx] = Material.Water;
        grid[idx + 1] = 200;
      }
    }
  }

  // Small hill
  for (let dy = -3; dy <= 3; dy++) {
    for (let dx = -3; dx <= 3; dx++) {
      const dist = Math.sqrt(dx * dx + dy * dy);
      const height = Math.max(0, Math.floor(3 - dist));
      for (let h = 0; h < height; h++) {
        const z = GROUND_LEVEL + 1 + h;
        if (z < GRID_Z) {
          const idx = ((30 + dx) + (30 + dy) * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
          grid[idx] = Material.Soil;
        }
      }
    }
  }

  // --- Trees and vegetation ---

  // Helper: place a tree with trunk and leaf canopy
  function placeTree(tx: number, ty: number, trunkHeight: number, canopyRadius: number): void {
    // Trunk
    for (let h = 0; h < trunkHeight; h++) {
      const z = GROUND_LEVEL + 1 + h;
      if (z >= GRID_Z) break;
      const idx = (tx + ty * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
      grid[idx] = Material.Trunk;
    }
    // Canopy (sphere of leaves at top of trunk)
    const canopyZ = GROUND_LEVEL + trunkHeight - 1;
    for (let dz = -1; dz <= canopyRadius; dz++) {
      for (let dy = -canopyRadius; dy <= canopyRadius; dy++) {
        for (let dx = -canopyRadius; dx <= canopyRadius; dx++) {
          if (dx === 0 && dy === 0 && dz <= 0) continue; // don't overwrite trunk
          const dist = Math.sqrt(dx * dx + dy * dy + dz * dz);
          if (dist <= canopyRadius + 0.3) {
            const z = canopyZ + dz;
            const x = tx + dx;
            const y = ty + dy;
            if (z >= 0 && z < GRID_Z && x >= 0 && x < GRID_X && y >= 0 && y < GRID_Y) {
              const idx = (x + y * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
              if (grid[idx] === Material.Air) {
                grid[idx] = Material.Leaf;
              }
            }
          }
        }
      }
    }
  }

  // Original tree
  placeTree(40, 40, 6, 3);

  // More trees for a small grove
  placeTree(50, 45, 8, 4);    // taller oak-like
  placeTree(35, 55, 5, 2);    // small tree
  placeTree(55, 55, 7, 3);    // medium tree
  placeTree(45, 35, 4, 2);    // short tree
  placeTree(65, 50, 9, 4);    // tall tree
  placeTree(70, 65, 6, 3);    // tree near edge

  // A few shrubs (short trunk + small canopy)
  placeTree(42, 50, 2, 1);
  placeTree(48, 42, 2, 1);
  placeTree(58, 48, 2, 1);

  return grid;
}
