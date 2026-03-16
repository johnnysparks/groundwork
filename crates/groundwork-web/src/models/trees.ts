/**
 * Charming tree model builders for the scene skirt.
 *
 * Each tree type creates a Three.js Group of soft primitives
 * with warm colors and toylike proportions. Designed to feel
 * like hand-placed diorama pieces from an asset pack.
 *
 * Tree types:
 *  0 - Round Oak: thick trunk, 2-3 overlapping canopy spheres
 *  1 - Tall Pine: thin trunk, 2-3 stacked cones
 *  2 - Birch: thin white trunk, small oval canopy
 *  3 - Bush: no visible trunk, wide round blob
 *  4 - Flowering: round canopy with pink accent blobs
 *  5 - Willow: thick trunk, dome + hanging drapes
 */

import * as THREE from 'three';
import { sphere, ellipsoid, cone, taperCylinder, cylinder, warmMat, hashFloat, hashRange } from './primitives';

// ─── Color Palettes ─────────────────────────────────────────────

// Trunk palettes
const TRUNK_BROWN   = 0x5C3A1E;
const TRUNK_DARK    = 0x4A2E15;
const TRUNK_BIRCH   = 0xD4C8B0;
const TRUNK_WILLOW  = 0x6B4E2A;

// Canopy palettes (warm, saturated)
const CANOPY_GREEN1 = 0x4D8C2A; // rich forest green
const CANOPY_GREEN2 = 0x5A9E35; // bright spring green
const CANOPY_GREEN3 = 0x3D7522; // deep green
const CANOPY_GREEN4 = 0x6AAE40; // lime-ish
const CANOPY_DARK   = 0x2E6318; // pine dark
const CANOPY_PINE   = 0x2A5E1A; // blue-ish pine
const CANOPY_WILLOW = 0x5E9930; // yellow-green willow

// Accent colors
const FLOWER_PINK   = 0xE8A0B0;
const FLOWER_WHITE  = 0xF0E8D8;
const FLOWER_YELLOW = 0xE8D060;
const BERRY_RED     = 0xC04040;

// Distant haze colors (progressively more muted)
const HAZE_CANOPY = [
  [CANOPY_GREEN1, CANOPY_GREEN2, CANOPY_GREEN3, CANOPY_GREEN4],   // near: vivid
  [0x5A8A40, 0x4E7E38, 0x629A48, 0x558A3A],                      // mid: muted
  [0x6A9A58, 0x5E8E50, 0x72A260, 0x659A52],                      // far: hazy
];

const HAZE_TRUNK = [TRUNK_BROWN, 0x6B5030, 0x7A6040];

export { HAZE_CANOPY, HAZE_TRUNK };

/**
 * Build a charming tree model. Returns a Group positioned at origin (0,0,0)
 * — caller sets final world position.
 *
 * @param treeType 0-5 (see header), or -1 for random based on seed
 * @param seed deterministic variation seed
 * @param scale overall size multiplier (1.0 = normal forest tree)
 * @param hazeLevel 0=near/vivid, 1=mid, 2=far/hazy (shifts colors)
 */
export function buildTree(
  treeType: number,
  seed: number,
  scale: number = 1.0,
  hazeLevel: number = 0,
): THREE.Group {
  const group = new THREE.Group();
  const s = scale;

  // Pick colors based on haze level
  const canopyPalette = HAZE_CANOPY[Math.min(hazeLevel, 2)];
  const trunkColor = HAZE_TRUNK[Math.min(hazeLevel, 2)];
  const canopyColor = canopyPalette[seed & 3];

  switch (treeType) {
    case 0: buildOak(group, s, seed, canopyColor, trunkColor); break;
    case 1: buildPine(group, s, seed, trunkColor); break;
    case 2: buildBirch(group, s, seed, canopyColor); break;
    case 3: buildBush(group, s, seed, canopyColor); break;
    case 4: buildFlowering(group, s, seed, canopyColor, trunkColor); break;
    case 5: buildWillow(group, s, seed, trunkColor); break;
    default: buildOak(group, s, seed, canopyColor, trunkColor); break;
  }

  return group;
}

// ─── Tree Builders ──────────────────────────────────────────────

