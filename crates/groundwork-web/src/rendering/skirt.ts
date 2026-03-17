/**
 * Ground skirt and decorative forest ring.
 *
 * The skirt hides the underground cross-section. The forest ring uses
 * charming soft-voxel tree models placed around the diorama edge to
 * create a "glen in the forest" feel. Each tree is a unique 3D model
 * built from rounded primitives (oaks, pines, birches, bushes,
 * flowering trees, willows) — like an asset pack in a cozy builder.
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GROUND_LEVEL } from '../bridge';
import { buildRandomTree } from '../models/trees';
import { hashFloat, hashRange, hash } from '../models/primitives';

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
 */
export function buildSkirtMesh(): SkirtResult {
  const group = new THREE.Group();
  group.name = 'ground_skirt';

  const gx = GRID_X;
  const gz = GRID_Y;
  const top = GROUND_LEVEL;
  const deep = -60;
  const cx = gx / 2;
  const cz = gz / 2;

  // Soil strata colors — darkened so the walls recede behind the garden.
  // The top edge blends toward the meadow green for a softer transition.
  const topsoil = [0.28, 0.22, 0.14];
  const subsoil = [0.34, 0.26, 0.16];
  const clay    = [0.38, 0.30, 0.20];
  const bedrock = [0.30, 0.26, 0.22];

  function strataColor(y: number): [number, number, number] {
    const t = (y - deep) / (top - deep);
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
    const mat = new THREE.MeshLambertMaterial({
      vertexColors: true,
      side: THREE.DoubleSide,
      emissive: new THREE.Color(0x2A1A10),
      emissiveIntensity: 0.2,
    });
    const mesh = new THREE.Mesh(geo, mat);
    mesh.receiveShadow = true;
    return mesh;
  }

  const walls: SkirtWall[] = [
    { mesh: buildWall([0, deep, 0], [gx, deep, 0], [gx, top, 0], [0, top, 0], [0, 0, -1]),       normalXZ: [0, -1] },
    { mesh: buildWall([gx, deep, gz], [0, deep, gz], [0, top, gz], [gx, top, gz], [0, 0, 1]),      normalXZ: [0, 1] },
    { mesh: buildWall([0, deep, gz], [0, deep, 0], [0, top, 0], [0, top, gz], [-1, 0, 0]),         normalXZ: [-1, 0] },
    { mesh: buildWall([gx, deep, 0], [gx, deep, gz], [gx, top, gz], [gx, top, 0], [1, 0, 0]),     normalXZ: [1, 0] },
  ];

  for (const w of walls) group.add(w.mesh);

  // Grass lip along the top edge — a thin overhang of mossy green that
  // softens the transition from garden to the underground cross-section.
  const lipH = 0.8;  // slightly thick for visibility
  const lipD = 1.5;  // overhang distance outward
  const lipMat = new THREE.MeshLambertMaterial({
    color: 0x3A6820,
    emissive: 0x1A3410,
    emissiveIntensity: 0.3,
  });
  // Four lip strips (one per wall edge)
  const lipDefs: [number, number, number, number, number][] = [
    // [x, y(=top), z, width, depth] — positioned at each wall top edge
    [cx, top, -lipD / 2, gx + lipD * 2, lipD],           // front (z=0)
    [cx, top, gz + lipD / 2, gx + lipD * 2, lipD],       // back (z=gz)
    [-lipD / 2, top, cz, lipD, gz],                       // left (x=0)
    [gx + lipD / 2, top, cz, lipD, gz],                   // right (x=gx)
  ];
  for (const [lx, ly, lz, lw, ld] of lipDefs) {
    const geo = new THREE.BoxGeometry(lw, lipH, ld);
    const lip = new THREE.Mesh(geo, lipMat);
    lip.position.set(lx, ly + lipH / 2, lz);
    group.add(lip);
  }

  // Bottom cap
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
  const capMesh = new THREE.Mesh(capGeo, new THREE.MeshBasicMaterial({ vertexColors: true }));
  capMesh.receiveShadow = true;
  group.add(capMesh);

  // Underground floor plane
  const underRadius = 400;
  const underGeo = new THREE.CircleGeometry(underRadius, 32);
  const underMat = new THREE.MeshBasicMaterial({ color: 0x4A3A2E });
  const underMesh = new THREE.Mesh(underGeo, underMat);
  underMesh.rotation.x = -Math.PI / 2;
  underMesh.position.set(cx, deep, cz);
  group.add(underMesh);

  return { group, walls };
}

/**
 * Update forest ring visibility: hide trees between the camera and the garden.
 */
