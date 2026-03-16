/**
 * Soft-voxel primitive builders.
 *
 * Shared geometry helpers for building charming, toylike 3D models
 * from rounded primitives. Think Pocket Build / cozy diorama aesthetic:
 * low-poly, warm colors, slightly oversized proportions.
 */

import * as THREE from 'three';

// Shared materials cache to avoid duplicate material creation
const materialCache = new Map<string, THREE.MeshLambertMaterial>();

/**
 * Get or create a MeshLambertMaterial with warm emissive fill.
 * Cached by color hex string for reuse across models.
 */
export function warmMat(color: number, emissiveIntensity = 0.25): THREE.MeshLambertMaterial {
  const key = `${color.toString(16)}_${emissiveIntensity}`;
  let mat = materialCache.get(key);
  if (!mat) {
    mat = new THREE.MeshLambertMaterial({
      color,
      emissive: color,
      emissiveIntensity,
    });
    materialCache.set(key, mat);
  }
  return mat;
}

// Shared geometry primitives (reused across all model instances)
const _sphereGeo = new THREE.SphereGeometry(1, 8, 6);
const _boxGeo = new THREE.BoxGeometry(1, 1, 1);
const _coneGeo = new THREE.ConeGeometry(1, 1, 6);
const _cylGeo = new THREE.CylinderGeometry(1, 1, 1, 6);
const _taperGeo = new THREE.CylinderGeometry(0.7, 1, 1, 6);

/** Soft sphere (8-segment, slightly faceted — toylike). */
export function sphere(
  parent: THREE.Group,
  x: number, y: number, z: number,
  radius: number,
  color: number,
): THREE.Mesh {
  const m = new THREE.Mesh(_sphereGeo, warmMat(color));
  m.position.set(x, y, z);
  m.scale.setScalar(radius);
  parent.add(m);
  return m;
}

/** Soft ellipsoid (stretched sphere). */
export function ellipsoid(
  parent: THREE.Group,
  x: number, y: number, z: number,
  rx: number, ry: number, rz: number,
  color: number,
): THREE.Mesh {
  const m = new THREE.Mesh(_sphereGeo, warmMat(color));
  m.position.set(x, y, z);
  m.scale.set(rx, ry, rz);
  parent.add(m);
  return m;
}

/** Rounded box (actually just a slightly scaled cube — reads as "soft" at small sizes). */
export function box(
  parent: THREE.Group,
  x: number, y: number, z: number,
  w: number, h: number, d: number,
  color: number,
): THREE.Mesh {
  const m = new THREE.Mesh(_boxGeo, warmMat(color));
  m.position.set(x, y, z);
  m.scale.set(w, h, d);
  parent.add(m);
  return m;
}

/** Cone (6-sided, toylike). */
export function cone(
  parent: THREE.Group,
  x: number, y: number, z: number,
  radius: number, height: number,
  color: number,
): THREE.Mesh {
  const m = new THREE.Mesh(_coneGeo, warmMat(color));
  m.position.set(x, y, z);
  m.scale.set(radius, height, radius);
  parent.add(m);
  return m;
}

/** Cylinder (6-sided). */
export function cylinder(
  parent: THREE.Group,
  x: number, y: number, z: number,
  radius: number, height: number,
  color: number,
): THREE.Mesh {
  const m = new THREE.Mesh(_cylGeo, warmMat(color));
  m.position.set(x, y, z);
  m.scale.set(radius, height, radius);
  parent.add(m);
  return m;
}

/** Tapered cylinder (wider at base — good for tree trunks). */
export function taperCylinder(
  parent: THREE.Group,
  x: number, y: number, z: number,
  radius: number, height: number,
  color: number,
): THREE.Mesh {
  const m = new THREE.Mesh(_taperGeo, warmMat(color));
  m.position.set(x, y, z);
  m.scale.set(radius, height, radius);
  parent.add(m);
  return m;
}

/** Deterministic hash for procedural variation. */
export function hash(seed: number): number {
  let x = (seed * 2654435761) >>> 0;
  x = ((x ^ (x >> 16)) * 0x45d9f3b) >>> 0;
  return x;
}

/** Hash to float [0, 1). */
export function hashFloat(seed: number): number {
  return (hash(seed) & 0xffff) / 65536;
}

/** Hash to float in range [min, max). */
export function hashRange(seed: number, min: number, max: number): number {
  return min + hashFloat(seed) * (max - min);
}
