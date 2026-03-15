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
    blurMax: { value: 3.0 },        // max blur in pixels
    focusCenter: { value: 0.45 },    // Y center of focus band (0–1, slightly below middle)
    focusWidth: { value: 0.3 },      // width of the sharp band (0–1)
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
    warmth: { value: 0.06 },       // red/yellow shift amount
    contrast: { value: 1.08 },     // contrast multiplier around mid-gray
    saturation: { value: 1.12 },   // saturation multiplier
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
    darkness: { value: 0.2 },    // gentle darkening at corners
    offset: { value: 1.4 },      // starts further from center
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

// ---------------------------------------------------------------------------
// Composer setup
// ---------------------------------------------------------------------------

export interface PostProcessing {
  composer: EffectComposer;
  /** Call on window resize */
  resize(width: number, height: number): void;
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
): PostProcessing {
  const width = renderer.domElement.clientWidth;
  const height = renderer.domElement.clientHeight;

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

  // 3. Bloom — subtle warm glow on sunlit edges
  const bloomPass = new UnrealBloomPass(
    new THREE.Vector2(width, height),
    0.15,   // strength — very subtle
    0.8,    // radius — wide, soft glow
    0.85,   // threshold — only brightest areas
  );
  composer.addPass(bloomPass);

  // 4. Tilt-shift DOF — diorama miniature effect
  const tiltShiftPass = new ShaderPass(TiltShiftShader);
  tiltShiftPass.uniforms.resolution.value.set(width, height);
  composer.addPass(tiltShiftPass);

  // 5. Color grading — warm, cozy tones
  const colorGradePass = new ShaderPass(ColorGradeShader);
  composer.addPass(colorGradePass);

  // 6. Vignette — draw eye to center
  const vignettePass = new ShaderPass(VignetteShader);
  composer.addPass(vignettePass);

  // 7. Output — tone mapping + sRGB conversion
  const outputPass = new OutputPass();
  composer.addPass(outputPass);

  return {
    composer,
    resize(w: number, h: number) {
      composer.setSize(w, h);
      tiltShiftPass.uniforms.resolution.value.set(w, h);
    },
  };
}
