/**
 * Foliage billboard renderer with wind sway animation.
 *
 * Replaces Leaf voxel cubes with billboard sprites that always face the
 * camera and gently sway in simulated wind. Creates a lush, organic look
 * instead of blocky cubes.
 *
 * Uses InstancedMesh for efficient rendering of many leaf sprites.
 * Custom ShaderMaterial provides:
 * - Camera-facing billboard rotation
 * - Sine-wave wind sway (offset by world position for natural variation)
 * - Soft circular alpha cutout
 * - Per-instance color variation for visual richness
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GRID_Z, VOXEL_BYTES, Material, GROUND_LEVEL, materialIsFoliage as isFoliage } from '../bridge';

/** Maximum number of foliage instances (pre-allocated) */
const MAX_FOLIAGE = 50_000;

/** Foliage sprite size (larger than a voxel for lush canopy overlap) */
const SPRITE_SIZE = 1.4;

/** Wind sway vertex shader */
const FOLIAGE_VERT = /* glsl */ `
  attribute vec3 instanceColor;
  attribute float instanceSpecies;

  uniform float uTime;
  uniform float uWindStrength;
  uniform vec2 uWindDir;
  uniform vec3 uDayTint;
  uniform float uDayAmount;

  varying vec3 vColor;
  varying vec2 vUv;
  varying float vSpecies;

  void main() {
    vColor = instanceColor * uDayTint;
    vUv = uv;
    vSpecies = instanceSpecies;

    int sp = int(instanceSpecies + 0.5);

    // Billboard: extract instance position from instance matrix
    vec4 worldPos = instanceMatrix * vec4(0.0, 0.0, 0.0, 1.0);

    // Wind sway: sine wave offset by world position for natural variation
    // Height above ground increases sway amplitude (Three.js Y = up)
    float heightFactor = max(0.0, (worldPos.y - ${GROUND_LEVEL.toFixed(1)}) * 0.04);

    // Species-specific wind personality:
    //   Oak(0):    slow heavy sway, deep amplitude
    //   Birch(1):  quick fluttery, high frequency
    //   Willow(2): slow deep swoops, dramatic amplitude
    //   Pine(3):   stiff, minimal sway
    //   Fern(4):   medium, bouncy
    //   Flowers/groundcover: light and responsive
    float timeScale = 1.0;
    float ampScale = 1.0;
    if (sp == 0) { timeScale = 0.7; ampScale = 1.3; }       // Oak: slow, heavy
    else if (sp == 1) { timeScale = 1.5; ampScale = 0.9; }   // Birch: fluttery
    else if (sp == 2) { timeScale = 0.5; ampScale = 1.6; }   // Willow: deep swoops
    else if (sp == 3) { timeScale = 0.9; ampScale = 0.4; }   // Pine: stiff
    else if (sp == 4) { timeScale = 1.1; ampScale = 1.1; }   // Fern: bouncy

    float t = uTime * timeScale;
    float swayX = sin(t * 1.2 + worldPos.x * 0.7 + worldPos.z * 0.3) * uWindStrength * heightFactor * ampScale;
    float swayY = cos(t * 0.9 + worldPos.z * 0.5 + worldPos.x * 0.4) * uWindStrength * heightFactor * 0.6 * ampScale;
    float swayZ = sin(t * 0.7 + worldPos.x * 0.3 + worldPos.z * 0.6) * uWindStrength * heightFactor * 0.2 * ampScale;

    // Directional lean: foliage leans in wind direction during gusts
    // Stronger wind = more directional lean, weaker = random sway dominates
    float leanAmount = uWindStrength * uWindStrength * heightFactor * 0.6 * ampScale;
    swayX += uWindDir.x * leanAmount;
    swayZ += uWindDir.y * leanAmount;

    // Billboard: orient quad to face camera
    // Keep local vertex position (quad corners), but orient in camera space
    vec3 cameraRight = vec3(viewMatrix[0][0], viewMatrix[1][0], viewMatrix[2][0]);
    vec3 cameraUp = vec3(viewMatrix[0][1], viewMatrix[1][1], viewMatrix[2][1]);

    // Scale from instance matrix (uniform scale assumed)
    float scale = length(instanceMatrix[0].xyz);

    // Flower bloom cycle: flowers open during day, close at night
    // Species 7=Wildflower, 8=Daisy
    float bloomScale = 1.0;
    if (sp == 7 || sp == 8) {
      // uDayAmount: 1.0 = full day, 0.0 = full night
      // Flowers shrink to 55% at night (closed), full size during day
      bloomScale = 0.55 + 0.45 * uDayAmount;
    }

    vec3 vertexWorld = worldPos.xyz
      + cameraRight * position.x * scale * bloomScale
      + cameraUp * position.y * scale * bloomScale
      + vec3(swayX, swayY, swayZ);

    gl_Position = projectionMatrix * viewMatrix * vec4(vertexWorld, 1.0);
  }
`;

