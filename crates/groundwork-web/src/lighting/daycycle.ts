/**
 * Time-of-day cycle: dawn → noon → golden hour → blue hour.
 * Lerps sun position, light colors, intensities, fog, and sky colors.
 *
 * Keyboard: '[' / ']' to step backward/forward, '\' to toggle auto-cycle.
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GROUND_LEVEL } from '../bridge';
import type { Lights } from './sun';

/** A snapshot of all light/color parameters at one time of day. */
interface LightPreset {
  /** Normalized time: 0 = midnight, 0.25 = dawn, 0.5 = noon, 0.75 = dusk */
  time: number;
  sunColor: THREE.Color;
  sunIntensity: number;
  /** Sun elevation angle in radians (0 = horizon, π/2 = overhead) */
  sunElevation: number;
  /** Sun azimuth offset from default, radians */
  sunAzimuth: number;
  hemiSkyColor: THREE.Color;
  hemiGroundColor: THREE.Color;
  hemiIntensity: number;
  ambientColor: THREE.Color;
  ambientIntensity: number;
  skyTop: THREE.Color;
  skyBottom: THREE.Color;
  fogColor: THREE.Color;
  fogDensity: number;
}

/** Pre-authored light presets for 4 key moments. */
const PRESETS: LightPreset[] = [
  {
    // Dawn — distinctly pink/peach, soft low light
    time: 0.25,
    sunColor: new THREE.Color(0xff8866),
    sunIntensity: 0.9,
    sunElevation: 0.15,
    sunAzimuth: -Math.PI / 3,
    hemiSkyColor: new THREE.Color(0xffbbaa),
    hemiGroundColor: new THREE.Color(0x443322),
    hemiIntensity: 0.4,
    ambientColor: new THREE.Color(0xcc8899),
    ambientIntensity: 0.25,
    skyTop: new THREE.Color(0x5566bb),
    skyBottom: new THREE.Color(0xdd8866),
    fogColor: new THREE.Color(0xddaa88),
    fogDensity: 0.003,
  },
  {
    // Noon — bright, clean, slightly cool white light
    time: 0.5,
    sunColor: new THREE.Color(0xffffff),
    sunIntensity: 2.0,
    sunElevation: Math.PI / 2.5,
    sunAzimuth: 0,
    hemiSkyColor: new THREE.Color(0xccddff),
    hemiGroundColor: new THREE.Color(0x556644),
    hemiIntensity: 0.8,
    ambientColor: new THREE.Color(0x99aacc),
    ambientIntensity: 0.15,
    skyTop: new THREE.Color(0x3388dd),
    skyBottom: new THREE.Color(0x88aa77),
    fogColor: new THREE.Color(0xccddcc),
    fogDensity: 0.003,
  },
  {
    // Golden hour — deep warm amber, long shadows
    time: 0.75,
    sunColor: new THREE.Color(0xffcc66),
    sunIntensity: 1.4,
    sunElevation: 0.25,
    sunAzimuth: Math.PI / 3,
    hemiSkyColor: new THREE.Color(0xffddaa),
    hemiGroundColor: new THREE.Color(0x554422),
    hemiIntensity: 0.6,
    ambientColor: new THREE.Color(0x887755),
    ambientIntensity: 0.2,
    skyTop: new THREE.Color(0x5588bb),
    skyBottom: new THREE.Color(0xcc8844),
    fogColor: new THREE.Color(0xddaa66),
    fogDensity: 0.003,
  },
  {
    // Night / Blue hour — cool blue moonlight, stars visible, cozy not dark
    time: 1.0, // wraps to 0.0
    sunColor: new THREE.Color(0x334466),
    sunIntensity: 0.05,    // barely visible moon-like glow from horizon
    sunElevation: 0.03,
    sunAzimuth: Math.PI * 0.8,
    hemiSkyColor: new THREE.Color(0x223355),
    hemiGroundColor: new THREE.Color(0x111122),
    hemiIntensity: 0.15,
    ambientColor: new THREE.Color(0x4466aa),  // cool blue moonlight fill
    ambientIntensity: 0.35,                   // enough to see garden silhouettes
    skyTop: new THREE.Color(0x0a0a22),        // deep night sky (stars show here)
    skyBottom: new THREE.Color(0x1a1528),
    fogColor: new THREE.Color(0x1a2233),
    fogDensity: 0.006,
  },
];

