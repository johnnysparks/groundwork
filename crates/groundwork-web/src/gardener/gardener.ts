/**
 * Garden gnome billboard sprite — infused with personality.
 *
 * A charming little character that walks to task locations and
 * executes garden work. Uses a procedural GLSL shader with
 * animatable body parts: arms, head tilt, eye blinks, tool in hand.
 *
 * Personality comes from idle behaviors (look around, yawn, stretch,
 * inspect plants, sit and rest), celebrations (happy jump on task
 * complete, dance when queue empties), and subtle reactions.
 *
 * Emotion particles float above the gnome's head: hearts when near
 * flowers, zzz when resting, sparkles on celebration, sweat when
 * working hard.
 */

import * as THREE from 'three';
import { GROUND_LEVEL, GRID_X, GRID_Y, ToolCode } from '../bridge';
import type { TaskQueue, GardenTask } from './queue';

// ─── State Machine ───────────────────────────────────────────────

/** Primary gnome states */
enum GnomeState {
  Idle,
  Walking,
  Working,
  Celebrating,
}

/** Sub-states for idle personality behaviors */
enum IdleBehavior {
  Standing,       // default — gentle breathing
  LookingAround,  // head turns side to side
  Yawning,        // big stretch, mouth open
  SittingDown,    // sits on ground, relaxed
  InspectingPlant, // kneels down, peers at something
  WavingAtCamera, // breaks fourth wall, waves at player
  Stretching,     // arms up big stretch after work
}

/** Sub-states for celebration */
enum CelebType {
  TaskDone,   // small hop
  QueueEmpty, // happy dance
  BigMilestone, // arms up, spin
}

// ─── Animation Timing ────────────────────────────────────────────

const IDLE_BEHAVIOR_MIN_WAIT = 2.0;  // seconds before first idle anim
const IDLE_BEHAVIOR_MAX_WAIT = 6.0;
const IDLE_BEHAVIOR_DURATION: Record<IdleBehavior, number> = {
  [IdleBehavior.Standing]: 0,
  [IdleBehavior.LookingAround]: 2.5,
  [IdleBehavior.Yawning]: 2.0,
  [IdleBehavior.SittingDown]: 4.0,
  [IdleBehavior.InspectingPlant]: 3.0,
  [IdleBehavior.WavingAtCamera]: 1.8,
  [IdleBehavior.Stretching]: 2.0,
};

const CELEB_DURATION: Record<CelebType, number> = {
  [CelebType.TaskDone]: 0.4,
  [CelebType.QueueEmpty]: 1.2,
  [CelebType.BigMilestone]: 1.8,
};

const WALK_SPEED = 8.0;     // voxels per second
const WORK_DURATION = 0.4;  // seconds per task
const BLINK_INTERVAL = 3.0; // seconds between blinks
const BLINK_DURATION = 0.15;

// ─── Emotion Particles ──────────────────────────────────────────

/** Types of floating emotion indicators */
enum EmotionType {
  Heart,    // near flowers / after planting
  Sparkle,  // celebration
  Sweat,    // working hard (many tasks)
  Zzz,      // resting / sitting
  Music,    // happy dance
  Exclaim,  // noticed something
}

interface EmotionParticle {
  type: EmotionType;
  x: number;
  y: number;
  z: number;
  age: number;
  maxAge: number;
  vx: number;
  vy: number;
}

// ─── Shaders ────────────────────────────────────────────────────

/** Gnome vertex shader — billboard with rich animation uniforms */
const GNOME_VERT = /* glsl */ `
  uniform float uTime;
  uniform float uBounce;      // vertical bounce amount
  uniform float uSquash;      // squash-and-stretch (1.0 = normal)
  uniform float uLean;        // body lean angle (-1 to 1)
  uniform float uBodyY;       // vertical offset for sitting etc.

  varying vec2 vUv;

  void main() {
    vUv = uv;

    vec4 worldPos = modelMatrix * vec4(0.0, 0.0, 0.0, 1.0);
    worldPos.y += uBounce + uBodyY;

    // Billboard
    vec3 camRight = vec3(viewMatrix[0][0], viewMatrix[1][0], viewMatrix[2][0]);
    vec3 camUp = vec3(viewMatrix[0][1], viewMatrix[1][1], viewMatrix[2][1]);

    float scale = 4.0;
    // Squash-and-stretch: wide when squashed, tall when stretched
    float sx = scale * (2.0 - uSquash);
    float sy = scale * uSquash;

    // Apply lean by rotating the local billboard axes
    float leanAngle = uLean * 0.15;
    vec3 leanRight = camRight * cos(leanAngle) + camUp * sin(leanAngle);
    vec3 leanUp = -camRight * sin(leanAngle) + camUp * cos(leanAngle);

    vec3 vert = worldPos.xyz
      + leanRight * position.x * sx
      + leanUp * position.y * sy;

    gl_Position = projectionMatrix * viewMatrix * vec4(vert, 1.0);
  }
`;

