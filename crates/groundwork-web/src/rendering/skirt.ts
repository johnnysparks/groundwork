/**
 * Ground skirt and decorative forest ring.
 *
 * The skirt hides the underground cross-section. The forest ring is pure
 * Three.js scenery — low-poly trees placed around the diorama edge to
 * create a "glen in the forest" feel. Zero sim cost, zero meshing.
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GROUND_LEVEL } from '../bridge';

/** A skirt wall mesh paired with its outward normal for culling. */
export interface SkirtWall {
  mesh: THREE.Mesh;
  /** Outward normal projected onto XZ plane (for dot-product culling). */
  normalXZ: [number, number];
}

export interface SkirtResult {
  group: THREE.Group;
  /** Direct references to the 4 wall meshes for per-frame culling. */
  walls: SkirtWall[];
}

/**
 * Build a skirt mesh that wraps the underground portion of the diorama.
 * Four vertical walls (each a separate mesh for x-ray culling) from
 * y=deep to y=GROUND_LEVEL at the grid edges, plus a bottom cap.
 *
 * Returns direct wall references so main.ts can cull them per-frame
 * using the same dot-product math as tree culling.
 */
export function buildSkirtMesh(): SkirtResult {
  const group = new THREE.Group();
  group.name = 'ground_skirt';

  const gx = GRID_X;
  const gz = GRID_Y;
  const top = GROUND_LEVEL;
  const deep = -60; // extend well below the voxel grid
  const cx = gx / 2;
  const cz = gz / 2;

  // Soil strata colors
  const topsoil = [0.36, 0.26, 0.16];   // dark brown
  const subsoil = [0.42, 0.30, 0.18];   // lighter brown
  const clay    = [0.48, 0.35, 0.22];   // reddish clay
  const bedrock = [0.32, 0.30, 0.28];   // dark gray stone

  function strataColor(y: number): [number, number, number] {
    const t = (y - deep) / (top - deep); // 0 = bottom, 1 = top
    if (t > 0.7) {
      const lt = (t - 0.7) / 0.3;
      return topsoil.map((v, j) => v + (subsoil[j] - v) * (1 - lt)) as [number, number, number];
    } else if (t > 0.4) {
      const lt = (t - 0.4) / 0.3;
      return clay.map((v, j) => v + (subsoil[j] - v) * lt) as [number, number, number];
    } else {
      const lt = t / 0.4;
      return bedrock.map((v, j) => v + (clay[j] - v) * lt) as [number, number, number];
    }
  }

  /** Build a single wall quad as its own mesh with double-sided rendering. */
  function buildWall(
    p0: [number, number, number], p1: [number, number, number],
    p2: [number, number, number], p3: [number, number, number],
    n: [number, number, number],
  ): THREE.Mesh {
    const verts: number[] = [];
    const norms: number[] = [];
    const colors: number[] = [];
    for (const p of [p0, p1, p2, p0, p2, p3]) {
      verts.push(p[0], p[1], p[2]);
      norms.push(n[0], n[1], n[2]);
      const [r, g, b] = strataColor(p[1]);
      colors.push(r, g, b);
    }
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.BufferAttribute(new Float32Array(verts), 3));
    geo.setAttribute('normal', new THREE.BufferAttribute(new Float32Array(norms), 3));
    geo.setAttribute('color', new THREE.BufferAttribute(new Float32Array(colors), 3));
    const mat = new THREE.MeshLambertMaterial({ vertexColors: true, side: THREE.DoubleSide });
    const mesh = new THREE.Mesh(geo, mat);
    mesh.receiveShadow = true;
    return mesh;
  }

  // Four walls — each a separate mesh so x-ray can cull camera-facing ones.
  const walls: SkirtWall[] = [
    { mesh: buildWall([0, deep, 0], [gx, deep, 0], [gx, top, 0], [0, top, 0], [0, 0, -1]),       normalXZ: [0, -1] },
    { mesh: buildWall([gx, deep, gz], [0, deep, gz], [0, top, gz], [gx, top, gz], [0, 0, 1]),      normalXZ: [0, 1] },
    { mesh: buildWall([0, deep, gz], [0, deep, 0], [0, top, 0], [0, top, gz], [-1, 0, 0]),         normalXZ: [-1, 0] },
    { mesh: buildWall([gx, deep, 0], [gx, deep, gz], [gx, top, gz], [gx, top, 0], [1, 0, 0]),     normalXZ: [1, 0] },
  ];

  for (const w of walls) group.add(w.mesh);

  // Bottom cap (not culled — always visible)
  const capVerts: number[] = [];
  const capNorms: number[] = [];
  const capColors: number[] = [];
  for (const p of [[0, deep, gz], [gx, deep, gz], [gx, deep, 0], [0, deep, gz], [gx, deep, 0], [0, deep, 0]] as [number, number, number][]) {
    capVerts.push(p[0], p[1], p[2]);
    capNorms.push(0, -1, 0);
    capColors.push(bedrock[0], bedrock[1], bedrock[2]);
  }
  const capGeo = new THREE.BufferGeometry();
  capGeo.setAttribute('position', new THREE.BufferAttribute(new Float32Array(capVerts), 3));
  capGeo.setAttribute('normal', new THREE.BufferAttribute(new Float32Array(capNorms), 3));
  capGeo.setAttribute('color', new THREE.BufferAttribute(new Float32Array(capColors), 3));
  const capMesh = new THREE.Mesh(capGeo, new THREE.MeshLambertMaterial({ vertexColors: true }));
  capMesh.receiveShadow = true;
  group.add(capMesh);

  // --- Underground floor plane (extends beyond the grid edges) ---
  const underRadius = 400;
  const underGeo = new THREE.CircleGeometry(underRadius, 32);
  const underMat = new THREE.MeshLambertMaterial({ color: 0x2A2826 }); // dark bedrock
  const underMesh = new THREE.Mesh(underGeo, underMat);
  underMesh.rotation.x = -Math.PI / 2;
  underMesh.position.set(cx, deep, cz);
  group.add(underMesh);

  return { group, walls };
}

