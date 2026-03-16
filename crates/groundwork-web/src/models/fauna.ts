/**
 * 3D fauna model builders.
 *
 * Each fauna type gets a charming toylike 3D model built from
 * soft primitives. These replace the flat billboard sprites with
 * actual geometry that reads as part of the diorama world.
 *
 * Models are small but deliberately oversized for readability
 * at the garden's viewing distance. Think "collectible figurine"
 * more than "anatomically correct insect."
 */

import * as THREE from 'three';
import { sphere, ellipsoid, box, cone, cylinder, warmMat } from './primitives';

// ─── Colors ─────────────────────────────────────────────────────

const BEE_YELLOW   = 0xE8C030;
const BEE_BLACK    = 0x2A2018;
const BEE_WING     = 0xD8E8F0;

const BUTTERFLY_BODY   = 0x3A2820;
const BUTTERFLY_WING1  = 0xE89050; // warm orange
const BUTTERFLY_WING2  = 0xD06888; // rose pink
const BUTTERFLY_WING3  = 0x70A8D0; // sky blue
const BUTTERFLY_WING4  = 0xC8B040; // golden

const BIRD_BODY    = 0x6B5040;
const BIRD_BREAST  = 0xC8A070;
const BIRD_BEAK    = 0xD8A030;
const BIRD_WING    = 0x584838;

const WORM_BODY    = 0xC08870;
const WORM_BELLY   = 0xD8A890;

const BEETLE_SHELL = 0x3A3020;
const BEETLE_SHEEN = 0x506030;
const BEETLE_HEAD  = 0x2A2018;

const SQUIRREL_BODY = 0x8B5E3C;  // warm chestnut
const SQUIRREL_BELLY = 0xD4B896; // cream
const SQUIRREL_TAIL = 0x6B4226;  // darker brown
const SQUIRREL_EYE = 0x181010;

// ─── Wing material (semi-transparent) ───────────────────────────

function wingMat(color: number): THREE.MeshLambertMaterial {
  const mat = new THREE.MeshLambertMaterial({
    color,
    emissive: color,
    emissiveIntensity: 0.15,
    transparent: true,
    opacity: 0.6,
    side: THREE.DoubleSide,
  });
  return mat;
}

// Shared plane geometry for wings
const _wingGeo = new THREE.PlaneGeometry(1, 1);

function wing(
  parent: THREE.Group,
  x: number, y: number, z: number,
  w: number, h: number,
  color: number,
  rotY: number = 0,
): THREE.Mesh {
  const m = new THREE.Mesh(_wingGeo, wingMat(color));
  m.position.set(x, y, z);
  m.scale.set(w, h, 1);
  m.rotation.y = rotY;
  parent.add(m);
  return m;
}

// ─── Model Builders ─────────────────────────────────────────────

/**
 * Build a bee model. ~1.5 voxels across.
 * Rounded yellow/black striped body, tiny wings, no legs.
 */
export function buildBee(seed: number = 0): THREE.Group {
  const g = new THREE.Group();
  g.name = 'bee';

  // Body: elongated ellipsoid
  ellipsoid(g, 0, 0, 0, 0.35, 0.25, 0.55, BEE_YELLOW);

  // Black stripes (thin boxes)
  box(g, 0, 0.02, -0.1, 0.38, 0.28, 0.08, BEE_BLACK);
  box(g, 0, 0.02,  0.15, 0.38, 0.28, 0.08, BEE_BLACK);

  // Head
  sphere(g, 0, 0.05, -0.45, 0.2, BEE_BLACK);

  // Eyes (tiny bright dots)
  sphere(g, -0.12, 0.1, -0.52, 0.06, 0xF0F0F0);
  sphere(g, 0.12, 0.1, -0.52, 0.06, 0xF0F0F0);

  // Wings (semi-transparent planes, angled up)
  const lw = wing(g, -0.25, 0.25, 0, 0.5, 0.3, BEE_WING);
  lw.rotation.set(0, 0, 0.5);
  lw.name = 'wing_l';
  const rw = wing(g, 0.25, 0.25, 0, 0.5, 0.3, BEE_WING);
  rw.rotation.set(0, 0, -0.5);
  rw.name = 'wing_r';

  // Stinger
  cone(g, 0, 0, 0.5, 0.08, 0.2, BEE_BLACK);

  return g;
}