/** Gnome fragment shader — procedural with animated body parts */
const GNOME_FRAG = /* glsl */ `
  uniform float uTime;
  uniform float uArmL;        // left arm angle (0=down, 1=up)
  uniform float uArmR;        // right arm angle (0=down, 1=up)
  uniform float uHeadTilt;    // head rotation (-1=left, 1=right)
  uniform float uEyeState;    // 0=open, 1=closed (blink), 2=happy squint
  uniform float uMouthState;  // 0=neutral, 1=smile, 2=yawn open
  uniform float uToolType;    // 0=none, 1=shovel, 2=watering can, 3=seed bag
  uniform float uHatSway;     // hat tip sway (-1 to 1)
  uniform float uCheekGlow;   // rosy cheeks intensity (0-1)
  uniform float uSitting;     // 0=standing, 1=sitting (legs change)

  varying vec2 vUv;

  // SDF helpers
  float sdCircle(vec2 p, vec2 c, float r) {
    return length(p - c) - r;
  }
  float sdEllipse(vec2 p, vec2 c, vec2 r) {
    vec2 d = (p - c) / r;
    return (length(d) - 1.0) * min(r.x, r.y);
  }
  float sdBox(vec2 p, vec2 c, vec2 b) {
    vec2 d = abs(p - c) - b;
    return length(max(d, 0.0)) + min(max(d.x, d.y), 0.0);
  }
  // Rotation helper
  vec2 rot(vec2 p, float a) {
    float c = cos(a);
    float s = sin(a);
    return vec2(p.x * c - p.y * s, p.x * s + p.y * c);
  }

  void main() {
    vec2 p = vUv - 0.5;

    // --- Body ---
    float bodyCenterY = mix(-0.04, -0.10, uSitting);
    float body = sdEllipse(p, vec2(0.0, bodyCenterY), vec2(0.16, 0.20));
    float bodyAlpha = 1.0 - smoothstep(-0.01, 0.01, body);

    // --- Head (tilts with uHeadTilt) ---
    float headBaseY = mix(0.14, 0.08, uSitting);
    vec2 headCenter = vec2(uHeadTilt * 0.02, headBaseY);
    float head = sdCircle(p, headCenter, 0.10);
    float headAlpha = 1.0 - smoothstep(-0.01, 0.01, head);

    // --- Hat (sways with uHatSway, tilts with head) ---
    vec2 hatBase = headCenter + vec2(0.0, 0.08);
    vec2 hp = p - hatBase;
    hp = rot(hp, (uHatSway + uHeadTilt * 0.5) * 0.2);
    float hatX = abs(hp.x);
    float hatW = 0.13;
    float hatH = 0.26;
    float hatSdf = hatX - (hatW - hp.y * (hatW / hatH));
    float hatAlpha = step(hatSdf, 0.0) * step(0.0, hp.y) * step(hp.y, hatH);
    // Round tip
    float tipDist = sdCircle(hp, vec2(0.0, hatH - 0.03), 0.03);
    hatAlpha = max(hatAlpha, 1.0 - smoothstep(-0.01, 0.01, tipDist));

    // --- Arms (circles + sticks, positioned by uArmL/uArmR) ---
    // Left arm: pivots from body shoulder
    float armAngleL = mix(-0.6, 2.2, uArmL); // radians from down to up
    vec2 shoulderL = vec2(-0.14, bodyCenterY + 0.10);
    vec2 armDirL = vec2(cos(armAngleL), sin(armAngleL));
    vec2 handL = shoulderL + armDirL * 0.12;
    // Arm as capsule (two circles + rect approximation)
    float armL1 = sdCircle(p, shoulderL, 0.03);
    float armL2 = sdCircle(p, handL, 0.035);
    // Line segment distance
    vec2 armSegL = handL - shoulderL;
    float tL = clamp(dot(p - shoulderL, armSegL) / dot(armSegL, armSegL), 0.0, 1.0);
    vec2 closestL = shoulderL + tL * armSegL;
    float armLDist = length(p - closestL) - 0.025;
    float armLAlpha = 1.0 - smoothstep(-0.01, 0.01, min(min(armL1, armL2), armLDist));

    // Right arm
    float armAngleR = mix(-0.6, 2.2, uArmR);
    // Mirror: right side
    float rArmAngle = 3.14159 - armAngleR;
    vec2 shoulderR = vec2(0.14, bodyCenterY + 0.10);
    vec2 armDirR = vec2(cos(rArmAngle), sin(rArmAngle));
    vec2 handR = shoulderR + armDirR * 0.12;
    float armR1 = sdCircle(p, shoulderR, 0.03);
    float armR2 = sdCircle(p, handR, 0.035);
    vec2 armSegR = handR - shoulderR;
    float tR = clamp(dot(p - shoulderR, armSegR) / dot(armSegR, armSegR), 0.0, 1.0);
    vec2 closestR = shoulderR + tR * armSegR;
    float armRDist = length(p - closestR) - 0.025;
    float armRAlpha = 1.0 - smoothstep(-0.01, 0.01, min(min(armR1, armR2), armRDist));

    // --- Boots (change shape when sitting) ---
    float bootSpreadX = mix(0.08, 0.12, uSitting);
    float bootY = mix(-0.24, -0.20, uSitting);
    float bootStretchX = mix(0.07, 0.06, uSitting);
    float bootStretchY = mix(0.04, 0.05, uSitting);
    float bootL = sdEllipse(p, vec2(-bootSpreadX, bootY), vec2(bootStretchX, bootStretchY));
    float bootR = sdEllipse(p, vec2(bootSpreadX, bootY), vec2(bootStretchX, bootStretchY));
    float bootsAlpha = max(
      1.0 - smoothstep(-0.01, 0.01, bootL),
      1.0 - smoothstep(-0.01, 0.01, bootR)
    );

    // --- Belt ---
    float beltAlpha = bodyAlpha
      * step(bodyCenterY - 0.02, p.y) * step(p.y, bodyCenterY + 0.02)
      * (1.0 - step(0.14, abs(p.x)));
    float buckleAlpha = step(bodyCenterY - 0.01, p.y) * step(p.y, bodyCenterY + 0.01)
      * step(abs(p.x), 0.025);

    // --- Tool in hand (right hand) ---
    float toolAlpha = 0.0;
    vec3 toolColor = vec3(0.5);

    if (uToolType > 0.5) {
      vec2 toolBase = handR;
      if (uToolType < 1.5) {
        // Shovel: brown handle + gray blade
        vec2 sp = p - toolBase;
        sp = rot(sp, -0.3);
        float handle = sdBox(sp, vec2(0.0, 0.04), vec2(0.008, 0.05));
        float blade = sdEllipse(sp, vec2(0.0, -0.02), vec2(0.02, 0.025));
        float shovelDist = min(handle, blade);
        toolAlpha = 1.0 - smoothstep(-0.005, 0.005, shovelDist);
        toolColor = mix(vec3(0.45, 0.35, 0.2), vec3(0.5, 0.5, 0.55), step(0.0, sp.y - 0.01));
      } else if (uToolType < 2.5) {
        // Watering can: blue body
        float canBody = sdBox(p, toolBase + vec2(0.01, 0.0), vec2(0.025, 0.02));
        float spout = sdBox(p, toolBase + vec2(0.04, 0.015), vec2(0.015, 0.005));
        toolAlpha = 1.0 - smoothstep(-0.005, 0.005, min(canBody, spout));
        toolColor = vec3(0.3, 0.5, 0.7);
      } else {
        // Seed bag: brown pouch
        float bag = sdCircle(p, toolBase + vec2(0.0, -0.01), 0.025);
        toolAlpha = 1.0 - smoothstep(-0.005, 0.005, bag);
        toolColor = vec3(0.55, 0.40, 0.25);
      }
    }

    // --- Combine alpha ---
    float alpha = max(max(max(max(bodyAlpha, headAlpha), hatAlpha), bootsAlpha),
                      max(max(armLAlpha, armRAlpha), toolAlpha));
    if (alpha < 0.1) discard;

    // --- Colors ---
    vec3 tunicColor = vec3(0.32, 0.45, 0.25);
    vec3 hatColor = vec3(0.72, 0.18, 0.12);
    vec3 skinColor = vec3(0.85, 0.70, 0.55);
    vec3 bootColor = vec3(0.35, 0.22, 0.12);
    vec3 beltColorV = vec3(0.30, 0.20, 0.10);
    vec3 buckleColorV = vec3(0.85, 0.75, 0.35);
    vec3 beardColor = vec3(0.82, 0.78, 0.70);
    vec3 armColor = tunicColor * 0.9; // sleeves same as tunic but slightly darker
    vec3 handColor = skinColor;

    // Layer colors bottom-to-top
    vec3 color = tunicColor;

    // Boots
    color = mix(color, bootColor, bootsAlpha);
    // Arms (behind body in layering)
    color = mix(color, armColor, max(armLAlpha, armRAlpha) * 0.9);
    // Hands (tips of arms)
    float handLMask = 1.0 - smoothstep(0.01, 0.03, length(p - handL));
    float handRMask = 1.0 - smoothstep(0.01, 0.03, length(p - handR));
    color = mix(color, handColor, max(handLMask, handRMask) * max(armLAlpha, armRAlpha));
    // Body over arms
    color = mix(color, tunicColor, bodyAlpha * 0.5);
    // Tool
    color = mix(color, toolColor, toolAlpha);
    // Head (skin)
    color = mix(color, skinColor, headAlpha);

    // Rosy cheeks
    float cheekL = 1.0 - smoothstep(0.015, 0.03, length(p - (headCenter + vec2(-0.06, -0.01))));
    float cheekR = 1.0 - smoothstep(0.015, 0.03, length(p - (headCenter + vec2(0.06, -0.01))));
    vec3 rosyColor = vec3(0.9, 0.55, 0.5);
    color = mix(color, rosyColor, max(cheekL, cheekR) * uCheekGlow * headAlpha);

    // Beard
    float beardMask = headAlpha * step(p.y, headCenter.y - 0.02) * step(headCenter.y - 0.10, p.y);
    color = mix(color, beardColor, beardMask);

    // Eyes — animated blink + happy squint
    vec2 eyeLPos = headCenter + vec2(-0.04, 0.02);
    vec2 eyeRPos = headCenter + vec2(0.04, 0.02);
    float eyeOpenL = 1.0 - smoothstep(0.01, 0.02, length(p - eyeLPos));
    float eyeOpenR = 1.0 - smoothstep(0.01, 0.02, length(p - eyeRPos));
    // Blink: flatten eye vertically
    float blinkMask = 1.0 - uEyeState; // 1 when open, 0 when closed
    // Happy squint: eyes become curved lines (approximated as compressed ellipses)
    float happyMask = smoothstep(1.5, 2.0, uEyeState);
    float eyeH = mix(0.02, 0.005, min(uEyeState, 1.0)); // shrink when blinking
    float happyEyeL = 1.0 - smoothstep(0.005, 0.015, sdEllipse(p, eyeLPos, vec2(0.015, eyeH)));
    float happyEyeR = 1.0 - smoothstep(0.005, 0.015, sdEllipse(p, eyeRPos, vec2(0.015, eyeH)));
    float eyeMask = max(happyEyeL, happyEyeR);
    vec3 eyeColor = mix(vec3(0.15, 0.10, 0.08), vec3(0.15, 0.10, 0.08), happyMask);
    color = mix(color, eyeColor, eyeMask * headAlpha);

    // Mouth — animated expressions
    vec2 mouthPos = headCenter + vec2(0.0, -0.03);
    float mouthAlpha = 0.0;
    vec3 mouthColor = vec3(0.6, 0.35, 0.3);
    if (uMouthState < 0.5) {
      // Neutral: tiny line
      float mouthLine = sdBox(p, mouthPos, vec2(0.02, 0.003));
      mouthAlpha = (1.0 - smoothstep(-0.002, 0.002, mouthLine)) * headAlpha;
    } else if (uMouthState < 1.5) {
      // Smile: curved arc
      float smileDist = length(p - (mouthPos - vec2(0.0, 0.01)));
      float smileArc = abs(smileDist - 0.025);
      float smileMask = (1.0 - smoothstep(0.0, 0.005, smileArc))
        * step(p.y, mouthPos.y)
        * step(mouthPos.y - 0.02, p.y);
      mouthAlpha = smileMask * headAlpha;
    } else {
      // Yawn: open circle
      float yawnDist = sdCircle(p, mouthPos, 0.025);
      mouthAlpha = (1.0 - smoothstep(-0.005, 0.005, yawnDist)) * headAlpha;
      mouthColor = vec3(0.4, 0.2, 0.2);
    }
    color = mix(color, mouthColor, mouthAlpha);

    // Nose
    float nose = 1.0 - smoothstep(0.01, 0.02, length(p - (headCenter + vec2(0.0, -0.01))));
    color = mix(color, skinColor * 0.9, nose * headAlpha);

    // Hat over head
    color = mix(color, hatColor, hatAlpha);
    // Hat brim highlight
    float brimY = hatBase.y;
    float brimMask = hatAlpha * step(brimY, p.y) * step(p.y, brimY + 0.03);
    color = mix(color, hatColor * 0.8, brimMask);
    // Belt and buckle
    color = mix(color, beltColorV, beltAlpha);
    color = mix(color, buckleColorV, buckleAlpha * bodyAlpha);

    // Edge shading
    float shade = 1.0 - length(p) * 0.25;
    color *= shade;

    // Shadow disc
    float shadowY = mix(-0.28, -0.24, uSitting);
    float shadowDist = sdEllipse(p, vec2(0.0, shadowY), vec2(0.14, 0.03));
    float shadowAlpha = (1.0 - smoothstep(-0.01, 0.02, shadowDist)) * 0.3;

    float finalAlpha = max(alpha * 0.95, shadowAlpha);
    vec3 finalColor = mix(vec3(0.0), color, step(0.1, alpha));
    if (finalAlpha < 0.02) discard;

    gl_FragColor = vec4(finalColor, finalAlpha);
  }
`;