// --- Layered parallax environment ---
// Concentric layers at increasing radii create depth like Alto's Odyssey:
//  Layer 0: 3D tree ring (nearest, full parallax from orbit)
//  Layer 1: Silhouette hill cylinder (mid, faded green)
//  Layer 2: Distant hill cylinder (far, hazy blue-green)
//  Ground:  Massive green plane extending under everything
//  Sky:     Gradient dome (handled by lighting/sky.ts)

const TRUNK_COLOR = 0x5C3A1E;
const CANOPY_COLORS = [0x4D8C2A, 0x3D7522, 0x5A9E35, 0x438028, 0x3A6B20, 0x558B2F];

/** Simple deterministic hash for variation */
function simpleHash(i: number): number {
  let x = (i * 2654435761) >>> 0;
  x = ((x ^ (x >> 16)) * 0x45d9f3b) >>> 0;
  return x;
}

/**
 * Build the full glade landscape: massive green ground plane, dense forest
 * ring near the garden, scattered trees fading into the distance.
 * Sized to always fill the screen at the tightest allowed zoom-out.
 */
/**
 * Update forest ring visibility: hide trees between the camera and the garden.
 * Call each frame with the camera's azimuth angle (theta from OrbitCamera).
 */
export function updateForestCulling(group: THREE.Group, cameraTheta: number): void {
  const cx = GRID_X / 2;
  const cz = GRID_Y / 2;
  // Camera look direction projected onto XZ plane (points from center toward camera)
  const camDirX = Math.cos(cameraTheta);
  const camDirZ = Math.sin(cameraTheta);

  group.traverse((obj) => {
    const angle = obj.userData.treeAngle as number | undefined;
    if (angle === undefined) return; // not a tree

    // Tree direction from center
    const treeDirX = Math.cos(angle);
    const treeDirZ = Math.sin(angle);

    // Dot product: positive = tree is on camera side (in front)
    const dot = camDirX * treeDirX + camDirZ * treeDirZ;

    // Hide trees in ~120° arc facing the camera (dot > 0.5 ≈ ±60°)
    obj.visible = dot < 0.5;
  });
}