/** Duration of one full day cycle in seconds. */
const DAY_LENGTH_S = 600; // 10 minutes per full day — slow enough to enjoy each mood

/** Smoothly lerp two colors. */
function lerpColor(out: THREE.Color, a: THREE.Color, b: THREE.Color, t: number): THREE.Color {
  out.r = a.r + (b.r - a.r) * t;
  out.g = a.g + (b.g - a.g) * t;
  out.b = a.b + (b.b - a.b) * t;
  return out;
}

/** Lerp a scalar. */
function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}

export class DayCycle {
  /** Normalized time of day: 0.0–1.0 */
  private time = 0.5; // Start at noon — bright blue sky
  private autoCycle = true; // ON by default — player experiences all lighting moods
  private elapsedTime = 0; // total elapsed seconds for sky animations

  /** Scratch colors to avoid allocation in the loop. */
  private readonly _c1 = new THREE.Color();
  private readonly _c2 = new THREE.Color();

  /** Get current time (0–1). */
  getTime(): number {
    return this.time;
  }

  /** Set time directly (0–1). */
  setTime(t: number): void {
    this.time = ((t % 1) + 1) % 1;
  }

  /** Toggle auto-cycle. */
  toggleAuto(): void {
    this.autoCycle = !this.autoCycle;
  }

  /** Is auto-cycling? */
  isAutoCycling(): boolean {
    return this.autoCycle;
  }

  /** Step time forward or backward by a fraction. */
  step(amount: number): void {
    this.time = ((this.time + amount) % 1 + 1) % 1;
  }