// ─── Emotion Particle Shader ────────────────────────────────────

const EMOTION_VERT = /* glsl */ `
  attribute float aAge;
  attribute float aMaxAge;
  attribute float aType;

  varying float vAge;
  varying float vType;

  void main() {
    vAge = aAge / aMaxAge;
    vType = aType;

    vec4 mvPos = modelViewMatrix * vec4(position, 1.0);
    // Fade out particles near end of life
    gl_PointSize = mix(12.0, 0.0, vAge * vAge) * (300.0 / -mvPos.z);
    gl_Position = projectionMatrix * mvPos;
  }
`;

const EMOTION_FRAG = /* glsl */ `
  varying float vAge;
  varying float vType;

  void main() {
    vec2 p = gl_PointCoord - 0.5;
    float dist = length(p);
    if (dist > 0.5) discard;

    float fadeOut = 1.0 - vAge;
    vec3 color;
    float alpha;

    if (vType < 0.5) {
      // Heart: red/pink
      // Heart shape approximation
      float heartBody = length(p * vec2(1.0, 1.3) + vec2(0.0, 0.1)) - 0.35;
      alpha = (1.0 - smoothstep(-0.05, 0.05, heartBody)) * fadeOut;
      color = vec3(0.9, 0.3, 0.4);
    } else if (vType < 1.5) {
      // Sparkle: gold star shape
      float angle = atan(p.y, p.x);
      float star = 0.3 + 0.15 * sin(angle * 5.0);
      alpha = (1.0 - smoothstep(star - 0.05, star, dist)) * fadeOut;
      color = vec3(1.0, 0.9, 0.4);
    } else if (vType < 2.5) {
      // Sweat: blue drop
      alpha = (1.0 - smoothstep(0.3, 0.5, dist)) * fadeOut;
      color = vec3(0.4, 0.6, 0.9);
    } else if (vType < 3.5) {
      // Zzz: white-blue
      alpha = (1.0 - smoothstep(0.3, 0.5, dist)) * fadeOut * 0.7;
      color = vec3(0.7, 0.8, 1.0);
    } else if (vType < 4.5) {
      // Music note: purple
      alpha = (1.0 - smoothstep(0.3, 0.5, dist)) * fadeOut;
      color = vec3(0.7, 0.4, 0.9);
    } else {
      // Exclaim: yellow
      alpha = (1.0 - smoothstep(0.3, 0.5, dist)) * fadeOut;
      color = vec3(1.0, 0.85, 0.3);
    }

    if (alpha < 0.02) discard;
    gl_FragColor = vec4(color, alpha);
  }
`;