export function buildForestRing(): THREE.Group {
  const group = new THREE.Group();
  group.name = 'glade_environment';

  const cx = GRID_X / 2;
  const cz = GRID_Y / 2;
  const groundY = GROUND_LEVEL + 0.5;
  // Trees start well outside the grid so they never overlap the sim
  const innerRing = Math.max(cx, cz) + 12;

  // Shared geometry
  const trunkGeo = new THREE.CylinderGeometry(0.5, 0.7, 1, 5);
  const trunkMat = new THREE.MeshLambertMaterial({ color: TRUNK_COLOR });

  // --- Meadow ground plane with garden cutout ---
  const groundRadius = 400;
  const gardenHalf = Math.max(cx, cz) + 1;
  const meadowShape = new THREE.Shape();
  meadowShape.absarc(0, 0, groundRadius, 0, Math.PI * 2, false);
  const holePath = new THREE.Path();
  holePath.moveTo(-gardenHalf, -gardenHalf);
  holePath.lineTo(gardenHalf, -gardenHalf);
  holePath.lineTo(gardenHalf, gardenHalf);
  holePath.lineTo(-gardenHalf, gardenHalf);
  holePath.closePath();
  meadowShape.holes.push(holePath);

  const groundGeo = new THREE.ShapeGeometry(meadowShape, 48);
  const groundMat = new THREE.MeshLambertMaterial({ color: 0x6AAF40 });
  const ground = new THREE.Mesh(groundGeo, groundMat);
  ground.name = 'meadow_ground';
  ground.rotation.x = -Math.PI / 2;
  ground.position.set(cx, groundY + 0.1, cz);
  ground.receiveShadow = true;
  group.add(ground);

  // --- Dense forest ring (outside garden footprint) ---
  const nearRings = [
    { radius: innerRing, count: 32, trunkMin: 6, trunkRange: 8, canopyMin: 4, canopyRange: 4 },
    { radius: innerRing + 8, count: 40, trunkMin: 8, trunkRange: 10, canopyMin: 5, canopyRange: 5 },
  ];

  for (const ring of nearRings) {
    for (let i = 0; i < ring.count; i++) {
      const h = simpleHash(i + ring.count * 100);
      const angle = (i / ring.count) * Math.PI * 2 + ((h % 100) / 100) * 0.2;
      const rVar = ((h >> 8) % 6) - 2;
      const r = ring.radius + rVar;
      addTree(group, cx, cz, groundY, r, angle, ring.trunkMin + (h % ring.trunkRange),
        ring.canopyMin + ((h >> 4) % ring.canopyRange), h, trunkGeo, trunkMat);
    }
  }

  // --- Scattered trees extending to the horizon ---
  // Multiple bands of increasing radius, decreasing density, hazier colors
  const bands = [
    { rMin: innerRing + 18, rMax: innerRing + 50, count: 60, colorIdx: 0 },
    { rMin: innerRing + 50, rMax: innerRing + 120, count: 80, colorIdx: 1 },
    { rMin: innerRing + 120, rMax: innerRing + 250, count: 100, colorIdx: 2 },
  ];

  // Haze-shifted canopy colors for distant trees
  const distantCanopy = [
    [0x4D8C2A, 0x3D7522, 0x5A9E35, 0x438028],    // near: vivid
    [0x5A8A40, 0x4E7E38, 0x629A48, 0x558A3A],     // mid: slightly muted
    [0x6A9A58, 0x5E8E50, 0x72A260, 0x659A52],     // far: hazy green
  ];

  for (const band of bands) {
    for (let i = 0; i < band.count; i++) {
      const h = simpleHash(i + band.count * 200 + band.rMin);
      const angle = (h % 10000) / 10000 * Math.PI * 2;
      const r = band.rMin + ((h >> 6) % (band.rMax - band.rMin));
      const trunkH = 6 + (h % 14);
      const canopyR = 4 + ((h >> 4) % 6);
      const colors = distantCanopy[band.colorIdx];
      const canopyColor = colors[(h >> 2) % colors.length];

      addTreeWithColor(group, cx, cz, groundY, r, angle, trunkH, canopyR, h, trunkGeo, trunkMat, canopyColor);
    }
  }

  return group;
}

/** Place a tree at polar coordinates from center */
function addTree(
  group: THREE.Group,
  cx: number, cz: number, groundY: number,
  r: number, angle: number, trunkH: number, canopyR: number,
  h: number, trunkGeo: THREE.CylinderGeometry, trunkMat: THREE.MeshLambertMaterial,
) {
  const canopyColor = CANOPY_COLORS[(h >> 2) % CANOPY_COLORS.length];
  addTreeWithColor(group, cx, cz, groundY, r, angle, trunkH, canopyR, h, trunkGeo, trunkMat, canopyColor);
}

function addTreeWithColor(
  group: THREE.Group,
  cx: number, cz: number, groundY: number,
  r: number, angle: number, trunkH: number, canopyR: number,
  h: number, trunkGeo: THREE.CylinderGeometry, trunkMat: THREE.MeshLambertMaterial,
  canopyColor: number,
) {
  const tx = cx + r * Math.cos(angle);
  const tz = cz + r * Math.sin(angle);

  // Group trunk + canopy so culling can toggle the whole tree
  const tree = new THREE.Group();
  tree.userData.treeAngle = angle;

  const trunk = new THREE.Mesh(trunkGeo, trunkMat);
  trunk.scale.set(1, trunkH, 1);
  trunk.position.set(tx, groundY + trunkH / 2, tz);
  tree.add(trunk);

  const canopyMat = new THREE.MeshLambertMaterial({ color: canopyColor });
  if (h % 4 === 0) {
    const geo = new THREE.ConeGeometry(canopyR, canopyR + 2, 6);
    const mesh = new THREE.Mesh(geo, canopyMat);
    mesh.position.set(tx, groundY + trunkH + (canopyR + 2) / 2 - 1, tz);
    tree.add(mesh);
  } else {
    const geo = new THREE.SphereGeometry(canopyR, 5, 4);
    const mesh = new THREE.Mesh(geo, canopyMat);
    mesh.position.set(tx, groundY + trunkH + canopyR * 0.4, tz);
    tree.add(mesh);
  }

  group.add(tree);
}
