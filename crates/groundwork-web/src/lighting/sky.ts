/**
 * Sky gradient background — a large sphere with a vertical color gradient.
 * Top color and bottom color are updated each frame by the DayCycle.
 *
 * Uses a custom ShaderMaterial so the gradient is smooth and resolution-independent.
 */

import * as THREE from 'three';

const vertexShader = /* glsl */ `
varying vec3 vWorldPosition;
void main() {
  vec4 worldPos = modelMatrix * vec4(position, 1.0);
  vWorldPosition = worldPos.xyz;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}
`;

const fragmentShader = /* glsl */ `
uniform vec3 topColor;
uniform vec3 bottomColor;
uniform vec3 horizonColor;
uniform float uNightAmount;
uniform float uTime;
uniform float uCloudDensity;
uniform float uRainbow;
uniform vec3 uSunDir;
varying vec3 vWorldPosition;

// Simple pseudo-random hash for star placement
float hash(vec2 p) {
  return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}
float hash1(float p) {
  return fract(sin(p * 78.233) * 43758.5453);
}

// Value noise for cloud shapes
float vnoise(vec2 p) {
  vec2 i = floor(p);
  vec2 f = fract(p);
  f = f * f * (3.0 - 2.0 * f);
  float a = hash(i);
  float b = hash(i + vec2(1.0, 0.0));
  float c = hash(i + vec2(0.0, 1.0));
  float d = hash(i + vec2(1.0, 1.0));
  return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

// Fractal Brownian motion — 4 octaves for soft fluffy clouds
float fbm(vec2 p) {
  float f = 0.0;
  f += 0.50 * vnoise(p); p *= 2.03;
  f += 0.25 * vnoise(p); p *= 2.01;
  f += 0.125 * vnoise(p); p *= 2.02;
  f += 0.0625 * vnoise(p);
  return f / 0.9375;
}

void main() {
  vec3 dir = normalize(vWorldPosition);
  float h = dir.y; // -1 (nadir) to +1 (zenith)

  // Three-band gradient: bottom → horizon → top
  vec3 color;
  if (h > 0.0) {
    float t = smoothstep(0.0, 0.6, h);
    color = mix(horizonColor, topColor, t);
  } else {
    float t = smoothstep(-0.5, 0.0, h);
    color = mix(bottomColor, horizonColor, t);
  }

  // Sun disc: warm glowing circle at sun position, most visible at dawn/dusk
  {
    float sunDot = dot(dir, uSunDir);
    // Tight bright core + soft halo
    float core = smoothstep(0.9985, 0.9995, sunDot);
    float halo = smoothstep(0.96, 0.999, sunDot);
    // Sun color: white-hot core, warm amber halo
    vec3 sunCore = vec3(1.0, 0.98, 0.92);
    vec3 sunHalo = mix(vec3(1.0, 0.7, 0.3), vec3(1.0, 0.95, 0.8), smoothstep(0.96, 0.999, sunDot));
    // Fade at night (sun below horizon)
    float sunVis = smoothstep(-0.02, 0.08, uSunDir.y) * (1.0 - uNightAmount);
    color += sunCore * core * 1.2 * sunVis;
    color += sunHalo * halo * 0.4 * sunVis;
  }

  // Clouds: soft drifting shapes above horizon, fade at night
  if (h > 0.0 && uCloudDensity > 0.0) {
    // Project onto dome — stretch near horizon for depth
    vec2 cloudUV = dir.xz / max(h, 0.06) * 1.2;
    // Slow drift: main layer + slight vertical creep
    cloudUV += vec2(uTime * 0.008, uTime * 0.002);

    float n = fbm(cloudUV);
    // Second layer offset for depth
    float n2 = fbm(cloudUV * 0.7 + vec2(uTime * 0.005, 3.7));

    // Blend layers: thick cumulus + thin wispy
    float cloudShape = smoothstep(0.50 - uCloudDensity * 0.12, 0.68, n) * 0.7
                     + smoothstep(0.45 - uCloudDensity * 0.10, 0.62, n2) * 0.3;

    // Height mask: fade near horizon (avoid harsh line) and at zenith
    float heightMask = smoothstep(0.02, 0.18, h) * smoothstep(0.85, 0.35, h);
    cloudShape *= heightMask;

    // Cloud color: warm white tinted by sky, darker at base
    vec3 cloudBright = mix(vec3(1.0, 0.98, 0.95), horizonColor, 0.2);
    vec3 cloudDark = mix(horizonColor, vec3(0.7, 0.7, 0.72), 0.3);
    // Use noise as self-shadow approximation
    vec3 cloudCol = mix(cloudDark, cloudBright, n * 0.6 + 0.4);

    float dayAmount = 1.0 - uNightAmount;
    color = mix(color, cloudCol, cloudShape * dayAmount * 0.55);
  }

  // Stars: faint points above horizon, visible only at night
  if (h > 0.05 && uNightAmount > 0.0) {
    // Grid-based star field using direction as UV
    vec2 starUV = dir.xz / max(h, 0.01) * 8.0;
    vec2 cell = floor(starUV);
    float starVal = hash(cell);
    // ~5% of cells have a star
    if (starVal > 0.95) {
      float dist = length(fract(starUV) - 0.5);
      float star = smoothstep(0.15, 0.0, dist) * (0.5 + 0.5 * starVal);
      float heightFade = smoothstep(0.05, 0.4, h);
      // Twinkle: each star has unique phase and speed from its hash
      float twinklePhase = hash(cell + 100.0) * 6.283;
      float twinkleSpeed = 0.5 + hash(cell + 200.0) * 1.5;
      float twinkle = 0.6 + 0.4 * sin(uTime * twinkleSpeed + twinklePhase);
      color += vec3(star * uNightAmount * heightFade * 0.7 * twinkle);
    }

    // Shooting star: one every ~45 seconds, lasts 0.6 seconds
    float shootSlot = floor(uTime / 45.0); // which shooting star event
    float shootPhase = fract(uTime / 45.0) * 45.0; // time within this slot
    if (shootPhase < 0.6) {
      // Shooting star path: random start direction, streaks across sky
      float seed = shootSlot * 7.13;
      vec2 startDir = normalize(vec2(hash1(seed) - 0.5, hash1(seed + 1.0) - 0.5));
      float startAngle = hash1(seed + 2.0) * 3.14159;
      float startH = 0.3 + hash1(seed + 3.0) * 0.5; // height 0.3-0.8
      float progress = shootPhase / 0.6; // 0→1

      // Streak position in sky-UV space
      vec2 meteorUV = dir.xz / max(dir.y, 0.1);
      vec2 meteorStart = startDir * 3.0;
      vec2 meteorPos = meteorStart + normalize(vec2(cos(startAngle), sin(startAngle))) * progress * 4.0;
      float meteorDist = length(meteorUV - meteorPos);

      // Streak shape: bright head + fading tail
      float headBright = smoothstep(0.3, 0.0, meteorDist) * (1.0 - progress);
      // Only show above horizon and at the right height range
      if (dir.y > 0.1 && headBright > 0.01) {
        color += vec3(1.0, 0.95, 0.8) * headBright * uNightAmount * 0.8;
      }
    }
  }

  // Rainbow arc: appears after rain, fades over time
  if (uRainbow > 0.01 && h > 0.0) {
    // Rainbow appears opposite the sun — fixed position in the sky
    // Use xz angle relative to a fixed "anti-sun" direction
    float angle = atan(dir.z, dir.x);
    // Arc center at elevation ~0.25, radius ~0.35 in angular space
    float arcDist = length(vec2(dir.x + 0.3, dir.y - 0.25));
    // Rainbow band: narrow ring at the right angular distance
    float band = smoothstep(0.30, 0.33, arcDist) * smoothstep(0.42, 0.39, arcDist);
    // Only show upper half of arc
    band *= smoothstep(0.0, 0.08, h);
    // Spectral colors from angular position within the band
    float bandPos = (arcDist - 0.30) / 0.12; // 0→1 across band width
    vec3 rainbow;
    if (bandPos < 0.2) rainbow = mix(vec3(0.55, 0.0, 0.55), vec3(0.0, 0.0, 0.9), bandPos / 0.2);
    else if (bandPos < 0.4) rainbow = mix(vec3(0.0, 0.0, 0.9), vec3(0.0, 0.7, 0.3), (bandPos - 0.2) / 0.2);
    else if (bandPos < 0.6) rainbow = mix(vec3(0.0, 0.7, 0.3), vec3(0.9, 0.9, 0.0), (bandPos - 0.4) / 0.2);
    else if (bandPos < 0.8) rainbow = mix(vec3(0.9, 0.9, 0.0), vec3(0.9, 0.4, 0.0), (bandPos - 0.6) / 0.2);
    else rainbow = mix(vec3(0.9, 0.4, 0.0), vec3(0.9, 0.0, 0.0), (bandPos - 0.8) / 0.2);
    color = mix(color, rainbow, band * uRainbow * 0.45);
  }

  gl_FragColor = vec4(color, 1.0);
}
`;