// ─── Main Class ─────────────────────────────────────────────────

const MAX_EMOTIONS = 20;

export class GardenerSprite {
  readonly group: THREE.Group;
  private mesh: THREE.Mesh;
  private material: THREE.ShaderMaterial;
  private state = GnomeState.Idle;

  /** Current position in sim coordinates */
  x: number;
  y: number;
  z: number;

  // Task management
  private currentTask: GardenTask | null = null;
  private workTimer = 0;
  private tasksCompletedStreak = 0;

  // Idle personality
  private idleBehavior = IdleBehavior.Standing;
  private idleTimer = 0;         // countdown to next idle behavior
  private idleBehaviorTimer = 0; // duration of current behavior
  private idleElapsed = 0;       // time within current behavior

  // Celebration
  private celebType = CelebType.TaskDone;
  private celebTimer = 0;
  private celebElapsed = 0;

  // Animation state (smoothly interpolated)
  private armL = 0;       // 0=down, 1=up
  private armR = 0;
  private headTilt = 0;   // -1 to 1
  private eyeState = 0;   // 0=open, 1=closed, 2=happy
  private mouthState = 0; // 0=neutral, 1=smile, 2=yawn
  private hatSway = 0;
  private cheekGlow = 0;
  private bodyLean = 0;   // -1 to 1
  private bounce = 0;
  private squash = 1;
  private sitting = 0;
  private bodyY = 0;
  private toolType = 0;

  // Blink timing
  private blinkTimer = BLINK_INTERVAL;
  private blinking = false;
  private blinkElapsed = 0;

  // Emotion particles
  private emotionParticles: EmotionParticle[] = [];
  private emotionMesh: THREE.Points;
  private emotionPositions: Float32Array;
  private emotionAges: Float32Array;
  private emotionMaxAges: Float32Array;
  private emotionTypes: Float32Array;

  // Walk direction tracking for lean
  private walkDirX = 0;
  private walkDirY = 0;

