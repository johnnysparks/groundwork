/**
 * Post-processing pipeline: SSAO → Bloom → Tilt-shift DOF → Color grading → Vignette.
 *
 * All effects tuned for the orthographic diorama camera with warm, cozy aesthetics.
 * Intensities are deliberately subtle — enhance depth and warmth without overwhelming.
 */

import * as THREE from 'three';
import { EffectComposer } from 'three/addons/postprocessing/EffectComposer.js';
import { RenderPass } from 'three/addons/postprocessing/RenderPass.js';
import { ShaderPass } from 'three/addons/postprocessing/ShaderPass.js';
import { UnrealBloomPass } from 'three/addons/postprocessing/UnrealBloomPass.js';
import { SSAOPass } from 'three/addons/postprocessing/SSAOPass.js';
import { OutputPass } from 'three/addons/postprocessing/OutputPass.js';

// ---------------------------------------------------------------------------
// Custom shaders
// ---------------------------------------------------------------------------

/**
 * Tilt-shift DOF: blurs top and bottom of screen to create a miniature/diorama look.
 * Focus band sits at the center of the screen. Blur increases with distance from center.
 * Uses 9-tap vertical gaussian weighted by distance from the focus strip.
 */
const TiltShiftShader = {
  uniforms: {
    tDiffuse: { value: null as THREE.Texture | null },
    resolution: { value: new THREE.Vector2(1, 1) },
    blurMax: { value: 4.5 },        // max blur in pixels — stronger miniature effect
    focusCenter: { value: 0.45 },    // Y center of focus band (0–1, slightly below middle)
    focusWidth: { value: 0.25 },     // narrower focus band — more pronounced DOF
  },
  vertexShader: /* glsl */ `
    varying vec2 vUv;
    void main() {
      vUv = uv;
      gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
    }
  `,
  fragmentShader: /* glsl */ `
    uniform sampler2D tDiffuse;
    uniform vec2 resolution;
    uniform float blurMax;
    uniform float focusCenter;
    uniform float focusWidth;

    varying vec2 vUv;

    void main() {
      float dist = abs(vUv.y - focusCenter);
      float blur = smoothstep(focusWidth * 0.5, focusWidth, dist) * blurMax / resolution.y;

      if (blur < 0.0001) {
        gl_FragColor = texture2D(tDiffuse, vUv);
        return;
      }

      // 9-tap gaussian kernel (sigma ≈ 2)
      vec4 sum = vec4(0.0);
      sum += texture2D(tDiffuse, vUv + vec2(0.0, -4.0 * blur)) * 0.0162;
      sum += texture2D(tDiffuse, vUv + vec2(0.0, -3.0 * blur)) * 0.0540;
      sum += texture2D(tDiffuse, vUv + vec2(0.0, -2.0 * blur)) * 0.1216;
      sum += texture2D(tDiffuse, vUv + vec2(0.0, -1.0 * blur)) * 0.1945;
      sum += texture2D(tDiffuse, vUv)                           * 0.2270;
      sum += texture2D(tDiffuse, vUv + vec2(0.0,  1.0 * blur)) * 0.1945;
      sum += texture2D(tDiffuse, vUv + vec2(0.0,  2.0 * blur)) * 0.1216;
      sum += texture2D(tDiffuse, vUv + vec2(0.0,  3.0 * blur)) * 0.0540;
      sum += texture2D(tDiffuse, vUv + vec2(0.0,  4.0 * blur)) * 0.0162;

      // Also sample horizontally for a softer look
      vec4 sumH = vec4(0.0);
      float blurH = blur * 0.6; // less horizontal blur than vertical
      sumH += texture2D(tDiffuse, vUv + vec2(-3.0 * blurH, 0.0)) * 0.0918;
      sumH += texture2D(tDiffuse, vUv + vec2(-2.0 * blurH, 0.0)) * 0.1553;
      sumH += texture2D(tDiffuse, vUv + vec2(-1.0 * blurH, 0.0)) * 0.2260;
      sumH += texture2D(tDiffuse, vUv)                            * 0.2538;
      sumH += texture2D(tDiffuse, vUv + vec2( 1.0 * blurH, 0.0)) * 0.2260;
      sumH += texture2D(tDiffuse, vUv + vec2( 2.0 * blurH, 0.0)) * 0.1553;
      sumH += texture2D(tDiffuse, vUv + vec2( 3.0 * blurH, 0.0)) * 0.0918;

      gl_FragColor = mix(sum, sumH, 0.35);
    }
  `,
};

/**
 * Warm color grading: slight warmth shift, contrast boost, saturation bump.
 * Designed to complement the golden hour lighting and earthy material palette.
 */