export interface SkyUniforms {
  topColor: THREE.IUniform<THREE.Color>;
  bottomColor: THREE.IUniform<THREE.Color>;
  horizonColor: THREE.IUniform<THREE.Color>;
  uNightAmount: THREE.IUniform<number>;
  uTime: THREE.IUniform<number>;
  uCloudDensity: THREE.IUniform<number>;
  uRainbow: THREE.IUniform<number>;
  uSunDir: THREE.IUniform<THREE.Vector3>;
  [key: string]: THREE.IUniform<unknown>;
}

/**
 * Create a sky dome (inverted sphere) with a vertical gradient.
 * Returns the mesh and the uniforms so the DayCycle can update colors each frame.
 */
export function createSkyGradient(scene: THREE.Scene): SkyUniforms {
  const uniforms: SkyUniforms = {
    topColor: { value: new THREE.Color(0x5599dd) },      // bright sky blue
    horizonColor: { value: new THREE.Color(0xccddcc) },   // haze — matches fog color
    bottomColor: { value: new THREE.Color(0x3a3228) },    // dark earth (below-horizon, rarely seen)
    uNightAmount: { value: 0.0 },                          // 0=day, 1=full night (drives star visibility)
    uTime: { value: 0.0 },                                 // elapsed time for shooting stars
    uCloudDensity: { value: 0.4 },                          // 0=clear sky, 1=overcast (0.4=scattered cumulus)
    uRainbow: { value: 0.0 },                                 // 0=invisible, 1=full rainbow (fades after rain)
    uSunDir: { value: new THREE.Vector3(0.5, 0.5, 0.0) },     // normalized sun direction in world space
  };

  const skyMat = new THREE.ShaderMaterial({
    vertexShader,
    fragmentShader,
    uniforms,
    side: THREE.BackSide,
    depthWrite: false,
    fog: false, // Sky dome must not be affected by scene fog
  });

  const skyGeo = new THREE.SphereGeometry(500, 32, 16);
  const skyMesh = new THREE.Mesh(skyGeo, skyMat);
  skyMesh.renderOrder = -1; // render behind everything
  scene.add(skyMesh);

  // Remove the flat background color — the sky dome replaces it
  scene.background = null;

  return uniforms;
}