  constructor() {
    this.group = new THREE.Group();
    this.group.name = 'gardener';

    this.x = GRID_X / 2;
    this.y = GRID_Y / 2;
    this.z = GROUND_LEVEL + 1;

    // Gnome billboard mesh
    const geo = new THREE.PlaneGeometry(1, 1);
    this.material = new THREE.ShaderMaterial({
      vertexShader: GNOME_VERT,
      fragmentShader: GNOME_FRAG,
      uniforms: {
        uTime: { value: 0 },
        uBounce: { value: 0 },
        uSquash: { value: 1 },
        uLean: { value: 0 },
        uBodyY: { value: 0 },
        uArmL: { value: 0 },
        uArmR: { value: 0 },
        uHeadTilt: { value: 0 },
        uEyeState: { value: 0 },
        uMouthState: { value: 0 },
        uToolType: { value: 0 },
        uHatSway: { value: 0 },
        uCheekGlow: { value: 0 },
        uSitting: { value: 0 },
      },
      transparent: true,
      depthWrite: false,
      side: THREE.DoubleSide,
    });

    this.mesh = new THREE.Mesh(geo, this.material);
    this.mesh.frustumCulled = false;
    this.group.add(this.mesh);

    // Emotion particle system
    this.emotionPositions = new Float32Array(MAX_EMOTIONS * 3);
    this.emotionAges = new Float32Array(MAX_EMOTIONS);
    this.emotionMaxAges = new Float32Array(MAX_EMOTIONS);
    this.emotionTypes = new Float32Array(MAX_EMOTIONS);

    const eGeo = new THREE.BufferGeometry();
    eGeo.setAttribute('position', new THREE.BufferAttribute(this.emotionPositions, 3));
    eGeo.setAttribute('aAge', new THREE.BufferAttribute(this.emotionAges, 1));
    eGeo.setAttribute('aMaxAge', new THREE.BufferAttribute(this.emotionMaxAges, 1));
    eGeo.setAttribute('aType', new THREE.BufferAttribute(this.emotionTypes, 1));

    const eMat = new THREE.ShaderMaterial({
      vertexShader: EMOTION_VERT,
      fragmentShader: EMOTION_FRAG,
      transparent: true,
      depthWrite: false,
    });

    this.emotionMesh = new THREE.Points(eGeo, eMat);
    this.emotionMesh.frustumCulled = false;
    this.group.add(this.emotionMesh);

    this.idleTimer = randomRange(IDLE_BEHAVIOR_MIN_WAIT, IDLE_BEHAVIOR_MAX_WAIT);
    this.updatePosition();
  }

  /** Update gnome each frame. Returns a completed task or null. */
  update(dt: number, elapsed: number, queue: TaskQueue): GardenTask | null {
    this.material.uniforms.uTime.value = elapsed;

    // Blink system (runs in all states)
    this.updateBlink(dt);

    // State machine
    let completedTask: GardenTask | null = null;

    switch (this.state) {
      case GnomeState.Idle:
        completedTask = this.updateIdle(dt, elapsed, queue);
        break;
      case GnomeState.Walking:
        this.updateWalking(dt);
        break;
      case GnomeState.Working:
        completedTask = this.updateWorking(dt, queue);
        break;
      case GnomeState.Celebrating:
        completedTask = this.updateCelebrating(dt, elapsed, queue);
        break;
    }

    // Smoothly interpolate animation uniforms
    this.updateAnimationUniforms(dt);

    // Emotion particles
    this.updateEmotions(dt);

    this.updatePosition();
    return completedTask;
  }

  // ─── State Updates ──────────────────────────────────────────

  private updateIdle(dt: number, _elapsed: number, queue: TaskQueue): GardenTask | null {
    // Check for new tasks
    if (queue.length > 0) {
      this.currentTask = queue.peek();
      this.state = GnomeState.Walking;
      this.idleBehavior = IdleBehavior.Standing;
      this.sitting = 0;
      this.mouthState = 0;
      // Determine tool type from task
      this.toolType = this.getToolType(this.currentTask);
      return null;
    }

    // Run idle behavior system
    if (this.idleBehavior === IdleBehavior.Standing) {
      // Gentle breathing animation
      this.squash = 1.0 + Math.sin(_elapsed * 1.5) * 0.02;
      this.hatSway = Math.sin(_elapsed * 0.7) * 0.15;

      // Countdown to next idle behavior
      this.idleTimer -= dt;
      if (this.idleTimer <= 0) {
        this.startRandomIdleBehavior();
      }
    } else {
      this.idleBehaviorTimer -= dt;
      this.idleElapsed += dt;

      this.animateIdleBehavior(_elapsed);

      if (this.idleBehaviorTimer <= 0) {
        // Return to standing
        this.idleBehavior = IdleBehavior.Standing;
        this.idleTimer = randomRange(IDLE_BEHAVIOR_MIN_WAIT, IDLE_BEHAVIOR_MAX_WAIT);
        this.sitting = 0;
        this.mouthState = 0;
        this.armL = 0;
        this.armR = 0;
        this.headTilt = 0;
        this.bodyLean = 0;
      }
    }

    return null;
  }

  private updateWalking(dt: number): void {
    if (!this.currentTask) {
      this.state = GnomeState.Idle;
      return;
    }

    const tx = this.currentTask.x;
    const ty = this.currentTask.y;
    const speed = WALK_SPEED * dt;
    const dx = tx - this.x;
    const dy = ty - this.y;
    const dist = Math.sqrt(dx * dx + dy * dy);

    if (dist < 1.0) {
      // Arrived
      this.x = tx;
      this.y = ty;
      this.state = GnomeState.Working;
      this.workTimer = WORK_DURATION;
      // Landing squash
      this.squash = 0.85;
      this.bounce = -0.05;
      return;
    }

    // Move toward target
    const ndx = dx / dist;
    const ndy = dy / dist;
    this.walkDirX = ndx;
    this.walkDirY = ndy;
    this.x += ndx * Math.min(speed, dist);
    this.y += ndy * Math.min(speed, dist);

    // Walking animation
    const walkPhase = this.x * 3.0 + this.y * 3.0; // phase based on position
    this.bounce = Math.abs(Math.sin(walkPhase * 2.0)) * 0.15;
    this.squash = 1.0 + Math.sin(walkPhase * 4.0) * 0.04;
    this.bodyLean = Math.sin(walkPhase * 2.0) * 0.3;
    this.hatSway = Math.sin(walkPhase * 2.5) * 0.3;

    // Arms swing opposite to walk
    const swing = Math.sin(walkPhase * 2.0);
    this.armL = 0.2 + swing * 0.15;
    this.armR = 0.2 - swing * 0.15;

    // Cheerful expression while walking
    this.mouthState = 1; // smile
    this.cheekGlow = 0.3;
  }

