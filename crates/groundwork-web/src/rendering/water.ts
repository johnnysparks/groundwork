/**
 * Water surface renderer: flat mesh with custom shader.
 *
 * Instead of rendering water as voxel cubes, we scan the grid for water cells
 * that have air above them (the "surface"), compute depth per column, and build
 * a flat mesh. A custom ShaderMaterial provides:
 *
 * - Scrolling procedural normals for subtle ripple animation
 * - Depth-based opacity (shallow = clear glass, deep = opaque teal)
 * - Soft edge blending where water meets terrain
 * - Subtle sky reflection tint on the surface
 */

import * as THREE from 'three';
import {
  GRID_X, GRID_Y, GRID_Z, VOXEL_BYTES,
  Material,
} from '../bridge';

// ---------------------------------------------------------------------------
// Water surface detection
// ---------------------------------------------------------------------------

interface WaterColumn {
  x: number;
  y: number;
  surfaceZ: number; // top z of water column
  depth: number;    // how many water cells deep
}

/**
 * Scan the grid for water surface cells: water voxels with air (or out-of-bounds) above.
 * Also computes depth — how many consecutive water cells below the surface.
 */
function findWaterSurfaces(grid: Uint8Array): WaterColumn[] {
  const columns: WaterColumn[] = [];

  for (let y = 0; y < GRID_Y; y++) {
    for (let x = 0; x < GRID_X; x++) {
      // Scan top-down to find the highest water cell with air above
      for (let z = GRID_Z - 1; z >= 0; z--) {
        const idx = (x + y * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
        const mat = grid[idx];

        if (mat !== Material.Water) continue;

        // Check if above is air (or out of bounds = sky)
        const aboveZ = z + 1;
        let aboveMat: number = Material.Air;
        if (aboveZ < GRID_Z) {
          const aboveIdx = (x + y * GRID_X + aboveZ * GRID_X * GRID_Y) * VOXEL_BYTES;
          aboveMat = grid[aboveIdx];
        }

        if (aboveMat !== Material.Air) continue;

        // Found a surface cell — compute depth
        let depth = 1;
        for (let dz = z - 1; dz >= 0; dz--) {
          const belowIdx = (x + y * GRID_X + dz * GRID_X * GRID_Y) * VOXEL_BYTES;
          if (grid[belowIdx] === Material.Water) {
            depth++;
          } else {
            break;
          }
        }

        columns.push({ x, y, surfaceZ: z, depth });
        break; // only top surface per column
      }
    }
  }

  return columns;
}

// ---------------------------------------------------------------------------
// Greedy merge for water surface quads
// ---------------------------------------------------------------------------

interface WaterQuad {
  x: number;
  y: number;
  z: number;
  w: number;  // width in x
  h: number;  // height in y
  depth: number;
}

/**
 * Greedy merge water surface cells into larger quads.
 * Groups cells at the same z with similar depth.
 */
function mergeWaterSurfaces(columns: WaterColumn[]): WaterQuad[] {
  if (columns.length === 0) return [];

  // Group by surfaceZ
  const byZ = new Map<number, WaterColumn[]>();
  for (const col of columns) {
    const list = byZ.get(col.surfaceZ) ?? [];
    list.push(col);
    byZ.set(col.surfaceZ, list);
  }

  const quads: WaterQuad[] = [];

  for (const [z, cols] of byZ) {
    // Build a 2D mask: depth at each (x,y), 0 if no water
    const mask = new Uint8Array(GRID_X * GRID_Y);
    for (const col of cols) {
      // Quantize depth to 4 levels for better merging
      const depthBucket = Math.min(col.depth, 255);
      mask[col.x + col.y * GRID_X] = depthBucket;
    }

    // Greedy merge in x then y
    for (let y = 0; y < GRID_Y; y++) {
      let x = 0;
      while (x < GRID_X) {
        const idx = x + y * GRID_X;
        const d = mask[idx];
        if (d === 0) { x++; continue; }

        // Quantize to shallow/medium/deep for merging (but keep original for rendering)
        const dBucket = d <= 1 ? 1 : d <= 3 ? 2 : 3;

        // Expand width
        let w = 1;
        while (x + w < GRID_X) {
          const nd = mask[(x + w) + y * GRID_X];
          const nBucket = nd <= 1 ? 1 : nd <= 3 ? 2 : 3;
          if (nd !== 0 && nBucket === dBucket) {
            w++;
          } else {
            break;
          }
        }

        // Expand height
        let h = 1;
        outer:
        while (y + h < GRID_Y) {
          for (let dx = 0; dx < w; dx++) {
            const nd = mask[(x + dx) + (y + h) * GRID_X];
            const nBucket = nd <= 1 ? 1 : nd <= 3 ? 2 : 3;
            if (nd === 0 || nBucket !== dBucket) break outer;
          }
          h++;
        }

        // Clear merged region
        for (let dy = 0; dy < h; dy++) {
          for (let dx = 0; dx < w; dx++) {
            mask[(x + dx) + (y + dy) * GRID_X] = 0;
          }
        }

        quads.push({ x, y, z, w, h, depth: d });
        x += w;
      }
    }
  }

  return quads;
}

// ---------------------------------------------------------------------------
// Shader
// ---------------------------------------------------------------------------

const waterVertexShader = /* glsl */ `
  varying vec2 vUv;
  varying vec3 vWorldPos;
  varying float vDepth;

  attribute float depth;
  uniform float uTime;

  void main() {
    vUv = uv;
    vDepth = depth;
    vec4 worldPos = modelMatrix * vec4(position, 1.0);

    // Gentle wave displacement — makes the surface undulate
    float wave1 = sin(worldPos.x * 1.2 + uTime * 0.8) * 0.04;
    float wave2 = sin(worldPos.z * 0.9 + uTime * 0.6) * 0.03;
    worldPos.y += wave1 + wave2;

    vWorldPos = worldPos.xyz;
    gl_Position = projectionMatrix * viewMatrix * worldPos;
  }
`;

const waterFragmentShader = /* glsl */ `
  uniform float uTime;
  uniform vec3 uShallowColor;
  uniform vec3 uDeepColor;
  uniform vec3 uSunDirection;
  uniform float uSunIntensity;
  uniform float uRainStrength;

  varying vec2 vUv;
  varying vec3 vWorldPos;
  varying float vDepth;

  // Simple hash-based noise for ripples (no texture needed)
  vec2 hash2(vec2 p) {
    p = vec2(dot(p, vec2(127.1, 311.7)), dot(p, vec2(269.5, 183.3)));
    return -1.0 + 2.0 * fract(sin(p) * 43758.5453123);
  }

  // Scalar hash for sparkle cells
  float hash1(vec2 p) {
    return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453123);
  }

  // Smooth noise
  float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);

    float a = dot(hash2(i + vec2(0.0, 0.0)), f - vec2(0.0, 0.0));
    float b = dot(hash2(i + vec2(1.0, 0.0)), f - vec2(1.0, 0.0));
    float c = dot(hash2(i + vec2(0.0, 1.0)), f - vec2(0.0, 1.0));
    float d = dot(hash2(i + vec2(1.0, 1.0)), f - vec2(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
  }

  // Layered ripple normal from scrolling noise
  vec3 rippleNormal(vec2 worldXY, float time) {
    // Two scrolling layers at different speeds and scales
    float n1 = noise(worldXY * 0.8 + vec2(time * 0.3, time * 0.2));
    float n2 = noise(worldXY * 1.6 - vec2(time * 0.15, time * 0.35));
    float n3 = noise(worldXY * 3.2 + vec2(time * 0.4, -time * 0.25));

    // Combine: large slow waves + small fast ripples
    float combined = n1 * 0.5 + n2 * 0.3 + n3 * 0.2;

    // Compute normal from heightfield gradient
    float eps = 0.05;
    float hx = noise((worldXY + vec2(eps, 0.0)) * 0.8 + vec2(time * 0.3, time * 0.2)) * 0.5
             + noise((worldXY + vec2(eps, 0.0)) * 1.6 - vec2(time * 0.15, time * 0.35)) * 0.3
             + noise((worldXY + vec2(eps, 0.0)) * 3.2 + vec2(time * 0.4, -time * 0.25)) * 0.2;
    float hy = noise((worldXY + vec2(0.0, eps)) * 0.8 + vec2(time * 0.3, time * 0.2)) * 0.5
             + noise((worldXY + vec2(0.0, eps)) * 1.6 - vec2(time * 0.15, time * 0.35)) * 0.3
             + noise((worldXY + vec2(0.0, eps)) * 3.2 + vec2(time * 0.4, -time * 0.25)) * 0.2;

    float dx = (hx - combined) / eps;
    float dy = (hy - combined) / eps;

    // Ripple strength — visible but cozy
    float strength = 0.22;
    return normalize(vec3(-dx * strength, -dy * strength, 1.0));
  }

  // Dancing sun sparkles: cell-based glints that wink on and off
  float sparkle(vec2 worldXY, float time) {
    // Tile the surface into cells; each cell has a random phase
    vec2 cell = floor(worldXY * 2.0); // ~0.5 voxel sparkle cells
    float phase = hash1(cell) * 6.283;
    float speed = 1.0 + hash1(cell + 100.0) * 2.0;

    // Sharp pulse: only bright for a narrow window of the sine cycle
    float pulse = sin(time * speed + phase);
    pulse = smoothstep(0.85, 0.95, pulse); // bright only at peak

    // Spatial variation: center of cell is brightest
    vec2 cellUV = fract(worldXY * 2.0) - 0.5;
    float dist = length(cellUV);
    float spot = smoothstep(0.3, 0.0, dist);

    return pulse * spot;
  }

  void main() {
    // Depth-based color blend: shallow → clear teal, deep → rich dark blue-green
    float depthFactor = clamp(vDepth / 5.0, 0.0, 1.0);
    vec3 baseColor = mix(uShallowColor, uDeepColor, depthFactor);

    // Shoreline foam: shallow edges get a soft white fringe
    float foamFactor = smoothstep(2.0, 0.5, vDepth);
    float foamNoise = noise(vWorldPos.xy * 4.0 + vec2(uTime * 0.5, uTime * 0.3));
    float foam = foamFactor * smoothstep(-0.1, 0.3, foamNoise);
    baseColor = mix(baseColor, vec3(0.75, 0.88, 0.90), foam * 0.5);

    // Animated ripple normal
    vec3 normal = rippleNormal(vWorldPos.xy, uTime);

    // Simple directional light (sun reflection)
    float NdotL = max(dot(normal, normalize(uSunDirection)), 0.0);
    vec3 lit = baseColor * (0.6 + 0.4 * NdotL * uSunIntensity);

    // Broad specular highlight — sun glint on ripples
    vec3 viewDir = vec3(0.0, 0.0, 1.0); // orthographic, looking down-ish
    vec3 halfDir = normalize(normalize(uSunDirection) + viewDir);
    float spec = pow(max(dot(normal, halfDir), 0.0), 48.0);
    lit += vec3(1.0, 0.95, 0.8) * spec * 0.5 * uSunIntensity;

    // Dancing sun sparkles — scattered bright glints across the surface
    float glint = sparkle(vWorldPos.xy, uTime);
    lit += vec3(1.0, 0.97, 0.85) * glint * 0.7 * uSunIntensity;

    // Sky reflection tint (subtle, based on normal perturbation)
    float skyFresnel = 1.0 - max(dot(normal, vec3(0.0, 0.0, 1.0)), 0.0);
    skyFresnel = pow(skyFresnel, 3.0) * 0.2;
    lit += vec3(0.53, 0.81, 0.92) * skyFresnel; // sky blue tint

    // Rain ripple rings: concentric circles from random drop points
    if (uRainStrength > 0.0) {
      float rippleSum = 0.0;
      for (int i = 0; i < 6; i++) {
        // Each "drop" has a unique position and phase
        float fi = float(i);
        vec2 dropPos = vec2(
          hash1(vec2(fi * 7.13, floor(uTime * 0.8 + fi * 1.7))) * 60.0,
          hash1(vec2(fi * 13.37, floor(uTime * 0.8 + fi * 1.7) + 100.0)) * 60.0
        );
        float dropAge = fract(uTime * 0.8 + fi * 1.7 / 6.0);
        float radius = dropAge * 3.0;
        float ring = abs(length(vWorldPos.xz - dropPos) - radius);
        float ripple = smoothstep(0.15, 0.0, ring) * (1.0 - dropAge);
        rippleSum += ripple;
      }
      lit += vec3(0.8, 0.9, 1.0) * rippleSum * uRainStrength * 0.3;
    }

    // Caustic patterns — bright refraction lines that shift and dance
    float c1 = noise(vWorldPos.xy * 2.5 + vec2(uTime * 0.4, uTime * 0.3));
    float c2 = noise(vWorldPos.xy * 3.7 - vec2(uTime * 0.25, uTime * 0.45));
    float caustic = abs(c1 + c2); // interference pattern creates sharp bright lines
    caustic = pow(caustic, 1.5) * 0.35; // brighten the peaks
    // Caustics strongest in shallow water (like a real pool)
    float causticStrength = (1.0 - depthFactor) * uSunIntensity;
    lit += vec3(0.85, 0.95, 0.90) * caustic * causticStrength;

    // Depth-based opacity: shallow water is see-through, deep is opaque
    // Shore edges slightly more opaque so foam reads clearly
    float alpha = mix(0.45, 0.88, depthFactor);
    alpha = mix(alpha, max(alpha, 0.6), foam);

    gl_FragColor = vec4(lit, alpha);
  }
`;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

let waterMesh: THREE.Mesh | null = null;
let waterMaterial: THREE.ShaderMaterial | null = null;

/**
 * Build (or rebuild) the water surface mesh from the current grid state.
 * Returns a Three.js mesh with the custom water shader, or null if no water.
 */
export function buildWaterMesh(grid: Uint8Array): THREE.Mesh | null {
  const columns = findWaterSurfaces(grid);
  if (columns.length === 0) return null;

  const quads = mergeWaterSurfaces(columns);
  if (quads.length === 0) return null;

  // Build geometry: each quad = 2 triangles = 6 vertices
  const vertCount = quads.length * 6;
  const positions = new Float32Array(vertCount * 3);
  const uvs = new Float32Array(vertCount * 2);
  const depths = new Float32Array(vertCount);

  let vi = 0;

  for (const quad of quads) {
    // Water surface sits at top of the water cell (z + 1), slightly inset
    // Y-up swap: sim Z → Three.js Y, sim Y → Three.js Z
    const threeY = quad.z + 1.0 - 0.05; // tiny offset below z+1 to avoid z-fighting
    const x0 = quad.x;
    const z0 = quad.y;  // sim Y → Three.js Z
    const x1 = quad.x + quad.w;
    const z1 = quad.y + quad.h;
    const d = quad.depth;

    // 4 corners: BL, BR, TR, TL (on XZ plane at height threeY)
    const corners: [number, number, number][] = [
      [x0, threeY, z0], [x1, threeY, z0], [x1, threeY, z1], [x0, threeY, z1],
    ];
    const cornerUVs: [number, number][] = [
      [x0, z0], [x1, z0], [x1, z1], [x0, z1],
    ];

    // Two triangles (reversed winding for Y↔Z handedness swap)
    const indices = [0, 2, 1, 0, 3, 2];
    for (const ci of indices) {
      positions[vi * 3] = corners[ci][0];
      positions[vi * 3 + 1] = corners[ci][1];
      positions[vi * 3 + 2] = corners[ci][2];

      // UV = world position (for tiling ripple patterns)
      uvs[vi * 2] = cornerUVs[ci][0];
      uvs[vi * 2 + 1] = cornerUVs[ci][1];

      depths[vi] = d;
      vi++;
    }
  }

  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3));
  geometry.setAttribute('uv', new THREE.BufferAttribute(uvs, 2));
  geometry.setAttribute('depth', new THREE.BufferAttribute(depths, 1));

  // Compute a flat +Y normal for the water surface (Y-up)
  geometry.computeVertexNormals();

  // Create or reuse shader material
  if (!waterMaterial) {
    waterMaterial = new THREE.ShaderMaterial({
      vertexShader: waterVertexShader,
      fragmentShader: waterFragmentShader,
      uniforms: {
        uTime: { value: 0 },
        uShallowColor: { value: new THREE.Color(0.22, 0.58, 0.55) },  // bright teal-green
        uDeepColor: { value: new THREE.Color(0.05, 0.18, 0.30) },      // dark blue-green
        uSunDirection: { value: new THREE.Vector3(0.5, -0.3, 0.8).normalize() },
        uSunIntensity: { value: 1.2 },
        uRainStrength: { value: 0 },
      },
      transparent: true,
      depthWrite: false,   // transparent surfaces shouldn't write depth
      side: THREE.DoubleSide,
    });
  }

  const mesh = new THREE.Mesh(geometry, waterMaterial);
  mesh.name = 'water_surface';
  mesh.renderOrder = 1; // render after opaque terrain

  waterMesh = mesh;
  return mesh;
}