/** Foliage fragment shader with species-specific procedural textures.
 *  Each plant type gets a distinct silhouette so species are visually
 *  identifiable beyond just color:
 *  - Oak/Birch/Willow: multi-lobe leaf clusters
 *  - Pine: spiky needle star
 *  - Fern: elongated frond
 *  - Shrubs: bushy irregular edge
 *  - Flowers: petal star
 *  - Groundcover: flat organic disc with textured edge
 */
const FOLIAGE_FRAG = /* glsl */ `
  varying vec3 vColor;
  varying vec2 vUv;
  varying float vSpecies;

  // Hash for procedural noise
  float hash21(vec2 p) {
    p = fract(p * vec2(123.34, 456.21));
    p += dot(p, p + 45.32);
    return fract(p.x * p.y);
  }

  void main() {
    vec2 uv = (vUv - 0.5) * 2.0; // -1 to 1
    float dist = length(uv);
    float angle = atan(uv.y, uv.x);
    int species = int(vSpecies + 0.5);

    float alpha = 0.0;
    float shade = 1.0;

    // === TREES (0-3): distinct leaf shapes ===
    if (species == 0) {
      // Oak: multi-lobe rounded clusters (5 bumpy lobes)
      float lobes = 0.55 + 0.18 * cos(angle * 5.0) + 0.08 * cos(angle * 3.0 + 1.5);
      alpha = 1.0 - smoothstep(lobes - 0.15, lobes, dist);
      // Internal vein-like texture
      float vein = abs(sin(angle * 5.0 + dist * 4.0)) * 0.12;
      shade = 0.85 + vein;
    } else if (species == 1) {
      // Birch: small trembling leaves — many small overlapping circles
      float r1 = length(uv - vec2( 0.15,  0.2));
      float r2 = length(uv - vec2(-0.2,   0.15));
      float r3 = length(uv - vec2( 0.05, -0.2));
      float r4 = length(uv - vec2(-0.15, -0.1));
      float r5 = length(uv - vec2( 0.25, -0.05));
      float minR = min(min(min(r1, r2), min(r3, r4)), r5);
      alpha = 1.0 - smoothstep(0.22, 0.32, minR);
      shade = 0.9 - minR * 0.15;
    } else if (species == 2) {
      // Willow: droopy teardrop, elongated downward
      vec2 wUv = uv;
      wUv.y -= 0.15; // shift up to droop down
      wUv.y *= 0.6;  // stretch vertically
      float wDist = length(wUv);
      // Wispy trailing edges
      float wisp = 0.5 + 0.12 * sin(angle * 7.0 + wDist * 3.0);
      alpha = 1.0 - smoothstep(wisp - 0.12, wisp, wDist);
      shade = 0.82 + abs(sin(angle * 4.0)) * 0.1;
    } else if (species == 3) {
      // Pine: sharp needle star (6 pointed)
      float star = cos(angle * 3.0); // 6-pointed via cos(3θ) symmetry
      float r = 0.35 + 0.3 * abs(star);
      alpha = 1.0 - smoothstep(r - 0.08, r, dist);
      // Needle texture: thin radial lines
      float needles = abs(sin(angle * 12.0)) * 0.15;
      shade = 0.75 + needles;

    // === SHRUBS (4-6): bushy, textured shapes ===
    } else if (species == 4) {
      // Fern: elongated frond shape (wider than tall, with serrated edge)
      vec2 fUv = uv;
      fUv.x *= 0.7; // stretch horizontally
      float fDist = length(fUv);
      float serration = 0.55 + 0.1 * cos(angle * 8.0) + 0.06 * cos(angle * 13.0);
      alpha = 1.0 - smoothstep(serration - 0.1, serration, fDist);
      // Frond mid-rib
      float rib = 1.0 - smoothstep(0.0, 0.06, abs(uv.y));
      shade = 0.8 + rib * 0.12;
    } else if (species == 5) {
      // Berry Bush: round but lumpy with berry-like dots
      float lumpy = 0.5 + 0.1 * cos(angle * 6.0 + 0.5) + 0.06 * sin(angle * 9.0);
      alpha = 1.0 - smoothstep(lumpy - 0.12, lumpy, dist);
      // Berry highlights (small bright dots)
      float berryDot = smoothstep(0.08, 0.04, length(uv - vec2(0.2, 0.15)))
                     + smoothstep(0.08, 0.04, length(uv - vec2(-0.15, 0.2)))
                     + smoothstep(0.08, 0.04, length(uv - vec2(0.1, -0.18)));
      shade = 0.82 + berryDot * 0.2;
    } else if (species == 6) {
      // Holly: pointed spiky leaf shape (like a holly leaf)
      float spikes = 0.48 + 0.15 * cos(angle * 4.0) + 0.08 * cos(angle * 8.0 + 2.0);
      alpha = 1.0 - smoothstep(spikes - 0.08, spikes, dist);
      shade = 0.78 + abs(cos(angle * 4.0)) * 0.12;

    // === FLOWERS (7-8): petal shapes ===
    } else if (species == 7) {
      // Wildflower: 5-petal flower shape
      float petals = 0.3 + 0.28 * pow(abs(cos(angle * 2.5)), 0.8);
      alpha = 1.0 - smoothstep(petals - 0.08, petals, dist);
      // Bright center
      float center = 1.0 - smoothstep(0.1, 0.18, dist);
      shade = 0.85 + center * 0.2;
    } else if (species == 8) {
      // Daisy: many thin petals radiating from center
      float petals = 0.25 + 0.32 * pow(abs(cos(angle * 6.0)), 1.5);
      alpha = 1.0 - smoothstep(petals - 0.06, petals, dist);
      // Yellow center disc
      float center = 1.0 - smoothstep(0.12, 0.2, dist);
      shade = 0.88 + center * 0.15;

    // === GROUNDCOVER (9-11): flat, organic shapes ===
    } else if (species == 9) {
      // Moss: irregular, soft-edged patch
      float wobble = 0.45 + 0.12 * sin(angle * 7.0 + 1.0) + 0.08 * cos(angle * 11.0);
      alpha = 1.0 - smoothstep(wobble - 0.2, wobble, dist);
      shade = 0.75 + dist * 0.1;
    } else if (species == 10) {
      // Grass: tall narrow vertical blades
      float blade1 = smoothstep(0.12, 0.04, abs(uv.x - 0.0)) * (1.0 - smoothstep(0.4, 0.65, abs(uv.y)));
      float blade2 = smoothstep(0.1, 0.03, abs(uv.x - 0.2)) * (1.0 - smoothstep(0.3, 0.55, abs(uv.y - 0.05)));
      float blade3 = smoothstep(0.1, 0.03, abs(uv.x + 0.2)) * (1.0 - smoothstep(0.35, 0.6, abs(uv.y + 0.05)));
      alpha = max(max(blade1, blade2), blade3);
      shade = 0.8 + uv.y * 0.1; // lighter at tips
    } else if (species == 11) {
      // Clover: 3-lobe shamrock
      vec2 l1 = uv - vec2(0.0, 0.22);
      vec2 l2 = uv - vec2(-0.19, -0.11);
      vec2 l3 = uv - vec2(0.19, -0.11);
      float c1 = 1.0 - smoothstep(0.16, 0.24, length(l1));
      float c2 = 1.0 - smoothstep(0.16, 0.24, length(l2));
      float c3 = 1.0 - smoothstep(0.16, 0.24, length(l3));
      // Thin stem
      float stem = smoothstep(0.04, 0.01, abs(uv.x)) * step(-0.45, uv.y) * step(uv.y, -0.1);
      alpha = max(max(max(c1, c2), c3), stem);
      shade = 0.85;
    } else {
      // Fallback: simple soft circle
      alpha = 1.0 - smoothstep(0.5, 0.8, dist);
      shade = 0.9;
    }

    if (alpha < 0.05) discard;

    // Slight edge darkening on top of species shading
    shade *= (1.0 - dist * 0.08);

    gl_FragColor = vec4(vColor * shade, alpha * 0.92);
  }
`;