/**
 * Build a butterfly model. ~2 voxels across.
 * Thin body stick with 4 colorful wing planes.
 */
export function buildButterfly(seed: number = 0): THREE.Group {
  const g = new THREE.Group();
  g.name = 'butterfly';

  // Thin body
  cylinder(g, 0, 0, 0, 0.06, 0.8, BUTTERFLY_BODY);
  // Head
  sphere(g, 0, 0, -0.35, 0.1, BUTTERFLY_BODY);

  // Antennae
  cylinder(g, -0.08, 0.12, -0.38, 0.015, 0.25, BUTTERFLY_BODY);
  cylinder(g, 0.08, 0.12, -0.38, 0.015, 0.25, BUTTERFLY_BODY);

  // Wing color based on seed
  const wingColors = [BUTTERFLY_WING1, BUTTERFLY_WING2, BUTTERFLY_WING3, BUTTERFLY_WING4];
  const wc = wingColors[seed & 3];

  // 4 wings (upper pair larger)
  const ulw = wing(g, -0.5, 0.05, -0.05, 0.8, 0.6, wc);
  ulw.name = 'wing_ul';
  const urw = wing(g, 0.5, 0.05, -0.05, 0.8, 0.6, wc);
  urw.name = 'wing_ur';

  // Lower pair (slightly smaller, slightly different color)
  const llw = wing(g, -0.35, -0.02, 0.2, 0.55, 0.45, wc);
  llw.name = 'wing_ll';
  const lrw = wing(g, 0.35, -0.02, 0.2, 0.55, 0.45, wc);
  lrw.name = 'wing_lr';

  return g;
}

/**
 * Build a bird model. ~3 voxels across.
 * Rounded body, pointed beak, wing shapes.
 */
export function buildBird(seed: number = 0): THREE.Group {
  const g = new THREE.Group();
  g.name = 'bird';

  // Body (teardrop = sphere + smaller sphere)
  ellipsoid(g, 0, 0, 0, 0.5, 0.45, 0.7, BIRD_BODY);

  // Breast (lighter underside)
  ellipsoid(g, 0, -0.1, -0.1, 0.35, 0.3, 0.5, BIRD_BREAST);

  // Head
  sphere(g, 0, 0.2, -0.55, 0.3, BIRD_BODY);

  // Eyes
  sphere(g, -0.18, 0.28, -0.7, 0.06, 0xF0F0F0);
  sphere(g, 0.18, 0.28, -0.7, 0.06, 0xF0F0F0);
  sphere(g, -0.18, 0.28, -0.72, 0.04, 0x181010);
  sphere(g, 0.18, 0.28, -0.72, 0.04, 0x181010);

  // Beak
  cone(g, 0, 0.15, -0.85, 0.08, 0.2, BIRD_BEAK);
  // Rotate beak to point forward
  g.children[g.children.length - 1].rotation.x = Math.PI / 2;

  // Wings (flat ellipsoids, angled)
  const lw = ellipsoid(g, -0.55, 0.1, 0.05, 0.45, 0.1, 0.55, BIRD_WING);
  lw.rotation.z = 0.3;
  lw.name = 'wing_l';
  const rw = ellipsoid(g, 0.55, 0.1, 0.05, 0.45, 0.1, 0.55, BIRD_WING);
  rw.rotation.z = -0.3;
  rw.name = 'wing_r';

  // Tail feathers
  ellipsoid(g, 0, 0.05, 0.6, 0.2, 0.08, 0.35, BIRD_WING);

  return g;
}

/**
 * Build a worm model. ~1 voxel across.
 * Segmented capsule chain with subtle color gradient.
 */
export function buildWorm(seed: number = 0): THREE.Group {
  const g = new THREE.Group();
  g.name = 'worm';

  // 4 body segments in a gentle curve
  const segments = 4;
  for (let i = 0; i < segments; i++) {
    const t = i / (segments - 1);
    const x = Math.sin(t * 1.0) * 0.2;
    const z = (t - 0.5) * 1.0;
    const r = 0.15 - t * 0.03; // tapers toward tail
    const color = i === 0 ? WORM_BELLY : WORM_BODY;
    sphere(g, x, 0, z, r, color);
  }

  // Tiny eye dots on head
  sphere(g, -0.08, 0.08, -0.5, 0.04, 0x181010);
  sphere(g, 0.08, 0.08, -0.5, 0.04, 0x181010);

  return g;
}

