/**
 * Voxel raycaster: converts mouse clicks to voxel grid coordinates.
 *
 * Uses Three.js Raycaster against the terrain mesh group to find the
 * clicked voxel face, then derives the target voxel position based on
 * the active tool (place tools target the neighbor; dig targets the hit voxel).
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GRID_Z } from '../bridge';

export interface VoxelHit {
  /** Voxel coordinates to act on (depends on tool: adjacent for place, hit for dig) */
  x: number;
  y: number;
  z: number;
  /** The voxel that was actually hit by the ray */
  hitX: number;
  hitY: number;
  hitZ: number;
  /** Face normal of the hit surface */
  normal: THREE.Vector3;
}

const raycaster = new THREE.Raycaster();
const mouse = new THREE.Vector2();

/**
 * Cast a ray from screen coordinates and find the voxel to act on.
 *
 * @param screenX Mouse clientX
 * @param screenY Mouse clientY
 * @param camera The active camera
 * @param terrainGroup Group containing all chunk meshes
 * @param placingMode If true, target the adjacent voxel (for placing). If false, target the hit voxel (for digging).
 * @returns The voxel hit info, or null if no terrain was hit
 */
export function raycastVoxel(
  screenX: number,
  screenY: number,
  camera: THREE.Camera,
  terrainGroup: THREE.Group,
  placingMode: boolean,
): VoxelHit | null {
  // Convert screen coords to normalized device coordinates (-1 to +1)
  mouse.x = (screenX / window.innerWidth) * 2 - 1;
  mouse.y = -(screenY / window.innerHeight) * 2 + 1;

  raycaster.setFromCamera(mouse, camera);

  const intersects = raycaster.intersectObjects(terrainGroup.children, false);
  if (intersects.length === 0) return null;

  const hit = intersects[0];
  if (!hit.face) return null;

  const point = hit.point;
  const normal = hit.face.normal.clone();

  // Transform normal from object space to world space
  const normalMatrix = new THREE.Matrix3().getNormalMatrix(hit.object.matrixWorld);
  normal.applyMatrix3(normalMatrix).normalize();

  // The hit point is on a face surface. To find the voxel behind the face,
  // step slightly inward from the surface (opposite normal direction).
  const epsilon = 0.01;
  const hitVoxelX = Math.floor(point.x - normal.x * epsilon);
  const hitVoxelY = Math.floor(point.y - normal.y * epsilon);
  const hitVoxelZ = Math.floor(point.z - normal.z * epsilon);

  // Clamp to grid bounds
  const hx = Math.max(0, Math.min(GRID_X - 1, hitVoxelX));
  const hy = Math.max(0, Math.min(GRID_Y - 1, hitVoxelY));
  const hz = Math.max(0, Math.min(GRID_Z - 1, hitVoxelZ));

  if (placingMode) {
    // Place on the adjacent voxel (step along normal)
    const adjX = Math.max(0, Math.min(GRID_X - 1, hx + Math.round(normal.x)));
    const adjY = Math.max(0, Math.min(GRID_Y - 1, hy + Math.round(normal.y)));
    const adjZ = Math.max(0, Math.min(GRID_Z - 1, hz + Math.round(normal.z)));
    return { x: adjX, y: adjY, z: adjZ, hitX: hx, hitY: hy, hitZ: hz, normal };
  }

  return { x: hx, y: hy, z: hz, hitX: hx, hitY: hy, hitZ: hz, normal };
}

/**
 * Get the voxel coordinate that the mouse is hovering over (for cursor highlight).
 * Same as raycastVoxel but always returns the adjacent voxel for preview.
 */
export function raycastHover(
  screenX: number,
  screenY: number,
  camera: THREE.Camera,
  terrainGroup: THREE.Group,
): VoxelHit | null {
  return raycastVoxel(screenX, screenY, camera, terrainGroup, true);
}