  private updateWorking(dt: number, queue: TaskQueue): GardenTask | null {
    this.workTimer -= dt;

    // Working animation: rhythmic motion based on tool
    const workPhase = (WORK_DURATION - this.workTimer) / WORK_DURATION;
    const pump = Math.sin(workPhase * Math.PI * 4); // two pumps during work

    if (this.toolType >= 0.5 && this.toolType < 1.5) {
      // Shovel: dig motion — arms pump up and down
      this.armR = 0.3 + pump * 0.4;
      this.armL = 0.1;
      this.bodyLean = pump * 0.2;
      this.bounce = pump * 0.05;
    } else if (this.toolType >= 1.5 && this.toolType < 2.5) {
      // Watering can: gentle tipping
      this.armR = 0.5 + Math.sin(workPhase * Math.PI * 2) * 0.2;
      this.armL = 0.2;
      this.bodyLean = Math.sin(workPhase * Math.PI * 2) * 0.15;
      this.bounce = 0;
    } else {
      // Seed bag / default: sprinkling motion
      this.armR = 0.4 + Math.sin(workPhase * Math.PI * 6) * 0.15;
      this.armL = 0.3;
      this.bodyLean = Math.sin(workPhase * Math.PI * 3) * 0.1;
      this.bounce = Math.abs(pump) * 0.03;
    }

    // Effort expression
    this.mouthState = 0; // focused neutral
    this.cheekGlow = 0.5;

    // Emit sweat if long streak
    if (this.tasksCompletedStreak > 8 && Math.random() < dt * 0.5) {
      this.emitEmotion(EmotionType.Sweat);
    }

    if (this.workTimer <= 0) {
      const completed = queue.dequeue();
      this.currentTask = null;
      this.tasksCompletedStreak++;

      // Celebrate!
      if (queue.length === 0) {
        // Queue empty — bigger celebration
        this.startCelebration(
          this.tasksCompletedStreak > 10 ? CelebType.BigMilestone : CelebType.QueueEmpty
        );
        this.tasksCompletedStreak = 0;
      } else if (this.tasksCompletedStreak % 5 === 0) {
        // Every 5th task — small celebration
        this.startCelebration(CelebType.TaskDone);
      } else {
        // Continue to next task
        this.currentTask = queue.peek();
        this.state = GnomeState.Walking;
        this.toolType = this.getToolType(this.currentTask);
      }

      // Emit sparkle on task complete
      if (completed) {
        const tool = completed.tool;
        if (tool === ToolCode.Seed) {
          this.emitEmotion(EmotionType.Heart);
        } else {
          this.emitEmotion(EmotionType.Sparkle);
        }
      }

      return completed;
    }

    return null;
  }

  private updateCelebrating(dt: number, elapsed: number, queue: TaskQueue): GardenTask | null {
    this.celebTimer -= dt;
    this.celebElapsed += dt;

    const t = this.celebElapsed;

    switch (this.celebType) {
      case CelebType.TaskDone: {
        // Quick hop
        this.bounce = Math.sin(t * Math.PI / CELEB_DURATION[CelebType.TaskDone]) * 0.3;
        this.squash = 1.0 + Math.sin(t * Math.PI * 2 / CELEB_DURATION[CelebType.TaskDone]) * 0.08;
        this.armL = 0.6;
        this.armR = 0.6;
        this.eyeState = 2; // happy squint
        this.mouthState = 1;
        break;
      }
      case CelebType.QueueEmpty: {
        // Happy dance: bounce + arm wave + spin lean
        const dancePhase = t * 5.0;
        this.bounce = Math.abs(Math.sin(dancePhase)) * 0.25;
        this.bodyLean = Math.sin(dancePhase * 0.7) * 0.5;
        this.armL = 0.5 + Math.sin(dancePhase + 1.0) * 0.4;
        this.armR = 0.5 + Math.sin(dancePhase) * 0.4;
        this.hatSway = Math.sin(dancePhase * 1.3) * 0.5;
        this.headTilt = Math.sin(dancePhase * 0.8) * 0.3;
        this.squash = 1.0 + Math.sin(dancePhase * 2) * 0.06;
        this.eyeState = 2;
        this.mouthState = 1;
        this.cheekGlow = 0.8;
        // Music notes
        if (Math.random() < dt * 3.0) {
          this.emitEmotion(EmotionType.Music);
        }
        break;
      }
      case CelebType.BigMilestone: {
        // Arms up triumphant + big bounce
        const phase = t * 4.0;
        this.bounce = Math.abs(Math.sin(phase)) * 0.35;
        this.armL = 0.8 + Math.sin(phase * 1.5) * 0.2;
        this.armR = 0.8 + Math.sin(phase * 1.5 + 0.5) * 0.2;
        this.bodyLean = Math.sin(phase * 0.5) * 0.4;
        this.hatSway = Math.sin(phase) * 0.6;
        this.squash = 1.0 + Math.sin(phase * 2) * 0.1;
        this.eyeState = 2;
        this.mouthState = 1;
        this.cheekGlow = 1.0;
        if (Math.random() < dt * 4.0) {
          this.emitEmotion(Math.random() < 0.5 ? EmotionType.Sparkle : EmotionType.Music);
        }
        break;
      }
    }

    if (this.celebTimer <= 0) {
      // Done celebrating
      if (queue.length > 0) {
        this.currentTask = queue.peek();
        this.state = GnomeState.Walking;
        this.toolType = this.getToolType(this.currentTask);
      } else {
        this.state = GnomeState.Idle;
        this.idleTimer = randomRange(1.0, 3.0); // short wait after celebration
      }
      this.resetAnimation();
    }

    return null;
  }