/**
 * Update the water shader time uniform for animation.
 * Call each frame with elapsed seconds.
 */
export function updateWaterTime(elapsedSeconds: number): void {
  if (waterMaterial) {
    waterMaterial.uniforms.uTime.value = elapsedSeconds;
  }
}

/**
 * Update sun direction in water shader (if lighting changes).
 */
export function updateWaterSun(direction: THREE.Vector3, intensity: number): void {
  if (waterMaterial) {
    waterMaterial.uniforms.uSunDirection.value.copy(direction).normalize();
    waterMaterial.uniforms.uSunIntensity.value = intensity;
  }
}

/**
 * Set rain strength for water surface ripple rings (0 = no rain, 1 = full rain).
 */
export function updateWaterRain(strength: number): void {
  if (waterMaterial) {
    waterMaterial.uniforms.uRainStrength.value = strength;
  }
}

/**
 * Count water surface cells in the grid and return a few frontier positions.
 * "Frontier" = water cells adjacent to non-water (the expanding edge).
 * Returns { count, frontier: [x, y, z][] } for up to maxFrontier positions.
 */
export function scanWaterFrontier(grid: Uint8Array, maxFrontier = 8): { count: number; frontier: [number, number, number][] } {
  let count = 0;
  const frontier: [number, number, number][] = [];

  for (let y = 0; y < GRID_Y; y++) {
    for (let x = 0; x < GRID_X; x++) {
      for (let z = GRID_Z - 1; z >= 0; z--) {
        const idx = (x + y * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
        if (grid[idx] !== Material.Water) continue;

        // Check if air above (surface cell)
        const aboveZ = z + 1;
        if (aboveZ < GRID_Z) {
          const ai = (x + y * GRID_X + aboveZ * GRID_X * GRID_Y) * VOXEL_BYTES;
          if (grid[ai] !== Material.Air) continue;
        }

        count++;

        // Frontier: has at least one non-water horizontal neighbor
        if (frontier.length < maxFrontier) {
          const dirs = [[1, 0], [-1, 0], [0, 1], [0, -1]];
          for (const [dx, dy] of dirs) {
            const nx = x + dx, ny = y + dy;
            if (nx < 0 || nx >= GRID_X || ny < 0 || ny >= GRID_Y) {
              frontier.push([x, y, z]);
              break;
            }
            const ni = (nx + ny * GRID_X + z * GRID_X * GRID_Y) * VOXEL_BYTES;
            if (grid[ni] !== Material.Water) {
              frontier.push([x, y, z]);
              break;
            }
          }
        }

        break; // only surface per column
      }
    }
  }

  return { count, frontier };
}