/**
 * Build a beetle model. ~1.2 voxels across.
 * Dome shell, small head, stubby legs.
 */
export function buildBeetle(seed: number = 0): THREE.Group {
  const g = new THREE.Group();
  g.name = 'beetle';

  // Shell (flattened sphere, shiny)
  ellipsoid(g, 0, 0.08, 0, 0.35, 0.25, 0.45, BEETLE_SHELL);

  // Shell sheen highlight (slightly lighter top)
  ellipsoid(g, 0, 0.15, -0.02, 0.28, 0.15, 0.35, BEETLE_SHEEN);

  // Head
  sphere(g, 0, 0.05, -0.4, 0.18, BEETLE_HEAD);

  // Eyes
  sphere(g, -0.1, 0.1, -0.48, 0.05, 0xE0E0D0);
  sphere(g, 0.1, 0.1, -0.48, 0.05, 0xE0E0D0);

  // Antennae
  cylinder(g, -0.06, 0.16, -0.5, 0.012, 0.15, BEETLE_HEAD);
  cylinder(g, 0.06, 0.16, -0.5, 0.012, 0.15, BEETLE_HEAD);

  // 6 stubby legs (3 per side)
  for (let i = 0; i < 3; i++) {
    const z = -0.15 + i * 0.15;
    // Left leg
    box(g, -0.3, -0.05, z, 0.15, 0.04, 0.04, BEETLE_HEAD);
    // Right leg
    box(g, 0.3, -0.05, z, 0.15, 0.04, 0.04, BEETLE_HEAD);
  }

  return g;
}

/**
 * Build a squirrel model. ~2.5 voxels across.
 * Rounded body, bushy tail curling upward, tiny paws.
 */
export function buildSquirrel(seed: number = 0): THREE.Group {
  const g = new THREE.Group();
  g.name = 'squirrel';

  // Body (oval, slightly upright)
  ellipsoid(g, 0, 0, 0, 0.3, 0.35, 0.5, SQUIRREL_BODY);

  // Lighter belly
  ellipsoid(g, 0, -0.08, -0.05, 0.2, 0.22, 0.35, SQUIRREL_BELLY);

  // Head (round, slightly forward and up)
  sphere(g, 0, 0.2, -0.45, 0.22, SQUIRREL_BODY);

  // Eyes (bright, alert)
  sphere(g, -0.12, 0.26, -0.58, 0.05, 0xF0F0F0);
  sphere(g, 0.12, 0.26, -0.58, 0.05, 0xF0F0F0);
  sphere(g, -0.12, 0.26, -0.6, 0.03, SQUIRREL_EYE);
  sphere(g, 0.12, 0.26, -0.6, 0.03, SQUIRREL_EYE);

  // Small round ears
  sphere(g, -0.14, 0.38, -0.38, 0.07, SQUIRREL_BODY);
  sphere(g, 0.14, 0.38, -0.38, 0.07, SQUIRREL_BODY);

  // Nose
  sphere(g, 0, 0.18, -0.65, 0.04, SQUIRREL_EYE);

  // Front paws (tiny)
  sphere(g, -0.15, -0.15, -0.35, 0.08, SQUIRREL_BODY);
  sphere(g, 0.15, -0.15, -0.35, 0.08, SQUIRREL_BODY);

  // Bushy tail — series of overlapping spheres curling upward
  sphere(g, 0, 0.05, 0.45, 0.18, SQUIRREL_TAIL);
  sphere(g, 0, 0.2, 0.55, 0.2, SQUIRREL_TAIL);
  sphere(g, 0, 0.4, 0.5, 0.22, SQUIRREL_TAIL);
  sphere(g, 0, 0.55, 0.4, 0.2, SQUIRREL_TAIL);
  sphere(g, 0, 0.6, 0.25, 0.15, SQUIRREL_TAIL);

  return g;
}

/** Build a fauna model by type index (matches FaunaType enum). */
export function buildFaunaModel(type: number, seed: number = 0): THREE.Group {
  switch (type) {
    case 0: return buildBee(seed);
    case 1: return buildButterfly(seed);
    case 2: return buildBird(seed);
    case 3: return buildWorm(seed);
    case 4: return buildBeetle(seed);
    case 5: return buildSquirrel(seed);
    default: return buildBee(seed);
  }
}
