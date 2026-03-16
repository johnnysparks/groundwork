/**
 * 3D garden gnome model builder.
 *
 * Creates a charming toylike gnome from soft primitives:
 * body, head, hat, arms (with hands), legs/boots, beard, belt,
 * eyes, nose, and tools. Each part is a named child so the
 * animation system can drive transforms per-frame.
 *
 * Proportions are deliberately chunky and cute:
 * oversized head, stubby limbs, big boots. Think garden
 * gnome figurine crossed with a Lego minifig.
 */

import * as THREE from 'three';
import { sphere, ellipsoid, box, cone, cylinder, taperCylinder, warmMat } from './primitives';

// ─── Colors ─────────────────────────────────────────────────────

const TUNIC     = 0x527A40; // earthy green
const TUNIC_DARK = 0x426830; // darker for sleeves
const HAT       = 0xB83020; // warm red
const HAT_DARK  = 0x982818; // hat shadow
const SKIN      = 0xD9B48F; // warm peachy
const BOOT      = 0x5A3820; // dark brown
const BELT      = 0x4D3218; // belt brown
const BUCKLE    = 0xD9C060; // gold
const BEARD     = 0xD1C8B8; // cream
const EYE       = 0x181008; // near black
const NOSE      = 0xC89878; // slightly darker skin
const CHEEK     = 0xE09088; // rosy

// Tool colors
const SHOVEL_HANDLE = 0x7A5A30;
const SHOVEL_BLADE  = 0x808890;
const WATER_CAN     = 0x5080B0;
const SEED_BAG      = 0x8A6838;

// ─── Gnome Parts ────────────────────────────────────────────────

export interface GnomeParts {
  root: THREE.Group;
  body: THREE.Mesh;
  head: THREE.Group;     // contains head sphere, eyes, nose, mouth, beard
  hat: THREE.Mesh;
  armL: THREE.Group;     // pivot at shoulder, contains upper arm + hand
  armR: THREE.Group;
  bootL: THREE.Mesh;
  bootR: THREE.Mesh;
  belt: THREE.Mesh;
  buckle: THREE.Mesh;
  cheekL: THREE.Mesh;
  cheekR: THREE.Mesh;
  eyeL: THREE.Mesh;
  eyeR: THREE.Mesh;
  // Tools (hidden by default, shown based on state)
  shovel: THREE.Group;
  waterCan: THREE.Group;
  seedBag: THREE.Group;
}

/**
 * Build the full 3D gnome model. Returns named parts for animation.
 * The model is ~4 units tall (matching the previous billboard scale).
 */