/** Base leaf colors with warm green variation */
const LEAF_COLORS = [
  new THREE.Color(0.30, 0.55, 0.20), // warm green (base)
  new THREE.Color(0.35, 0.58, 0.22), // slightly lighter
  new THREE.Color(0.25, 0.50, 0.18), // slightly darker
  new THREE.Color(0.32, 0.52, 0.25), // yellow-green tint
  new THREE.Color(0.28, 0.48, 0.22), // deeper green
];

/** Species-specific foliage color palettes.
 *  Index matches species_id stored in voxel nutrient_level byte.
 *  0=Oak, 1=Birch, 2=Willow, 3=Pine, 4=Fern, 5=Berry Bush, 6=Holly,
 *  7=Wildflower, 8=Daisy, 9=Moss, 10=Grass, 11=Clover
 *
 *  Colors are pushed wide apart so each species reads as visually
 *  distinct even under warm golden-hour lighting. Hue varies from
 *  warm olive (oak) through cool sage (willow) to vibrant emerald (fern). */
const SPECIES_FOLIAGE: THREE.Color[] = [
  new THREE.Color(0.22, 0.48, 0.12),  // Oak: deep warm forest green
  new THREE.Color(0.48, 0.72, 0.22),  // Birch: bright spring yellow-green
  new THREE.Color(0.25, 0.52, 0.38),  // Willow: silver-sage, distinctly blue
  new THREE.Color(0.10, 0.30, 0.18),  // Pine: very dark blue-green
  new THREE.Color(0.16, 0.62, 0.35),  // Fern: vibrant emerald
  new THREE.Color(0.38, 0.52, 0.16),  // Berry Bush: warm olive
  new THREE.Color(0.12, 0.38, 0.12),  // Holly: dark pure green
  new THREE.Color(0.75, 0.35, 0.55),  // Wildflower: vivid magenta-pink
  new THREE.Color(0.85, 0.78, 0.22),  // Daisy: bright golden yellow
  new THREE.Color(0.18, 0.40, 0.16),  // Moss: dark muted olive
  new THREE.Color(0.30, 0.65, 0.14),  // Grass: bright fresh green
  new THREE.Color(0.38, 0.60, 0.18),  // Clover: yellow-green
];