  // ─── Idle Behaviors ──────────────────────────────────────────

  private startRandomIdleBehavior(): void {
    const behaviors = [
      IdleBehavior.LookingAround,
      IdleBehavior.Yawning,
      IdleBehavior.SittingDown,
      IdleBehavior.InspectingPlant,
      IdleBehavior.WavingAtCamera,
      IdleBehavior.Stretching,
    ];
    this.idleBehavior = behaviors[Math.floor(Math.random() * behaviors.length)];
    this.idleBehaviorTimer = IDLE_BEHAVIOR_DURATION[this.idleBehavior];
    this.idleElapsed = 0;
  }

  private animateIdleBehavior(elapsed: number): void {
    const t = this.idleElapsed;
    const dur = IDLE_BEHAVIOR_DURATION[this.idleBehavior];
    const progress = dur > 0 ? t / dur : 0;

    switch (this.idleBehavior) {
      case IdleBehavior.LookingAround: {
        // Head turns left, pauses, turns right, pauses
        this.headTilt = Math.sin(t * 2.0) * 0.8;
        this.hatSway = this.headTilt * 0.5;
        this.eyeState = 0; // alert eyes
        // Hand on hip
        this.armL = 0.15;
        this.armR = 0.1;
        break;
      }
      case IdleBehavior.Yawning: {
        // Build up, big yawn, settle
        const yawnCurve = Math.sin(progress * Math.PI);
        this.mouthState = yawnCurve > 0.3 ? 2 : 0; // mouth open during yawn
        this.armL = yawnCurve * 0.6; // arms stretch up
        this.armR = yawnCurve * 0.6;
        this.squash = 1.0 + yawnCurve * 0.06; // body stretches
        this.bodyLean = Math.sin(t * 0.5) * 0.1;
        this.eyeState = yawnCurve > 0.5 ? 1 : 0; // eyes close during peak
        if (yawnCurve > 0.5 && Math.random() < 0.02) {
          this.emitEmotion(EmotionType.Zzz);
        }
        break;
      }
      case IdleBehavior.SittingDown: {
        // Transition to sitting, rest, then stand back up
        const sitPhase = progress < 0.15 ? progress / 0.15
          : progress > 0.85 ? 1.0 - (progress - 0.85) / 0.15
          : 1.0;
        this.sitting = sitPhase;
        this.bodyY = -sitPhase * 0.3;
        this.armL = 0.05;
        this.armR = 0.05;
        this.headTilt = Math.sin(t * 0.8) * 0.2;
        this.mouthState = 1; // content smile
        this.eyeState = sitPhase > 0.5 ? 0 : 0;
        this.cheekGlow = 0.4;
        if (sitPhase > 0.8 && Math.random() < 0.03) {
          this.emitEmotion(EmotionType.Zzz);
        }
        break;
      }
      case IdleBehavior.InspectingPlant: {
        // Lean forward, peer down, nod
        this.bodyLean = Math.sin(progress * Math.PI) * 0.4;
        this.headTilt = Math.sin(t * 3.0) * 0.15; // tiny nods
        this.armL = 0.2 + Math.sin(t * 2.0) * 0.05;
        this.armR = 0.15;
        this.squash = 1.0 - Math.sin(progress * Math.PI) * 0.03; // slight crouch
        this.bodyY = -Math.sin(progress * Math.PI) * 0.15;
        this.mouthState = 0;
        // Occasional "!" of interest
        if (progress > 0.4 && progress < 0.6 && Math.random() < 0.02) {
          this.emitEmotion(EmotionType.Exclaim);
        }
        break;
      }
      case IdleBehavior.WavingAtCamera: {
        // Wave right arm enthusiastically
        const wavePhase = t * 6.0;
        this.armR = 0.7 + Math.sin(wavePhase) * 0.25;
        this.armL = 0.1;
        this.bodyLean = -0.15; // lean toward camera
        this.headTilt = Math.sin(t * 2.0) * 0.15;
        this.mouthState = 1; // big smile
        this.eyeState = 2; // happy squint
        this.cheekGlow = 0.7;
        this.hatSway = Math.sin(wavePhase * 0.5) * 0.2;
        break;
      }
      case IdleBehavior.Stretching: {
        // Arms up, big stretch, settle
        const stretchCurve = Math.sin(progress * Math.PI);
        this.armL = stretchCurve * 0.9;
        this.armR = stretchCurve * 0.9;
        this.squash = 1.0 + stretchCurve * 0.08;
        this.bodyLean = Math.sin(t * 1.5) * 0.1;
        this.bounce = stretchCurve * 0.05;
        this.mouthState = stretchCurve > 0.5 ? 2 : 0; // yawn at peak
        this.eyeState = stretchCurve > 0.6 ? 1 : 0;
        break;
      }
      default:
        break;
    }
  }

  // ─── Celebration ─────────────────────────────────────────────

  private startCelebration(type: CelebType): void {
    this.state = GnomeState.Celebrating;
    this.celebType = type;
    this.celebTimer = CELEB_DURATION[type];
    this.celebElapsed = 0;
  }

  // ─── Blink System ───────────────────────────────────────────