export function buildGnomeModel(): GnomeParts {
  const root = new THREE.Group();
  root.name = 'gnome';

  // Scale: gnome is about 4 units tall
  // Body center at y=0, head at y~1.5, boots at y~-1.8, hat tip at y~3

  // ─── Body (tunic) ──────────────────────────────────
  const body = ellipsoid(root, 0, 0, 0, 0.7, 0.9, 0.5, TUNIC);
  body.name = 'body';

  // ─── Head ──────────────────────────────────────────
  const head = new THREE.Group();
  head.name = 'head';
  head.position.set(0, 1.2, 0);
  root.add(head);

  // Head sphere
  sphere(head, 0, 0, 0, 0.55, SKIN);

  // Eyes
  const eyeL = sphere(head, -0.22, 0.08, -0.42, 0.08, EYE);
  eyeL.name = 'eyeL';
  const eyeR = sphere(head, 0.22, 0.08, -0.42, 0.08, EYE);
  eyeR.name = 'eyeR';

  // Eye whites (behind pupils)
  sphere(head, -0.22, 0.08, -0.38, 0.1, 0xF0E8E0);
  sphere(head, 0.22, 0.08, -0.38, 0.1, 0xF0E8E0);

  // Nose (round ball)
  sphere(head, 0, -0.05, -0.5, 0.1, NOSE);

  // Rosy cheeks
  const cheekL = sphere(head, -0.35, -0.08, -0.3, 0.12, CHEEK);
  cheekL.name = 'cheekL';
  cheekL.visible = true;
  const cheekR = sphere(head, 0.35, -0.08, -0.3, 0.12, CHEEK);
  cheekR.name = 'cheekR';
  cheekR.visible = true;

  // Mouth (tiny dark ellipse, neutral position)
  ellipsoid(head, 0, -0.2, -0.45, 0.12, 0.04, 0.05, 0x905050);

  // Beard (half-sphere below chin)
  ellipsoid(head, 0, -0.35, -0.2, 0.35, 0.3, 0.3, BEARD);

  // ─── Hat ───────────────────────────────────────────
  const hat = cone(root, 0, 2.2, 0, 0.55, 1.2, HAT);
  hat.name = 'hat';

  // Hat brim (disk at base of cone)
  ellipsoid(root, 0, 1.6, 0, 0.65, 0.1, 0.65, HAT_DARK);

  // Hat tip ball
  sphere(root, 0, 2.85, 0, 0.1, HAT);

  // ─── Arms ──────────────────────────────────────────
  // Each arm is a Group pivoting at the shoulder
  const armL = new THREE.Group();
  armL.name = 'armL';
  armL.position.set(-0.75, 0.5, 0); // shoulder position
  root.add(armL);

  // Upper arm (cylinder pointing down from pivot)
  cylinder(armL, 0, -0.35, 0, 0.12, 0.5, TUNIC_DARK);
  // Hand (sphere at end)
  sphere(armL, 0, -0.7, 0, 0.15, SKIN);

  const armR = new THREE.Group();
  armR.name = 'armR';
  armR.position.set(0.75, 0.5, 0);
  root.add(armR);

  cylinder(armR, 0, -0.35, 0, 0.12, 0.5, TUNIC_DARK);
  sphere(armR, 0, -0.7, 0, 0.15, SKIN);

  // ─── Boots ─────────────────────────────────────────
  const bootL = ellipsoid(root, -0.3, -1.5, -0.05, 0.28, 0.25, 0.35, BOOT);
  bootL.name = 'bootL';
  const bootR = ellipsoid(root, 0.3, -1.5, -0.05, 0.28, 0.25, 0.35, BOOT);
  bootR.name = 'bootR';

  // ─── Belt ──────────────────────────────────────────
  const belt = ellipsoid(root, 0, -0.4, 0, 0.72, 0.1, 0.52, BELT);
  belt.name = 'belt';
  const buckle = box(root, 0, -0.4, -0.52, 0.15, 0.12, 0.05, BUCKLE);
  buckle.name = 'buckle';

  // ─── Tools (all hidden by default) ─────────────────
  const shovel = buildShovel();
  shovel.visible = false;
  armR.add(shovel);

  const waterCan = buildWaterCan();
  waterCan.visible = false;
  armR.add(waterCan);

  const seedBag = buildSeedBag();
  seedBag.visible = false;
  armR.add(seedBag);

  return {
    root, body, head, hat, armL, armR, bootL, bootR,
    belt, buckle, cheekL, cheekR, eyeL, eyeR,
    shovel, waterCan, seedBag,
  };
}

// ─── Tool Builders ──────────────────────────────────────────────

function buildShovel(): THREE.Group {
  const g = new THREE.Group();
  g.name = 'shovel';
  g.position.set(0, -0.7, -0.15); // positioned at hand

  // Handle
  cylinder(g, 0, 0.3, 0, 0.04, 0.8, SHOVEL_HANDLE);
  // Blade
  ellipsoid(g, 0, -0.15, 0, 0.15, 0.2, 0.05, SHOVEL_BLADE);

  return g;
}

function buildWaterCan(): THREE.Group {
  const g = new THREE.Group();
  g.name = 'waterCan';
  g.position.set(0, -0.7, -0.15);

  // Can body
  box(g, 0, 0, 0, 0.25, 0.2, 0.18, WATER_CAN);
  // Spout
  cylinder(g, 0.18, 0.1, 0, 0.04, 0.2, WATER_CAN);
  g.children[g.children.length - 1].rotation.z = -0.8;
  // Handle on top
  box(g, 0, 0.14, 0, 0.18, 0.04, 0.04, WATER_CAN);

  return g;
}

function buildSeedBag(): THREE.Group {
  const g = new THREE.Group();
  g.name = 'seedBag';
  g.position.set(0, -0.7, -0.15);

  // Bag body (rounded)
  sphere(g, 0, 0, 0, 0.18, SEED_BAG);
  // Cinched top
  cylinder(g, 0, 0.15, 0, 0.06, 0.1, SEED_BAG);
  // Seeds poking out (tiny colored dots)
  sphere(g, -0.05, 0.2, 0, 0.04, 0x80A030);
  sphere(g, 0.05, 0.18, 0.03, 0.04, 0x90B040);

  return g;
}