export class FoliageRenderer {
  readonly group: THREE.Group;

  private mesh: THREE.InstancedMesh;
  private material: THREE.ShaderMaterial;
  private colorAttr: THREE.InstancedBufferAttribute;
  private speciesAttr: THREE.InstancedBufferAttribute;
  private instanceCount = 0;
  private dummy = new THREE.Object3D();

  constructor() {
    this.group = new THREE.Group();
    this.group.name = 'foliage';

    // Quad geometry for billboard sprite
    const geo = new THREE.PlaneGeometry(SPRITE_SIZE, SPRITE_SIZE);

    // Custom shader material with wind sway + billboard
    this.material = new THREE.ShaderMaterial({
      vertexShader: FOLIAGE_VERT,
      fragmentShader: FOLIAGE_FRAG,
      uniforms: {
        uTime: { value: 0 },
        uWindStrength: { value: 0.35 },
        uWindDir: { value: new THREE.Vector2(1, 0) },
        uDayTint: { value: new THREE.Color(1, 1, 1) },
        uDayAmount: { value: 1.0 },
      },
      transparent: true,
      depthWrite: false,
      side: THREE.DoubleSide,
    });

    // Per-instance colors
    const colorArray = new Float32Array(MAX_FOLIAGE * 3);
    this.colorAttr = new THREE.InstancedBufferAttribute(colorArray, 3);

    // Per-instance species ID (for texture selection in fragment shader)
    const speciesArray = new Float32Array(MAX_FOLIAGE);
    this.speciesAttr = new THREE.InstancedBufferAttribute(speciesArray, 1);

    this.mesh = new THREE.InstancedMesh(geo, this.material, MAX_FOLIAGE);
    this.mesh.instanceColor = null; // we use custom attribute instead
    this.mesh.geometry.setAttribute('instanceColor', this.colorAttr);
    this.mesh.geometry.setAttribute('instanceSpecies', this.speciesAttr);
    this.mesh.frustumCulled = false; // billboards change size, let them all render
    this.mesh.count = 0;

    this.group.add(this.mesh);
  }