function buildOak(g: THREE.Group, s: number, seed: number, canopyColor: number, trunkColor: number): void {
  const trunkH = (8 + hashRange(seed, 0, 6)) * s;
  const trunkR = (0.8 + hashRange(seed + 1, 0, 0.4)) * s;
  const canopyR = (4 + hashRange(seed + 2, 0, 3)) * s;

  // Thick tapered trunk
  taperCylinder(g, 0, trunkH / 2, 0, trunkR, trunkH, trunkColor);

  // 2-3 overlapping canopy spheres for organic shape
  const mainY = trunkH + canopyR * 0.5;
  sphere(g, 0, mainY, 0, canopyR, canopyColor);

  // Offset satellite blobs
  const offX = hashRange(seed + 3, -1, 1) * canopyR * 0.4;
  const offZ = hashRange(seed + 4, -1, 1) * canopyR * 0.4;
  sphere(g, offX * s, mainY - canopyR * 0.2, offZ * s, canopyR * 0.75, canopyColor);

  if (hashFloat(seed + 5) > 0.4) {
    const offX2 = hashRange(seed + 6, -1, 1) * canopyR * 0.5;
    const offZ2 = hashRange(seed + 7, -1, 1) * canopyR * 0.5;
    sphere(g, offX2 * s, mainY + canopyR * 0.25, offZ2 * s, canopyR * 0.6, canopyColor);
  }

  // Exposed root bumps at base
  if (hashFloat(seed + 8) > 0.5) {
    const rootAngle = hashRange(seed + 9, 0, Math.PI * 2);
    const rootR = trunkR * 0.6;
    ellipsoid(g,
      Math.cos(rootAngle) * trunkR * 1.2, rootR * 0.3, Math.sin(rootAngle) * trunkR * 1.2,
      rootR, rootR * 0.5, rootR,
      trunkColor,
    );
  }
}

function buildPine(g: THREE.Group, s: number, seed: number, trunkColor: number): void {
  const trunkH = (10 + hashRange(seed, 0, 8)) * s;
  const trunkR = (0.5 + hashRange(seed + 1, 0, 0.3)) * s;
  const baseR = (3.5 + hashRange(seed + 2, 0, 2)) * s;

  // Thin straight trunk
  cylinder(g, 0, trunkH / 2, 0, trunkR, trunkH, trunkColor);

  // 2-3 stacked cones, getting smaller toward top
  const layers = 2 + (hashFloat(seed + 3) > 0.5 ? 1 : 0);
  const layerH = trunkH * 0.35;
  const startY = trunkH * 0.35;

  for (let i = 0; i < layers; i++) {
    const t = i / layers;
    const r = baseR * (1.0 - t * 0.35);
    const h = layerH * (1.0 - t * 0.2);
    const y = startY + i * (layerH * 0.65);
    const pineColor = i === 0 ? CANOPY_DARK : CANOPY_PINE;
    cone(g, 0, y + h / 2, 0, r, h, pineColor);
  }

  // Pointed tip
  const tipY = startY + layers * (layerH * 0.65);
  cone(g, 0, tipY + layerH * 0.3, 0, baseR * 0.25, layerH * 0.6, CANOPY_PINE);
}

function buildBirch(g: THREE.Group, s: number, seed: number, canopyColor: number): void {
  const trunkH = (9 + hashRange(seed, 0, 5)) * s;
  const trunkR = (0.35 + hashRange(seed + 1, 0, 0.15)) * s;
  const canopyR = (3 + hashRange(seed + 2, 0, 2)) * s;

  // Thin white trunk
  cylinder(g, 0, trunkH / 2, 0, trunkR, trunkH, TRUNK_BIRCH);

  // Oval canopy (taller than wide)
  const mainY = trunkH + canopyR * 0.3;
  ellipsoid(g, 0, mainY, 0, canopyR * 0.8, canopyR * 1.1, canopyR * 0.8, canopyColor);

  // Small accent blob
  if (hashFloat(seed + 3) > 0.3) {
    const aX = hashRange(seed + 4, -1, 1) * canopyR * 0.3;
    const aZ = hashRange(seed + 5, -1, 1) * canopyR * 0.3;
    sphere(g, aX * s, mainY + canopyR * 0.5, aZ * s, canopyR * 0.4, canopyColor);
  }

  // Birch bark marks (small dark rings on trunk)
  const markCount = 2 + Math.floor(hashFloat(seed + 6) * 3);
  for (let i = 0; i < markCount; i++) {
    const markY = trunkH * (0.2 + i * 0.25);
    cylinder(g, 0, markY, 0, trunkR * 1.05, trunkR * 0.3, 0x8A7A68);
  }
}

