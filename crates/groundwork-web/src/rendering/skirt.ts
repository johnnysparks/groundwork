/**
 * Ground skirt: opaque walls around the diorama edges and a bottom cap.
 *
 * Hides the underground voxel cross-section that looks dark and messy from
 * the isometric diorama view. Makes the garden look like it's sitting on a
 * solid pedestal / tabletop.
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GROUND_LEVEL } from '../bridge';

/**
 * Build a skirt mesh that wraps the underground portion of the diorama.
 * Four vertical walls from y=0 to y=GROUND_LEVEL at the grid edges,
 * plus a bottom cap at y=0.
 */
export function buildSkirtMesh(): THREE.Mesh {
  const geometry = new THREE.BufferGeometry();

  const gx = GRID_X;
  const gz = GRID_Y; // sim Y → Three.js Z
  const top = GROUND_LEVEL;
  const bot = 0;

  // 4 walls + 1 bottom = 5 quads = 10 triangles = 30 vertices
  const positions: number[] = [];
  const normals: number[] = [];

  // Helper: add a quad (2 triangles, 6 vertices) with a given normal
  function addQuad(
    p0: [number, number, number],
    p1: [number, number, number],
    p2: [number, number, number],
    p3: [number, number, number],
    n: [number, number, number],
  ) {
    // Triangle 1: p0, p1, p2
    for (const p of [p0, p1, p2]) {
      positions.push(p[0], p[1], p[2]);
      normals.push(n[0], n[1], n[2]);
    }
    // Triangle 2: p0, p2, p3
    for (const p of [p0, p2, p3]) {
      positions.push(p[0], p[1], p[2]);
      normals.push(n[0], n[1], n[2]);
    }
  }

  // Front wall (Z = 0, facing -Z)
  addQuad(
    [0, bot, 0], [gx, bot, 0], [gx, top, 0], [0, top, 0],
    [0, 0, -1],
  );

  // Back wall (Z = gz, facing +Z)
  addQuad(
    [gx, bot, gz], [0, bot, gz], [0, top, gz], [gx, top, gz],
    [0, 0, 1],
  );

  // Left wall (X = 0, facing -X)
  addQuad(
    [0, bot, gz], [0, bot, 0], [0, top, 0], [0, top, gz],
    [-1, 0, 0],
  );

  // Right wall (X = gx, facing +X)
  addQuad(
    [gx, bot, 0], [gx, bot, gz], [gx, top, gz], [gx, top, 0],
    [1, 0, 0],
  );

  // Bottom cap (Y = 0, facing -Y)
  addQuad(
    [0, bot, gz], [gx, bot, gz], [gx, bot, 0], [0, bot, 0],
    [0, -1, 0],
  );

  const posArr = new Float32Array(positions);
  const norArr = new Float32Array(normals);

  geometry.setAttribute('position', new THREE.BufferAttribute(posArr, 3));
  geometry.setAttribute('normal', new THREE.BufferAttribute(norArr, 3));

  // Dark earthy color — like a wooden planter box edge, darker than soil for contrast
  const material = new THREE.MeshLambertMaterial({
    color: 0x5C4A3A, // dark wood/earth
  });

  const mesh = new THREE.Mesh(geometry, material);
  mesh.name = 'ground_skirt';
  mesh.receiveShadow = true;
  return mesh;
}