  /**
   * Scan the voxel grid and build foliage instances for all Leaf voxels.
   * Call after grid changes (tick, tool placement).
   */
  rebuild(grid: Uint8Array): void {
    let count = 0;

    for (let z = 0; z < GRID_Z && count < MAX_FOLIAGE; z++) {
      for (let y = 0; y < GRID_Y && count < MAX_FOLIAGE; y++) {
        for (let x = 0; x < GRID_X && count < MAX_FOLIAGE; x++) {
          const idx = (x + y * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
          const mat = grid[idx];

          if (!isFoliage(mat)) continue;

          // Position at voxel center (sim Y↔Z swap: sim Z=up → Three.js Y=up)
          this.dummy.position.set(x + 0.5, z + 0.5, y + 0.5);

          // Count neighboring leaf voxels for density-based sizing.
          // Interior canopy leaves (many neighbors) get larger → cohesive mass.
          // Edge leaves (few neighbors) stay smaller → natural fringe.
          let neighbors = 0;
          for (let dz = -1; dz <= 1; dz++) {
            for (let dy = -1; dy <= 1; dy++) {
              for (let dx = -1; dx <= 1; dx++) {
                if (dx === 0 && dy === 0 && dz === 0) continue;
                const nx = x + dx, ny = y + dy, nz = z + dz;
                if (nx >= 0 && nx < GRID_X && ny >= 0 && ny < GRID_Y && nz >= 0 && nz < GRID_Z) {
                  const ni = (nx + ny * GRID_X + nz * GRID_X * GRID_Y) * VOXEL_BYTES;
                  if (isFoliage(grid[ni])) neighbors++;
                }
              }
            }
          }
          // Scale: edge leaves 0.8, dense interior 1.4 — plus hash noise
          const hash = (x * 73856093 ^ y * 19349663 ^ z * 83492791) & 0xffff;
          const densityScale = 0.8 + (neighbors / 26.0) * 0.6;
          const noiseScale = ((hash & 0xff) / 255.0 - 0.5) * 0.15;
          const scaleVar = densityScale + noiseScale;
          this.dummy.scale.setScalar(scaleVar);

          // Random rotation around Z for visual variety (won't matter much with billboard)
          this.dummy.rotation.set(0, 0, (hash >> 8) / 255.0 * Math.PI);
          this.dummy.updateMatrix();
          this.mesh.setMatrixAt(count, this.dummy.matrix);

          // Pick color based on species (nutrient_level byte = species_id)
          const speciesId = grid[idx + 3];
          this.speciesAttr.setX(count, speciesId);
          const baseColor = speciesId < SPECIES_FOLIAGE.length
            ? SPECIES_FOLIAGE[speciesId]
            : LEAF_COLORS[hash % LEAF_COLORS.length];

          // Health-based stress tint: water_level byte MAY store health.
          // However, leaf voxels in the canopy often have water_level=0 (no
          // water above ground), which is NOT "dead" — it means "no data".
          // Treat 0 as fully healthy. Only values 1–59 indicate actual stress.
          const health = grid[idx + 1];
          const healthFrac = health === 0 ? 1.0 : Math.min(health / 60, 1);
          const stressTint = Math.pow(Math.max(0, 1 - healthFrac), 2);

          // Per-instance brightness variation for visual richness
          const brightness = 0.9 + ((hash >> 4) & 0xf) / 15.0 * 0.2;
          const r = baseColor.r * brightness + stressTint * 0.2;
          const g = baseColor.g * brightness * (1 - stressTint * 0.3);
          const b = baseColor.b * brightness * (1 - stressTint * 0.5);
          this.colorAttr.setXYZ(count, r, g, b);

          count++;
        }
      }
    }

    this.instanceCount = count;
    this.mesh.count = count;
    this.mesh.instanceMatrix.needsUpdate = true;
    this.colorAttr.needsUpdate = true;
    this.speciesAttr.needsUpdate = true;
  }

  /**
   * Update wind animation. Call each frame with elapsed time.
   */
  update(elapsedTime: number): void {
    this.material.uniforms.uTime.value = elapsedTime;
  }

  /** Get current wind strength */
  getWindStrength(): number {
    return this.material.uniforms.uWindStrength.value as number;
  }

  /** Set wind strength (0 = still, 1 = gusty) */
  setWindStrength(strength: number): void {
    this.material.uniforms.uWindStrength.value = strength;
  }

  /** Set wind direction (angle in radians, slowly drifting) */
  setWindDirection(angle: number): void {
    const dir = this.material.uniforms.uWindDir.value as THREE.Vector2;
    dir.set(Math.cos(angle), Math.sin(angle));
  }

  /** Set foliage tint based on time of day (0-1 cycle).
   *  Dawn=warm gold, noon=neutral, golden hour=amber, night=cool blue. */
  setDayTint(dayTime: number): void {
    const tint = this.material.uniforms.uDayTint.value as THREE.Color;
    // Day cycle: 0.25=dawn, 0.5=noon, 0.75=golden hour, 1.0=night
    if (dayTime < 0.2) {
      // Night → dawn transition: cool blue → warm
      const t = dayTime / 0.2;
      tint.setRGB(0.6 + t * 0.4, 0.65 + t * 0.3, 0.85 - t * 0.15);
    } else if (dayTime < 0.35) {
      // Dawn: warm golden
      const t = (dayTime - 0.2) / 0.15;
      tint.setRGB(1.0 + t * 0.05, 0.95 - t * 0.05, 0.7 + t * 0.3);
    } else if (dayTime < 0.65) {
      // Midday: neutral warm white
      tint.setRGB(1.05, 0.98, 1.0);
    } else if (dayTime < 0.8) {
      // Golden hour: warm amber
      const t = (dayTime - 0.65) / 0.15;
      tint.setRGB(1.0 + t * 0.1, 0.95 - t * 0.1, 0.95 - t * 0.25);
    } else {
      // Night: cool blue-grey
      const t = (dayTime - 0.8) / 0.2;
      tint.setRGB(1.1 - t * 0.5, 0.85 - t * 0.2, 0.7 + t * 0.15);
    }
  }

  /** Set drought stress level (0=healthy, 1=fully stressed).
   *  Lerps foliage tint toward dry yellow-brown. */
  setDroughtStress(stress: number): void {
    if (stress <= 0) return;
    const tint = this.material.uniforms.uDayTint.value as THREE.Color;
    // Push toward dry yellow-brown proportional to stress
    const dr = 1.05 + stress * 0.1;
    const dg = 0.95 - stress * 0.25;
    const db = 0.85 - stress * 0.35;
    tint.setRGB(
      tint.r * (1 - stress * 0.3) + dr * stress * 0.3,
      tint.g * (1 - stress * 0.3) + dg * stress * 0.3,
      tint.b * (1 - stress * 0.3) + db * stress * 0.3,
    );
  }

  /** Set day amount for flower bloom cycle (0=night/closed, 1=day/open) */
  setDayAmount(amount: number): void {
    this.material.uniforms.uDayAmount.value = amount;
  }

  /** Current number of active foliage sprites */
  get count(): number {
    return this.instanceCount;
  }

  dispose(): void {
    this.mesh.geometry.dispose();
    this.material.dispose();
  }
}