  private updateBlink(dt: number): void {
    if (this.blinking) {
      this.blinkElapsed += dt;
      if (this.blinkElapsed >= BLINK_DURATION) {
        this.blinking = false;
        this.blinkElapsed = 0;
        // Don't reset eyeState if it's being controlled by a behavior
        if (this.state === GnomeState.Idle && this.idleBehavior === IdleBehavior.Standing) {
          this.eyeState = 0;
        }
      } else {
        // Only apply blink if not overridden by behavior
        if (this.eyeState < 1.5) {
          this.eyeState = 1;
        }
      }
    } else {
      this.blinkTimer -= dt;
      if (this.blinkTimer <= 0) {
        this.blinking = true;
        this.blinkElapsed = 0;
        this.blinkTimer = BLINK_INTERVAL + (Math.random() - 0.5) * 2.0;
      }
    }
  }

  // ─── Emotion Particles ──────────────────────────────────────

  private emitEmotion(type: EmotionType): void {
    if (this.emotionParticles.length >= MAX_EMOTIONS) return;

    // Emit above gnome's head in Three.js coords
    const baseX = this.x + 0.5;
    const baseY = this.z + 3.5; // above head
    const baseZ = this.y + 0.5;

    this.emotionParticles.push({
      type,
      x: baseX + (Math.random() - 0.5) * 0.5,
      y: baseY,
      z: baseZ + (Math.random() - 0.5) * 0.5,
      age: 0,
      maxAge: 1.0 + Math.random() * 0.5,
      vx: (Math.random() - 0.5) * 0.5,
      vy: 1.5 + Math.random() * 0.5,
    });
  }

  private updateEmotions(dt: number): void {
    // Update existing particles
    for (let i = this.emotionParticles.length - 1; i >= 0; i--) {
      const p = this.emotionParticles[i];
      p.age += dt;
      p.x += p.vx * dt;
      p.y += p.vy * dt;
      p.vy *= 0.98; // slow rise

      if (p.age >= p.maxAge) {
        this.emotionParticles.splice(i, 1);
      }
    }

    // Write to buffers
    const count = Math.min(this.emotionParticles.length, MAX_EMOTIONS);
    for (let i = 0; i < count; i++) {
      const p = this.emotionParticles[i];
      this.emotionPositions[i * 3] = p.x;
      this.emotionPositions[i * 3 + 1] = p.y;
      this.emotionPositions[i * 3 + 2] = p.z;
      this.emotionAges[i] = p.age;
      this.emotionMaxAges[i] = p.maxAge;
      this.emotionTypes[i] = p.type;
    }

    // Zero out unused slots
    for (let i = count; i < MAX_EMOTIONS; i++) {
      this.emotionPositions[i * 3 + 1] = -1000; // hide off-screen
      this.emotionAges[i] = 1;
      this.emotionMaxAges[i] = 1;
    }

    const geo = this.emotionMesh.geometry;
    (geo.getAttribute('position') as THREE.BufferAttribute).needsUpdate = true;
    (geo.getAttribute('aAge') as THREE.BufferAttribute).needsUpdate = true;
    (geo.getAttribute('aMaxAge') as THREE.BufferAttribute).needsUpdate = true;
    (geo.getAttribute('aType') as THREE.BufferAttribute).needsUpdate = true;
    geo.setDrawRange(0, count);
  }

  // ─── Animation Smoothing ───────────────────────────────────

  private updateAnimationUniforms(dt: number): void {
    const u = this.material.uniforms;
    const lerp = 1.0 - Math.pow(0.001, dt); // ~60fps smooth

    u.uBounce.value = smoothTo(u.uBounce.value, this.bounce, lerp);
    u.uSquash.value = smoothTo(u.uSquash.value, this.squash, lerp);
    u.uLean.value = smoothTo(u.uLean.value, this.bodyLean, lerp);
    u.uBodyY.value = smoothTo(u.uBodyY.value, this.bodyY, lerp);
    u.uArmL.value = smoothTo(u.uArmL.value, this.armL, lerp);
    u.uArmR.value = smoothTo(u.uArmR.value, this.armR, lerp);
    u.uHeadTilt.value = smoothTo(u.uHeadTilt.value, this.headTilt, lerp);
    u.uEyeState.value = smoothTo(u.uEyeState.value, this.eyeState, lerp * 3); // fast blink
    u.uMouthState.value = smoothTo(u.uMouthState.value, this.mouthState, lerp);
    u.uToolType.value = this.toolType; // instant switch
    u.uHatSway.value = smoothTo(u.uHatSway.value, this.hatSway, lerp);
    u.uCheekGlow.value = smoothTo(u.uCheekGlow.value, this.cheekGlow, lerp);
    u.uSitting.value = smoothTo(u.uSitting.value, this.sitting, lerp);
  }

  // ─── Helpers ────────────────────────────────────────────────

  private resetAnimation(): void {
    this.armL = 0;
    this.armR = 0;
    this.headTilt = 0;
    this.mouthState = 0;
    this.cheekGlow = 0;
    this.bodyLean = 0;
    this.bounce = 0;
    this.squash = 1;
    this.sitting = 0;
    this.bodyY = 0;
    this.hatSway = 0;
    this.toolType = 0;
  }

  private getToolType(task: GardenTask | null): number {
    if (!task) return 0;
    switch (task.tool) {
      case ToolCode.Shovel: return 1;
      case ToolCode.Water: return 2;
      case ToolCode.Seed: return 3;
      case ToolCode.Soil: return 1; // uses shovel visual
      case ToolCode.Stone: return 1;
      default: return 0;
    }
  }

  private updatePosition(): void {
    // Sim (x,y,z) Z-up → Three.js (x,z,y) Y-up
    this.mesh.position.set(this.x + 0.5, this.z + 1.5, this.y + 0.5);
  }

  dispose(): void {
    this.mesh.geometry.dispose();
    this.material.dispose();
    this.emotionMesh.geometry.dispose();
    (this.emotionMesh.material as THREE.Material).dispose();
  }
}

// ─── Utilities ───────────────────────────────────────────────

function randomRange(min: number, max: number): number {
  return min + Math.random() * (max - min);
}

function smoothTo(current: number, target: number, factor: number): number {
  return current + (target - current) * Math.min(factor, 1.0);
}