const ColorGradeShader = {
  uniforms: {
    tDiffuse: { value: null as THREE.Texture | null },
    warmth: { value: 0.035 },      // red/yellow shift — cozy warmth
    contrast: { value: 1.10 },     // contrast multiplier around mid-gray
    saturation: { value: 1.18 },   // saturation multiplier — vivid greens and golds
  },
  vertexShader: /* glsl */ `
    varying vec2 vUv;
    void main() {
      vUv = uv;
      gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
    }
  `,
  fragmentShader: /* glsl */ `
    uniform sampler2D tDiffuse;
    uniform float warmth;
    uniform float contrast;
    uniform float saturation;

    varying vec2 vUv;

    void main() {
      vec4 texel = texture2D(tDiffuse, vUv);
      vec3 color = texel.rgb;

      // Warmth: shift reds up, blues down
      color.r += warmth;
      color.g += warmth * 0.4;
      color.b -= warmth * 0.3;

      // Contrast: pivot around mid-gray
      color = (color - 0.5) * contrast + 0.5;

      // Saturation: lerp toward luminance
      float lum = dot(color, vec3(0.299, 0.587, 0.114));
      color = mix(vec3(lum), color, saturation);

      gl_FragColor = vec4(clamp(color, 0.0, 1.0), texel.a);
    }
  `,
};

/**
 * Vignette: smooth radial darkening at screen edges.
 * Draws the eye to the center of the garden.
 */
const VignetteShader = {
  uniforms: {
    tDiffuse: { value: null as THREE.Texture | null },
    darkness: { value: 0.35 },   // noticeable darkening at corners — frames the garden
    offset: { value: 1.3 },      // starts slightly closer for a warmer frame
  },
  vertexShader: /* glsl */ `
    varying vec2 vUv;
    void main() {
      vUv = uv;
      gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
    }
  `,
  fragmentShader: /* glsl */ `
    uniform sampler2D tDiffuse;
    uniform float darkness;
    uniform float offset;

    varying vec2 vUv;

    void main() {
      vec4 texel = texture2D(tDiffuse, vUv);

      // Distance from center (0 at center, ~0.7 at corners)
      vec2 uv = (vUv - 0.5) * 2.0;
      float dist = length(uv);

      // Smooth falloff
      float vignette = smoothstep(offset, offset - 0.6, dist);
      vignette = mix(1.0 - darkness, 1.0, vignette);

      gl_FragColor = vec4(texel.rgb * vignette, texel.a);
    }
  `,
};

/**
 * Heat shimmer: subtle UV distortion that rises from the bottom of the screen.
 * Active during drought / hot midday — makes the air feel thick and heated.
 */
const HeatShimmerShader = {
  uniforms: {
    tDiffuse: { value: null as THREE.Texture | null },
    uTime: { value: 0 },
    uStrength: { value: 0 },  // 0=off, ~0.003=subtle shimmer
  },
  vertexShader: /* glsl */ `
    varying vec2 vUv;
    void main() {
      vUv = uv;
      gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
    }
  `,
  fragmentShader: /* glsl */ `
    uniform sampler2D tDiffuse;
    uniform float uTime;
    uniform float uStrength;
    varying vec2 vUv;

    void main() {
      if (uStrength < 0.0001) {
        gl_FragColor = texture2D(tDiffuse, vUv);
        return;
      }
      // Rising distortion stronger toward bottom of screen
      float rise = 1.0 - vUv.y; // stronger at bottom (ground)
      rise = rise * rise; // quadratic falloff
      float wave1 = sin(vUv.y * 40.0 + uTime * 2.5) * 0.5;
      float wave2 = sin(vUv.y * 25.0 - uTime * 1.8 + 3.0) * 0.5;
      vec2 offset = vec2(
        (wave1 + wave2) * uStrength * rise,
        sin(vUv.x * 30.0 + uTime * 1.5) * uStrength * rise * 0.3
      );
      gl_FragColor = texture2D(tDiffuse, vUv + offset);
    }
  `,
};

// ---------------------------------------------------------------------------
// Composer setup
// ---------------------------------------------------------------------------

export interface PostProcessing {
  composer: EffectComposer;
  /** Call on window resize */
  resize(width: number, height: number): void;
  /** Set scene desaturation (0 = full color, 1 = greyscale). Used for x-ray mode. */
  setDesaturation(amount: number): void;
  /** Adjust bloom strength — boost for golden hour warmth. */
  setBloomStrength(strength: number): void;
  /** Set heat shimmer intensity and animate. */
  setHeatShimmer(strength: number, time: number): void;
  /** Set ecosystem warmth boost (0 = no boost, 1 = full thriving glow). */
  setEcoWarmth(amount: number): void;
  /** Adjust DOF based on camera zoom level (1.0 = default). */
  setZoomDOF(zoom: number): void;
  /** Set night amount (0-1) for atmospheric color shift + vignette. */
  setNightAtmosphere(amount: number): void;
}

/**
 * Create the full post-processing pipeline.
 *
 * Pipeline order:
 *   RenderPass → SSAO → Bloom → Tilt-shift → Color grade → Vignette → Output
 */