export function updateForestCulling(group: THREE.Group, cameraTheta: number): void {
  const camDirX = Math.cos(cameraTheta);
  const camDirZ = Math.sin(cameraTheta);

  group.traverse((obj) => {
    const angle = obj.userData.treeAngle as number | undefined;
    if (angle === undefined) return;

    const treeDirX = Math.cos(angle);
    const treeDirZ = Math.sin(angle);
    const dot = camDirX * treeDirX + camDirZ * treeDirZ;
    obj.visible = dot < 0.5;
  });
}

/**
 * Build the full glade landscape: meadow ground plane, dense forest ring
 * with charming varied tree models, and scattered trees fading to distance.
 */
export function buildForestRing(): THREE.Group {
  const group = new THREE.Group();
  group.name = 'glade_environment';

  const cx = GRID_X / 2;
  const cz = GRID_Y / 2;
  const groundY = GROUND_LEVEL + 0.5;
  const innerRing = Math.max(cx, cz) + 12;

  // ─── Meadow ground plane with garden cutout ───────
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
  const groundMat = new THREE.MeshLambertMaterial({ color: 0x3A8828 });
  const ground = new THREE.Mesh(groundGeo, groundMat);
  ground.name = 'meadow_ground';
  ground.rotation.x = -Math.PI / 2;
  ground.position.set(cx, groundY + 0.1, cz);
  ground.receiveShadow = true;
  group.add(ground);

  // ─── Small ground details (rocks, mushrooms) near garden ───
  addGroundDetails(group, cx, cz, groundY, innerRing);

  // ─── Dense forest ring (charming 3D tree models) ──────────
  const nearRings = [
    { radius: innerRing, count: 32 },
    { radius: innerRing + 8, count: 40 },
  ];

  for (const ring of nearRings) {
    for (let i = 0; i < ring.count; i++) {
      const seed = hash(i + ring.count * 100);
      const angle = (i / ring.count) * Math.PI * 2 + hashFloat(seed) * 0.2;
      const rVar = hashRange(seed + 1, -2, 4);
      const r = ring.radius + rVar;

      const tree = buildRandomTree(seed, 0);
      const tx = cx + r * Math.cos(angle);
      const tz = cz + r * Math.sin(angle);
      tree.position.set(tx, groundY, tz);
      tree.userData.treeAngle = angle;

      // Slight random Y rotation for variety
      tree.rotation.y = hashRange(seed + 50, 0, Math.PI * 2);

      group.add(tree);
    }
  }

  // ─── Scattered trees extending to horizon ─────────────────
  const bands = [
    { rMin: innerRing + 18, rMax: innerRing + 50, count: 60, haze: 0 },
    { rMin: innerRing + 50, rMax: innerRing + 120, count: 80, haze: 1 },
    { rMin: innerRing + 120, rMax: innerRing + 250, count: 100, haze: 2 },
  ];

  for (const band of bands) {
    for (let i = 0; i < band.count; i++) {
      const seed = hash(i + band.count * 200 + band.rMin);
      const angle = hashFloat(seed) * Math.PI * 2;
      const r = band.rMin + hashRange(seed + 1, 0, band.rMax - band.rMin);

      // Scale increases slightly with distance for visual fill
      const distScale = 0.9 + hashRange(seed + 2, 0, 0.8);
      const tree = buildRandomTree(seed, band.haze);
      tree.scale.setScalar(distScale);

      const tx = cx + r * Math.cos(angle);
      const tz = cz + r * Math.sin(angle);
      tree.position.set(tx, groundY, tz);
      tree.userData.treeAngle = angle;
      tree.rotation.y = hashRange(seed + 50, 0, Math.PI * 2);

      group.add(tree);
    }
  }

  return group;
}

// ─── Ground Details ─────────────────────────────────────────────

