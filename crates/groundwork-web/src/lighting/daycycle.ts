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
    fogDensity: 0.001,
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
    fogDensity: 0.0006,
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
    fogDensity: 0.001,
  },
  {
    // Blue hour — cool blue-purple twilight, dramatic contrast
    time: 1.0, // wraps to 0.0
    sunColor: new THREE.Color(0x6677aa),
    sunIntensity: 0.25,
    sunElevation: 0.05,
    sunAzimuth: Math.PI * 0.8,
    hemiSkyColor: new THREE.Color(0x445588),
    hemiGroundColor: new THREE.Color(0x1a1a2a),
    hemiIntensity: 0.25,
    ambientColor: new THREE.Color(0x5566aa),
    ambientIntensity: 0.3,
    skyTop: new THREE.Color(0x111133),
    skyBottom: new THREE.Color(0x332244),
    fogColor: new THREE.Color(0x334466),
    fogDensity: 0.0025,
  },
];

/** Duration of one full day cycle in seconds. */
const DAY_LENGTH_S = 120;

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
  private autoCycle = false;

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
    skyUniforms?: { topColor: THREE.IUniform<THREE.Color>; bottomColor: THREE.IUniform<THREE.Color> },
  ): void {
    if (this.autoCycle) {
      this.time = (this.time + deltaS / DAY_LENGTH_S) % 1;
    }

    // Find surrounding presets
    const { a, b, t } = this.findPresets();

    // --- Sun ---
    lights.sun.color.copy(lerpColor(this._c1, a.sunColor, b.sunColor, t));
    lights.sun.intensity = lerp(a.sunIntensity, b.sunIntensity, t);

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