export function createPostProcessing(
  renderer: THREE.WebGLRenderer,
  scene: THREE.Scene,
  camera: THREE.Camera,
  opts?: { mobile?: boolean },
): PostProcessing {
  const width = renderer.domElement.clientWidth;
  const height = renderer.domElement.clientHeight;
  const mobile = opts?.mobile ?? false;

  const composer = new EffectComposer(renderer);

  // 1. Render the scene
  const renderPass = new RenderPass(scene, camera);
  composer.addPass(renderPass);

  // 2. SSAO — disabled: creates dark halo artifacts at larger grid scales.
  // TODO: re-tune kernelRadius/minDistance/maxDistance for variable grid sizes.
  // const ssaoPass = new SSAOPass(scene, camera, width, height);
  // ssaoPass.kernelRadius = 2;
  // ssaoPass.minDistance = 0.002;
  // ssaoPass.maxDistance = 0.08;
  // ssaoPass.output = SSAOPass.OUTPUT.Default;
  // composer.addPass(ssaoPass);

  // 3. Bloom — warm glow on sunlit edges and golden foliage
  // On mobile: half resolution for cheaper fill rate
  const bloomRes = mobile
    ? new THREE.Vector2(Math.floor(width / 2), Math.floor(height / 2))
    : new THREE.Vector2(width, height);
  const bloomPass = new UnrealBloomPass(
    bloomRes,
    mobile ? 0.20 : 0.25,   // slightly reduced on mobile
    0.9,    // radius — wide, soft glow
    0.80,   // threshold — catches sunlit canopy highlights
  );
  composer.addPass(bloomPass);

  // 4. Heat shimmer — drought / midday air distortion
  const heatShimmerPass = new ShaderPass(HeatShimmerShader);
  composer.addPass(heatShimmerPass);

  // 5. Tilt-shift DOF — diorama miniature effect (skip on mobile — expensive blur)
  let tiltShiftPass: ShaderPass | null = null;
  if (!mobile) {
    tiltShiftPass = new ShaderPass(TiltShiftShader);
    tiltShiftPass.uniforms.resolution.value.set(width, height);
    composer.addPass(tiltShiftPass);
  }

  // 5. Color grading — warm, cozy tones
  const colorGradePass = new ShaderPass(ColorGradeShader);
  composer.addPass(colorGradePass);

  // 6. Vignette — draw eye to center
  const vignettePass = new ShaderPass(VignetteShader);
  composer.addPass(vignettePass);

  // 7. Output — tone mapping + sRGB conversion
  const outputPass = new OutputPass();
  composer.addPass(outputPass);

  const baseSaturation = colorGradePass.uniforms.saturation.value;
  const baseWarmth = colorGradePass.uniforms.warmth.value;

  return {
    composer,
    resize(w: number, h: number) {
      composer.setSize(w, h);
      if (tiltShiftPass) {
        tiltShiftPass.uniforms.resolution.value.set(w, h);
      }
    },
    setDesaturation(amount: number) {
      // amount: 0 = full color (normal), 1 = full greyscale (x-ray)
      colorGradePass.uniforms.saturation.value = baseSaturation * (1 - amount * 0.85);
    },
    /** Adjust bloom strength — use to boost warm glow at golden hour */
    setBloomStrength(strength: number) {
      bloomPass.strength = strength;
    },
    /** Set heat shimmer intensity and update time for animation */
    setHeatShimmer(strength: number, time: number) {
      heatShimmerPass.uniforms.uStrength.value = strength;
      heatShimmerPass.uniforms.uTime.value = time;
    },
    /** Set ecosystem warmth boost: healthy gardens glow slightly warmer and more saturated */
    setEcoWarmth(amount: number) {
      colorGradePass.uniforms.warmth.value = baseWarmth + amount * 0.015;
      colorGradePass.uniforms.saturation.value = baseSaturation + amount * 0.08;
    },
    /** Adjust DOF based on camera zoom — closer = tighter focus band, more blur */
    setZoomDOF(zoom: number) {
      if (!tiltShiftPass) return;
      // zoom 0.35 (far) → wide focus (0.35), weak blur (2.5)
      // zoom 1.0 (default) → moderate (0.25, 4.5)
      // zoom 4.0 (close) → narrow focus (0.12), strong blur (6.0)
      const t = Math.max(0, Math.min(1, (zoom - 0.35) / 3.65)); // 0-1 normalized
      tiltShiftPass.uniforms.focusWidth.value = 0.35 - t * 0.23;
      tiltShiftPass.uniforms.blurMax.value = 2.5 + t * 3.5;
    },
    /** Night atmosphere: stronger vignette + cooler color grade at night */
    setNightAtmosphere(amount: number) {
      // Vignette deepens at night: 0.35 → 0.55 (draws eye to illuminated center)
      vignettePass.uniforms.darkness.value = 0.35 + amount * 0.20;
      // Cool blue shift at night: warmth decreases, slight blue tint emerges
      if (amount > 0.01) {
        colorGradePass.uniforms.warmth.value = baseWarmth * (1 - amount * 0.8);
      }
    },
  };
}