function buildBush(g: THREE.Group, s: number, seed: number, canopyColor: number): void {
  const bushR = (2.5 + hashRange(seed, 0, 2)) * s;
  const bushH = bushR * (0.6 + hashRange(seed + 1, 0, 0.3));

  // No visible trunk, just rounded blob cluster
  ellipsoid(g, 0, bushH * 0.6, 0, bushR, bushH, bushR, canopyColor);

  // 1-2 accent blobs for organic shape
  const blobX = hashRange(seed + 2, -1, 1) * bushR * 0.4;
  const blobZ = hashRange(seed + 3, -1, 1) * bushR * 0.4;
  sphere(g, blobX, bushH * 0.8, blobZ, bushR * 0.5, canopyColor);

  // Berry clusters on some bushes
  if (hashFloat(seed + 4) > 0.6) {
    const berryCount = 3 + Math.floor(hashFloat(seed + 5) * 4);
    for (let i = 0; i < berryCount; i++) {
      const angle = hashRange(seed + 10 + i, 0, Math.PI * 2);
      const r = bushR * (0.6 + hashFloat(seed + 20 + i) * 0.3);
      const bx = Math.cos(angle) * r;
      const bz = Math.sin(angle) * r;
      const by = bushH * (0.4 + hashFloat(seed + 30 + i) * 0.4);
      sphere(g, bx, by, bz, 0.25 * s, BERRY_RED);
    }
  }
}

function buildFlowering(g: THREE.Group, s: number, seed: number, canopyColor: number, trunkColor: number): void {
  const trunkH = (6 + hashRange(seed, 0, 4)) * s;
  const trunkR = (0.6 + hashRange(seed + 1, 0, 0.3)) * s;
  const canopyR = (3.5 + hashRange(seed + 2, 0, 2)) * s;

  // Trunk
  taperCylinder(g, 0, trunkH / 2, 0, trunkR, trunkH, trunkColor);

  // Round canopy
  const mainY = trunkH + canopyR * 0.4;
  sphere(g, 0, mainY, 0, canopyR, canopyColor);

  // Flower accent blobs scattered on canopy surface
  const flowerColors = [FLOWER_PINK, FLOWER_WHITE, FLOWER_YELLOW];
  const flowerColor = flowerColors[seed % 3];
  const flowerCount = 4 + Math.floor(hashFloat(seed + 3) * 5);

  for (let i = 0; i < flowerCount; i++) {
    const phi = hashRange(seed + 10 + i, 0, Math.PI * 2);
    const theta = hashRange(seed + 20 + i, 0.2, 1.2);
    const fx = Math.sin(theta) * Math.cos(phi) * canopyR * 0.9;
    const fy = Math.cos(theta) * canopyR * 0.7;
    const fz = Math.sin(theta) * Math.sin(phi) * canopyR * 0.9;
    sphere(g, fx, mainY + fy, fz, (0.4 + hashFloat(seed + 30 + i) * 0.4) * s, flowerColor);
  }
}

function buildWillow(g: THREE.Group, s: number, seed: number, trunkColor: number): void {
  const trunkH = (8 + hashRange(seed, 0, 5)) * s;
  const trunkR = (0.9 + hashRange(seed + 1, 0, 0.4)) * s;
  const canopyR = (4.5 + hashRange(seed + 2, 0, 2)) * s;

  // Thick trunk, slightly curved via offset top
  taperCylinder(g, 0, trunkH / 2, 0, trunkR, trunkH, trunkColor);

  // Dome canopy (wider than tall)
  const mainY = trunkH + canopyR * 0.2;
  ellipsoid(g, 0, mainY, 0, canopyR, canopyR * 0.7, canopyR, CANOPY_WILLOW);

  // Hanging drape cylinders (willow fronds)
  const drapCount = 6 + Math.floor(hashFloat(seed + 3) * 4);
  for (let i = 0; i < drapCount; i++) {
    const angle = (i / drapCount) * Math.PI * 2 + hashRange(seed + 10 + i, -0.2, 0.2);
    const r = canopyR * (0.65 + hashFloat(seed + 20 + i) * 0.3);
    const dx = Math.cos(angle) * r;
    const dz = Math.sin(angle) * r;
    const drapeH = (3 + hashRange(seed + 30 + i, 0, 4)) * s;
    const drapeY = mainY - drapeH * 0.3;
    cylinder(g, dx, drapeY, dz, 0.35 * s, drapeH, CANOPY_WILLOW);
  }
}

// ─── Scatter Helpers ────────────────────────────────────────────

/** Pick a tree type from seed with weighted distribution. */
export function pickTreeType(seed: number): number {
  const r = hashFloat(seed);
  if (r < 0.30) return 0; // oak (most common)
  if (r < 0.50) return 1; // pine
  if (r < 0.65) return 2; // birch
  if (r < 0.78) return 3; // bush
  if (r < 0.90) return 4; // flowering
  return 5;               // willow
}

/** Build a tree with scale variation from seed. */
export function buildRandomTree(seed: number, hazeLevel: number = 0): THREE.Group {
  const type = pickTreeType(seed);
  const scale = 0.8 + hashRange(seed + 99, 0, 0.6);
  return buildTree(type, seed, scale, hazeLevel);
}