  /**
   * Advance time (if auto-cycling) and update all scene lighting.
   * Call once per frame with deltaTime in seconds.
   */
  update(
    deltaS: number,
    lights: Lights,
    scene: THREE.Scene,
    skyUniforms?: { topColor: THREE.IUniform<THREE.Color>; bottomColor: THREE.IUniform<THREE.Color>; horizonColor?: THREE.IUniform<THREE.Color>; uNightAmount?: THREE.IUniform<number>; uTime?: THREE.IUniform<number>; uSunDir?: THREE.IUniform<THREE.Vector3> },
  ): void {
    this.elapsedTime += deltaS;
    if (this.autoCycle) {
      this.time = (this.time + deltaS / DAY_LENGTH_S) % 1;
    }

    // Find surrounding presets
    const { a, b, t } = this.findPresets();

    // --- Sun ---
    lights.sun.color.copy(lerpColor(this._c1, a.sunColor, b.sunColor, t));
    let sunIntensity = lerp(a.sunIntensity, b.sunIntensity, t);

    // Cloud shadow pulses: overlapping slow sine waves create organic
    // brightness variation during daytime (as if clouds drift overhead).
    // Only modulate when sun is strong enough to notice — fades out at night.
    if (sunIntensity > 0.3) {
      const e = this.elapsedTime;
      const cloudWave =
        Math.sin(e * 0.32) * 0.06 +   // ~20s period
        Math.sin(e * 0.18) * 0.04 +   // ~35s period
        Math.sin(e * 0.10) * 0.03;    // ~63s period
      // Fade modulation by how "daytime" it is (0 at night, full at noon)
      const dayFade = Math.min(1, (sunIntensity - 0.3) / 1.0);
      sunIntensity *= 1.0 + cloudWave * dayFade;
    }
    lights.sun.intensity = sunIntensity;

    const elev = lerp(a.sunElevation, b.sunElevation, t);
    const azim = lerp(a.sunAzimuth, b.sunAzimuth, t);
    const sunDist = 100;
    const cx = GRID_X / 2;
    const cy = GRID_Y / 2;
    lights.sun.position.set(
      cx + sunDist * Math.cos(elev) * Math.cos(azim),
      cy + sunDist * Math.cos(elev) * Math.sin(azim),
      GROUND_LEVEL + sunDist * Math.sin(elev),
    );
    lights.sun.target.position.set(cx, cy, GROUND_LEVEL);

    // --- Hemisphere ---
    lights.hemisphere.color.copy(lerpColor(this._c1, a.hemiSkyColor, b.hemiSkyColor, t));
    lights.hemisphere.groundColor.copy(lerpColor(this._c2, a.hemiGroundColor, b.hemiGroundColor, t));
    lights.hemisphere.intensity = lerp(a.hemiIntensity, b.hemiIntensity, t);

    // --- Ambient ---
    lights.ambient.color.copy(lerpColor(this._c1, a.ambientColor, b.ambientColor, t));
    lights.ambient.intensity = lerp(a.ambientIntensity, b.ambientIntensity, t);

    // --- Fog ---
    const fogColor = lerpColor(this._c1, a.fogColor, b.fogColor, t);
    if (scene.fog instanceof THREE.FogExp2) {
      scene.fog.color.copy(fogColor);
      scene.fog.density = lerp(a.fogDensity, b.fogDensity, t);
    }

    // --- Sky gradient ---
    if (skyUniforms) {
      lerpColor(skyUniforms.topColor.value, a.skyTop, b.skyTop, t);
      lerpColor(skyUniforms.bottomColor.value, a.skyBottom, b.skyBottom, t);
      // Horizon color matches fog — seamless blend at any camera angle
      if (skyUniforms.horizonColor) {
        lerpColor(skyUniforms.horizonColor.value, a.fogColor, b.fogColor, t);
      }
      // Elapsed time for sky animations (shooting stars)
      if (skyUniforms.uTime) {
        skyUniforms.uTime.value = this.elapsedTime;
      }
      // Sun direction for sun disc rendering in sky shader
      if (skyUniforms.uSunDir) {
        // Normalize to unit direction — Y-up for sky shader (Y = height = sin(elev))
        skyUniforms.uSunDir.value.set(
          Math.cos(elev) * Math.cos(azim),
          Math.sin(elev),
          Math.cos(elev) * Math.sin(azim),
        );
      }
      // Night amount for star visibility: peaks at midnight (0.0), zero at noon (0.5)
      if (skyUniforms.uNightAmount) {
        // 0.0=midnight → 1.0, 0.25=dawn → 0.0, 0.5=noon → 0.0, 0.75=golden hour → 0.3, 1.0=blue hour → 0.9
        const night = this.time <= 0.25
          ? 1.0 - this.time / 0.25  // midnight→dawn: 1→0
          : this.time >= 0.75
            ? (this.time - 0.75) / 0.25 // golden hour→midnight: 0→1
            : 0.0; // daytime: 0
        skyUniforms.uNightAmount.value = Math.max(0, Math.min(1, night));
      }
    }
  }

  /** Find the two presets surrounding current time and the interpolation factor. */
  private findPresets(): { a: LightPreset; b: LightPreset; t: number } {
    const time = this.time;
    const n = PRESETS.length;

    // Find the preset just before current time
    let idx = 0;
    for (let i = 0; i < n; i++) {
      const presetTime = PRESETS[i].time % 1;
      if (presetTime <= time) idx = i;
    }

    const a = PRESETS[idx];
    const b = PRESETS[(idx + 1) % n];

    const aTime = a.time % 1;
    let bTime = b.time % 1;
    // Handle wrap-around (blue hour at 1.0/0.0 → dawn at 0.25)
    if (bTime <= aTime) bTime += 1;
    let adjustedTime = time;
    if (adjustedTime < aTime) adjustedTime += 1;

    const range = bTime - aTime;
    const t = range > 0 ? Math.max(0, Math.min(1, (adjustedTime - aTime) / range)) : 0;

    return { a, b, t };
  }
}