/** Add small decorative objects between garden and forest ring. */
function addGroundDetails(
  group: THREE.Group,
  cx: number, cz: number,
  groundY: number,
  innerRing: number,
): void {
  const detailRing = innerRing - 4;

  // Scatter rocks
  for (let i = 0; i < 15; i++) {
    const seed = hash(i + 5000);
    const angle = hashFloat(seed) * Math.PI * 2;
    const r = detailRing + hashRange(seed + 1, -3, 8);
    const x = cx + r * Math.cos(angle);
    const z = cz + r * Math.sin(angle);

    const rockGroup = new THREE.Group();
    rockGroup.position.set(x, groundY, z);

    // Random rock shape: either a flattened sphere or a small boulder cluster
    const rockSize = 0.5 + hashFloat(seed + 2) * 1.0;
    const rockColor = hashFloat(seed + 3) > 0.5 ? 0x8A8078 : 0x706860;

    const _sphereGeo = new THREE.SphereGeometry(1, 5, 4);
    const rockMat = new THREE.MeshLambertMaterial({
      color: rockColor,
      emissive: rockColor,
      emissiveIntensity: 0.2,
    });
    const rock = new THREE.Mesh(_sphereGeo, rockMat);
    rock.scale.set(rockSize, rockSize * 0.5, rockSize * 0.8);
    rock.rotation.y = hashRange(seed + 4, 0, Math.PI * 2);
    rockGroup.add(rock);

    // Sometimes add a smaller rock next to it
    if (hashFloat(seed + 5) > 0.5) {
      const r2 = new THREE.Mesh(_sphereGeo, rockMat);
      const s2 = rockSize * 0.5;
      r2.scale.set(s2, s2 * 0.5, s2 * 0.7);
      r2.position.set(rockSize * 0.6, 0, rockSize * 0.3);
      rockGroup.add(r2);
    }

    group.add(rockGroup);
  }

  // Scatter small mushroom clusters
  for (let i = 0; i < 8; i++) {
    const seed = hash(i + 6000);
    const angle = hashFloat(seed) * Math.PI * 2;
    const r = detailRing + hashRange(seed + 1, -2, 6);
    const x = cx + r * Math.cos(angle);
    const z = cz + r * Math.sin(angle);

    const mushGroup = new THREE.Group();
    mushGroup.position.set(x, groundY, z);

    const count = 1 + Math.floor(hashFloat(seed + 2) * 3);
    for (let j = 0; j < count; j++) {
      const mx = hashRange(seed + 10 + j, -0.3, 0.3);
      const mz = hashRange(seed + 20 + j, -0.3, 0.3);
      const mh = 0.3 + hashFloat(seed + 30 + j) * 0.4;
      const capR = 0.15 + hashFloat(seed + 40 + j) * 0.2;

      // Stem
      const stemGeo = new THREE.CylinderGeometry(0.06, 0.08, mh, 5);
      const stemMat = new THREE.MeshLambertMaterial({ color: 0xE8D8C0, emissive: 0xE8D8C0, emissiveIntensity: 0.2 });
      const stem = new THREE.Mesh(stemGeo, stemMat);
      stem.position.set(mx, mh / 2, mz);
      mushGroup.add(stem);

      // Cap
      const capColor = hashFloat(seed + 50 + j) > 0.5 ? 0xC84030 : 0xB07040;
      const capGeo = new THREE.SphereGeometry(capR, 6, 4, 0, Math.PI * 2, 0, Math.PI / 2);
      const capMat = new THREE.MeshLambertMaterial({ color: capColor, emissive: capColor, emissiveIntensity: 0.25 });
      const cap = new THREE.Mesh(capGeo, capMat);
      cap.position.set(mx, mh, mz);
      mushGroup.add(cap);
    }

    group.add(mushGroup);
  }

  // Scatter small flower patches
  for (let i = 0; i < 10; i++) {
    const seed = hash(i + 7000);
    const angle = hashFloat(seed) * Math.PI * 2;
    const r = detailRing + hashRange(seed + 1, -5, 10);
    const x = cx + r * Math.cos(angle);
    const z = cz + r * Math.sin(angle);

    const patchGroup = new THREE.Group();
    patchGroup.position.set(x, groundY, z);

    const flowerCount = 2 + Math.floor(hashFloat(seed + 2) * 4);
    const petalColors = [0xE8A0B0, 0xE0D050, 0xB0B8E0, 0xF0E8D0];
    const petalColor = petalColors[seed & 3];

    for (let j = 0; j < flowerCount; j++) {
      const fx = hashRange(seed + 10 + j, -0.5, 0.5);
      const fz = hashRange(seed + 20 + j, -0.5, 0.5);
      const fh = 0.4 + hashFloat(seed + 30 + j) * 0.3;

      // Stem
      const stemGeo = new THREE.CylinderGeometry(0.02, 0.02, fh, 4);
      const stemMat = new THREE.MeshLambertMaterial({ color: 0x4A8030, emissive: 0x4A8030, emissiveIntensity: 0.2 });
      const stem = new THREE.Mesh(stemGeo, stemMat);
      stem.position.set(fx, fh / 2, fz);
      patchGroup.add(stem);

      // Flower head (tiny sphere)
      const headGeo = new THREE.SphereGeometry(0.08, 5, 4);
      const headMat = new THREE.MeshLambertMaterial({ color: petalColor, emissive: petalColor, emissiveIntensity: 0.3 });
      const head = new THREE.Mesh(headGeo, headMat);
      head.position.set(fx, fh + 0.05, fz);
      patchGroup.add(head);
    }

    group.add(patchGroup);
  }
}
