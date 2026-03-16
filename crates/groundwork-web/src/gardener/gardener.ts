/**
 * Garden gnome — charming 3D model with personality.
 *
 * Sim-driven: the Rust sim is the single authority for gnome position,
 * state, and task execution. This class handles visual rendering only:
 * smooth position interpolation, walk/work/idle animations, emotion
 * particles, and celebrations.
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
import type { GnomeView } from '../bridge';
import { buildGnomeModel, type GnomeParts } from '../models/gnome';

// ─── Sim State Constants (must match gnome.rs GnomeState repr) ────

const SimState = {
  Idle: 0,
  Walking: 1,
  Working: 2,
  Eating: 3,
  Resting: 4,
  Wandering: 5,
  Inspecting: 6,
} as const;

// ─── Animation State (JS-only, for visual rendering) ─────────────

enum AnimState {
  Idle,
  Walking,
  Working,
  Celebrating,
  Wandering,
  Eating,
  Resting,
}

enum IdleBehavior {
  Standing,
  LookingAround,
  Yawning,
  SittingDown,
  InspectingPlant,
  WavingAtCamera,
  Stretching,
}

enum CelebType {
  TaskDone,
  QueueEmpty,
  BigMilestone,
}

// ─── Animation Timing ────────────────────────────────────────────

const IDLE_BEHAVIOR_MIN_WAIT = 2.0;
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

const BLINK_INTERVAL = 3.0;
const BLINK_DURATION = 0.15;

/** Base scale for the gnome model — oversized so it's visible at default zoom
 *  among trees and foliage. At 1.8x the gnome is ~8 units tall. */
const GNOME_BASE_SCALE = 1.8;

/** Position lerp rate — how quickly visual catches up to sim position.
 *  Higher = snappier, lower = floatier. Tuned for 60fps rendering
 *  with ~10 ticks/sec sim rate. */
const POSITION_LERP_SPEED = 12.0;

// ─── Emotion Particles ──────────────────────────────────────────

enum EmotionType {
  Heart,
  Sparkle,
  Sweat,
  Zzz,
  Music,
  Exclaim,
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

// Emotion particle shader (point sprites)
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
      float heartBody = length(p * vec2(1.0, 1.3) + vec2(0.0, 0.1)) - 0.35;
      alpha = (1.0 - smoothstep(-0.05, 0.05, heartBody)) * fadeOut;
      color = vec3(0.9, 0.3, 0.4);
    } else if (vType < 1.5) {
      float angle = atan(p.y, p.x);
      float star = 0.3 + 0.15 * sin(angle * 5.0);
      alpha = (1.0 - smoothstep(star - 0.05, star, dist)) * fadeOut;
      color = vec3(1.0, 0.9, 0.4);
    } else if (vType < 2.5) {
      alpha = (1.0 - smoothstep(0.3, 0.5, dist)) * fadeOut;
      color = vec3(0.4, 0.6, 0.9);
    } else if (vType < 3.5) {
      alpha = (1.0 - smoothstep(0.3, 0.5, dist)) * fadeOut * 0.7;
      color = vec3(0.7, 0.8, 1.0);
    } else if (vType < 4.5) {
      alpha = (1.0 - smoothstep(0.3, 0.5, dist)) * fadeOut;
      color = vec3(0.7, 0.4, 0.9);
    } else {
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
  private parts: GnomeParts;
  private glowDisc: THREE.Mesh;
  private animState = AnimState.Idle;

  /** Current visual position (smoothly interpolated toward sim position) */
  private visualX: number;
  private visualY: number;
  private visualZ: number;

  /** Last known sim position (updated each syncFromSim call) */
  private simX: number;
  private simY: number;
  private simZ: number;

  /** Previous sim queue length — used to detect task completions */
  private prevSimQueueLen = 0;

  /** Tasks completed streak (for celebration intensity) */
  private tasksCompletedStreak = 0;

  // Idle personality
  private idleBehavior = IdleBehavior.Standing;
  private idleTimer = 0;
  private idleBehaviorTimer = 0;
  private idleElapsed = 0;

  // Celebration
  private celebType = CelebType.TaskDone;
  private celebTimer = 0;
  private celebElapsed = 0;

  // Animation targets (smoothly interpolated)
  private armLAngle = 0;
  private armRAngle = 0;
  private headTiltY = 0;
  private headTiltZ = 0;
  private hatSway = 0;
  private bodyLean = 0;
  private bounce = 0;
  private squashY = 1;
  private bodyY = 0;
  private bootSpread = 0;
  private toolType = 0;
  private cheekGlow = 0;

  // Current smooth values
  private sArmL = 0;
  private sArmR = 0;
  private sHeadY = 0;
  private sHeadZ = 0;
  private sHat = 0;
  private sLean = 0;
  private sBounce = 0;
  private sSquash = 1;
  private sBodyY = 0;
  private sBootSpread = 0;

  // Blink timing
  private blinkTimer = BLINK_INTERVAL;
  private blinking = false;
  private blinkElapsed = 0;
  private eyesClosed = false;

  // Emotion particles
  private emotionParticles: EmotionParticle[] = [];
  private emotionMesh: THREE.Points;
  private emotionPositions: Float32Array;
  private emotionAges: Float32Array;
  private emotionMaxAges: Float32Array;
  private emotionTypes: Float32Array;

  // Walk direction (derived from position delta for facing)
  private facingAngle = 0;

  constructor() {
    this.group = new THREE.Group();
    this.group.name = 'gardener';

    const cx = GRID_X / 2;
    const cy = GRID_Y / 2;
    const cz = GROUND_LEVEL + 1;
    this.visualX = cx;
    this.visualY = cy;
    this.visualZ = cz;
    this.simX = cx;
    this.simY = cy;
    this.simZ = cz;

    // Build 3D gnome model
    this.parts = buildGnomeModel();
    this.group.add(this.parts.root);

    // Warm glow disc on the ground beneath the gnome
    const glowGeo = new THREE.CircleGeometry(3.5, 24);
    const glowMat = new THREE.MeshBasicMaterial({
      color: 0xffcc66,
      transparent: true,
      opacity: 0.25,
      depthWrite: false,
      blending: THREE.AdditiveBlending,
    });
    this.glowDisc = new THREE.Mesh(glowGeo, glowMat);
    this.glowDisc.rotation.x = -Math.PI / 2;
    this.group.add(this.glowDisc);

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

  /** Sync visual state from sim gnome. Call once per render frame.
   *  Returns true if a task was completed this frame (for particles/remesh). */
  syncFromSim(sim: GnomeView, dt: number, elapsed: number): boolean {
    // --- Position interpolation ---
    this.simX = sim.x;
    this.simY = sim.y;
    this.simZ = sim.z;

    const lerpFactor = 1.0 - Math.exp(-POSITION_LERP_SPEED * dt);
    this.visualX += (this.simX - this.visualX) * lerpFactor;
    this.visualY += (this.simY - this.visualY) * lerpFactor;
    this.visualZ += (this.simZ - this.visualZ) * lerpFactor;

    // --- Facing direction from movement delta ---
    const dx = this.simX - this.visualX;
    const dy = this.simY - this.visualY;
    const moveDist = Math.sqrt(dx * dx + dy * dy);
    if (moveDist > 0.05) {
      this.facingAngle = Math.atan2(dy, dx);
    }

    // --- Detect task completion (sim queue length decreased) ---
    let taskCompleted = false;
    if (sim.queueLen < this.prevSimQueueLen) {
      taskCompleted = true;
      this.tasksCompletedStreak++;
      // Trigger celebration
      if (sim.queueLen === 0) {
        this.startCelebration(
          this.tasksCompletedStreak > 10 ? CelebType.BigMilestone : CelebType.QueueEmpty
        );
        this.tasksCompletedStreak = 0;
      } else if (this.tasksCompletedStreak % 5 === 0) {
        this.startCelebration(CelebType.TaskDone);
      }
      // Emit emotion
      if (sim.activeTool === ToolCode.Seed) {
        this.emitEmotion(EmotionType.Heart);
      } else {
        this.emitEmotion(EmotionType.Sparkle);
      }
    }
    this.prevSimQueueLen = sim.queueLen;

    // --- Map sim state → animation state ---
    // Celebrations override sim state until they finish
    if (this.animState !== AnimState.Celebrating) {
      this.mapSimStateToAnim(sim);
    }

    // --- Tool type for work animation ---
    this.toolType = this.getToolTypeFromCode(sim.activeTool);

    // --- Fauna reactions ---
    this.reactToFauna(sim.nearbyFauna, sim.squirrelTrust, dt);

    // --- Blink ---
    this.updateBlink(dt);

    // --- Run current animation ---
    switch (this.animState) {
      case AnimState.Idle:
        this.animateIdle(dt, elapsed);
        break;
      case AnimState.Walking:
        this.animateWalking();
        break;
      case AnimState.Working:
        this.animateWorking(dt);
        break;
      case AnimState.Celebrating:
        this.animateCelebrating(dt, elapsed);
        break;
      case AnimState.Wandering:
        this.animateWandering();
        break;
      case AnimState.Eating:
        this.animateEating(dt, elapsed);
        break;
      case AnimState.Resting:
        this.animateResting(dt, elapsed);
        break;
    }

    // --- Smooth interpolation + apply to 3D model ---
    this.applyAnimation(dt);

    // --- Emotion particles ---
    this.updateEmotions(dt);

    // --- Position the Three.js group ---
    this.updatePosition();

    return taskCompleted;
  }

  // ─── State Mapping ──────────────────────────────────────────

  private mapSimStateToAnim(sim: GnomeView): void {
    switch (sim.state) {
      case SimState.Idle:
        if (this.animState !== AnimState.Idle) {
          this.animState = AnimState.Idle;
          this.idleBehavior = IdleBehavior.Standing;
          this.idleTimer = randomRange(IDLE_BEHAVIOR_MIN_WAIT, IDLE_BEHAVIOR_MAX_WAIT);
          this.resetAnimation();
        }
        break;
      case SimState.Walking:
        this.animState = AnimState.Walking;
        break;
      case SimState.Working:
        this.animState = AnimState.Working;
        break;
      case SimState.Eating:
        this.animState = AnimState.Eating;
        break;
      case SimState.Resting:
        this.animState = AnimState.Resting;
        break;
      case SimState.Wandering:
        this.animState = AnimState.Wandering;
        break;
      case SimState.Inspecting:
        // Inspecting looks like an idle inspect behavior
        if (this.animState !== AnimState.Idle || this.idleBehavior !== IdleBehavior.InspectingPlant) {
          this.animState = AnimState.Idle;
          this.idleBehavior = IdleBehavior.InspectingPlant;
          this.idleBehaviorTimer = 3.0;
          this.idleElapsed = 0;
        }
        break;
    }
  }

  // ─── Animation Functions ───────────────────────────────────

  private animateIdle(dt: number, elapsed: number): void {
    if (this.idleBehavior === IdleBehavior.Standing) {
      // Gentle breathing
      this.squashY = 1.0 + Math.sin(elapsed * 1.5) * 0.02;
      this.hatSway = Math.sin(elapsed * 0.7) * 0.1;

      this.idleTimer -= dt;
      if (this.idleTimer <= 0) {
        this.startRandomIdleBehavior();
      }
    } else {
      this.idleBehaviorTimer -= dt;
      this.idleElapsed += dt;
      this.animateIdleBehavior(elapsed);

      if (this.idleBehaviorTimer <= 0) {
        this.idleBehavior = IdleBehavior.Standing;
        this.idleTimer = randomRange(IDLE_BEHAVIOR_MIN_WAIT, IDLE_BEHAVIOR_MAX_WAIT);
        this.resetAnimation();
      }
    }
  }

  private animateWalking(): void {
    const walkPhase = this.visualX * 3.0 + this.visualY * 3.0;
    this.bounce = Math.abs(Math.sin(walkPhase * 2.0)) * 0.3;
    this.squashY = 1.0 + Math.sin(walkPhase * 4.0) * 0.04;
    this.bodyLean = Math.sin(walkPhase * 2.0) * 0.15;
    this.hatSway = Math.sin(walkPhase * 2.5) * 0.2;

    const swing = Math.sin(walkPhase * 2.0);
    this.armLAngle = 0.3 + swing * 0.4;
    this.armRAngle = 0.3 - swing * 0.4;
    this.cheekGlow = 0.3;
  }

  private animateWorking(dt: number): void {
    const t = (Date.now() % 1000) / 1000; // continuous work phase
    const pump = Math.sin(t * Math.PI * 4);

    if (this.toolType >= 0.5 && this.toolType < 1.5) {
      // Shovel: dig motion
      this.armRAngle = 0.5 + pump * 0.6;
      this.armLAngle = 0.2;
      this.bodyLean = pump * 0.1;
      this.bounce = pump * 0.1;
    } else if (this.toolType >= 1.5 && this.toolType < 2.5) {
      // Watering can: gentle tipping
      this.armRAngle = 0.8 + Math.sin(t * Math.PI * 2) * 0.3;
      this.armLAngle = 0.3;
      this.bodyLean = Math.sin(t * Math.PI * 2) * 0.08;
    } else {
      // Seed bag: sprinkling
      this.armRAngle = 0.6 + Math.sin(t * Math.PI * 6) * 0.2;
      this.armLAngle = 0.4;
      this.bodyLean = Math.sin(t * Math.PI * 3) * 0.05;
      this.bounce = Math.abs(pump) * 0.06;
    }

    this.cheekGlow = 0.5;

    if (this.tasksCompletedStreak > 8 && Math.random() < dt * 0.5) {
      this.emitEmotion(EmotionType.Sweat);
    }
  }

  private animateWandering(): void {
    const walkPhase = this.visualX * 2.0 + this.visualY * 2.0;
    this.bounce = Math.abs(Math.sin(walkPhase * 2.0)) * 0.15;
    this.squashY = 1.0 + Math.sin(walkPhase * 3.0) * 0.02;
    this.bodyLean = Math.sin(walkPhase * 1.5) * 0.08;
    this.hatSway = Math.sin(walkPhase * 2.0) * 0.15;
    const swing = Math.sin(walkPhase * 1.5);
    this.armLAngle = 0.2 + swing * 0.2;
    this.armRAngle = 0.2 - swing * 0.2;
    this.cheekGlow = 0.2;
  }

  private animateEating(dt: number, elapsed: number): void {
    // Gnome sits and eats — similar to SittingDown idle but with arm motion
    this.bodyY = -0.5;
    this.bootSpread = 0.25;
    this.armLAngle = 0.6 + Math.sin(elapsed * 4.0) * 0.2; // munching motion
    this.armRAngle = 0.3;
    this.headTiltY = Math.sin(elapsed * 1.2) * 0.1;
    this.cheekGlow = 0.6;

    if (Math.random() < dt * 0.3) {
      this.emitEmotion(EmotionType.Heart);
    }
  }

  private animateResting(dt: number, elapsed: number): void {
    // Gnome sits and rests — drowsy, gentle breathing
    this.bodyY = -0.6;
    this.bootSpread = 0.2;
    this.armLAngle = 0.05;
    this.armRAngle = 0.05;
    this.headTiltZ = Math.sin(elapsed * 0.3) * 0.15; // head drooping
    this.squashY = 1.0 + Math.sin(elapsed * 1.0) * 0.03;
    this.cheekGlow = 0.3;

    if (Math.random() < dt * 0.4) {
      this.emitEmotion(EmotionType.Zzz);
    }
  }

  private animateCelebrating(dt: number, _elapsed: number): void {
    this.celebTimer -= dt;
    this.celebElapsed += dt;
    const t = this.celebElapsed;

    switch (this.celebType) {
      case CelebType.TaskDone: {
        this.bounce = Math.sin(t * Math.PI / CELEB_DURATION[CelebType.TaskDone]) * 0.5;
        this.squashY = 1.0 + Math.sin(t * Math.PI * 2 / CELEB_DURATION[CelebType.TaskDone]) * 0.08;
        this.armLAngle = 1.2;
        this.armRAngle = 1.2;
        break;
      }
      case CelebType.QueueEmpty: {
        const dp = t * 5.0;
        this.bounce = Math.abs(Math.sin(dp)) * 0.4;
        this.bodyLean = Math.sin(dp * 0.7) * 0.2;
        this.armLAngle = 1.0 + Math.sin(dp + 1.0) * 0.6;
        this.armRAngle = 1.0 + Math.sin(dp) * 0.6;
        this.hatSway = Math.sin(dp * 1.3) * 0.3;
        this.headTiltY = Math.sin(dp * 0.8) * 0.3;
        this.squashY = 1.0 + Math.sin(dp * 2) * 0.06;
        this.cheekGlow = 0.8;
        if (Math.random() < dt * 3.0) this.emitEmotion(EmotionType.Music);
        break;
      }
      case CelebType.BigMilestone: {
        const phase = t * 4.0;
        this.bounce = Math.abs(Math.sin(phase)) * 0.6;
        this.armLAngle = 1.5 + Math.sin(phase * 1.5) * 0.3;
        this.armRAngle = 1.5 + Math.sin(phase * 1.5 + 0.5) * 0.3;
        this.bodyLean = Math.sin(phase * 0.5) * 0.15;
        this.hatSway = Math.sin(phase) * 0.4;
        this.squashY = 1.0 + Math.sin(phase * 2) * 0.1;
        this.cheekGlow = 1.0;
        if (Math.random() < dt * 4.0) {
          this.emitEmotion(Math.random() < 0.5 ? EmotionType.Sparkle : EmotionType.Music);
        }
        break;
      }
    }

    if (this.celebTimer <= 0) {
      this.animState = AnimState.Idle;
      this.idleTimer = randomRange(1.0, 3.0);
      this.resetAnimation();
    }
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

  private animateIdleBehavior(_elapsed: number): void {
    const t = this.idleElapsed;
    const dur = IDLE_BEHAVIOR_DURATION[this.idleBehavior];
    const progress = dur > 0 ? t / dur : 0;

    switch (this.idleBehavior) {
      case IdleBehavior.LookingAround: {
        this.headTiltY = Math.sin(t * 2.0) * 0.6;
        this.hatSway = this.headTiltY * 0.3;
        this.armLAngle = 0.15;
        this.armRAngle = 0.1;
        break;
      }
      case IdleBehavior.Yawning: {
        const yawnCurve = Math.sin(progress * Math.PI);
        this.armLAngle = yawnCurve * 1.2;
        this.armRAngle = yawnCurve * 1.2;
        this.squashY = 1.0 + yawnCurve * 0.06;
        this.bodyLean = Math.sin(t * 0.5) * 0.05;
        if (yawnCurve > 0.5 && Math.random() < 0.02) {
          this.emitEmotion(EmotionType.Zzz);
        }
        break;
      }
      case IdleBehavior.SittingDown: {
        const sitPhase = progress < 0.15 ? progress / 0.15
          : progress > 0.85 ? 1.0 - (progress - 0.85) / 0.15
          : 1.0;
        this.bodyY = -sitPhase * 0.6;
        this.bootSpread = sitPhase * 0.3;
        this.armLAngle = 0.05;
        this.armRAngle = 0.05;
        this.headTiltY = Math.sin(t * 0.8) * 0.15;
        this.cheekGlow = 0.4;
        if (sitPhase > 0.8 && Math.random() < 0.03) {
          this.emitEmotion(EmotionType.Zzz);
        }
        break;
      }
      case IdleBehavior.InspectingPlant: {
        this.bodyLean = Math.sin(progress * Math.PI) * 0.15;
        this.headTiltZ = Math.sin(progress * Math.PI) * -0.3;
        this.headTiltY = Math.sin(t * 3.0) * 0.1;
        this.armLAngle = 0.3 + Math.sin(t * 2.0) * 0.1;
        this.armRAngle = 0.2;
        this.bodyY = -Math.sin(progress * Math.PI) * 0.3;
        if (progress > 0.4 && progress < 0.6 && Math.random() < 0.02) {
          this.emitEmotion(EmotionType.Exclaim);
        }
        break;
      }
      case IdleBehavior.WavingAtCamera: {
        const wavePhase = t * 6.0;
        this.armRAngle = 1.5 + Math.sin(wavePhase) * 0.4;
        this.armLAngle = 0.1;
        this.bodyLean = -0.05;
        this.headTiltY = Math.sin(t * 2.0) * 0.1;
        this.cheekGlow = 0.7;
        this.hatSway = Math.sin(wavePhase * 0.5) * 0.15;
        break;
      }
      case IdleBehavior.Stretching: {
        const stretchCurve = Math.sin(progress * Math.PI);
        this.armLAngle = stretchCurve * 2.0;
        this.armRAngle = stretchCurve * 2.0;
        this.squashY = 1.0 + stretchCurve * 0.08;
        this.bodyLean = Math.sin(t * 1.5) * 0.04;
        this.bounce = stretchCurve * 0.1;
        break;
      }
      default:
        break;
    }
  }

  // ─── Celebration ─────────────────────────────────────────────

  private startCelebration(type: CelebType): void {
    this.animState = AnimState.Celebrating;
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
        this.eyesClosed = false;
      } else {
        this.eyesClosed = true;
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

  // ─── Apply Animation to 3D Model ─────────────────────────────

  private applyAnimation(dt: number): void {
    const lerp = 1.0 - Math.pow(0.001, dt);
    const p = this.parts;

    // Smooth interpolation
    this.sArmL = smoothTo(this.sArmL, this.armLAngle, lerp);
    this.sArmR = smoothTo(this.sArmR, this.armRAngle, lerp);
    this.sHeadY = smoothTo(this.sHeadY, this.headTiltY, lerp);
    this.sHeadZ = smoothTo(this.sHeadZ, this.headTiltZ, lerp);
    this.sHat = smoothTo(this.sHat, this.hatSway, lerp);
    this.sLean = smoothTo(this.sLean, this.bodyLean, lerp);
    this.sBounce = smoothTo(this.sBounce, this.bounce, lerp);
    this.sSquash = smoothTo(this.sSquash, this.squashY, lerp);
    this.sBodyY = smoothTo(this.sBodyY, this.bodyY, lerp);
    this.sBootSpread = smoothTo(this.sBootSpread, this.bootSpread, lerp);

    // Body: bounce + squash + lean (with base scale for visibility)
    p.root.position.y = this.sBounce + this.sBodyY;
    const sx = (2.0 - this.sSquash) * GNOME_BASE_SCALE;
    const sy = this.sSquash * GNOME_BASE_SCALE;
    const sz = GNOME_BASE_SCALE;
    p.root.scale.set(sx, sy, sz);
    p.root.rotation.z = this.sLean;

    // Face direction of movement (rotate entire gnome)
    if (this.animState === AnimState.Walking || this.animState === AnimState.Wandering) {
      const targetAngle = -this.facingAngle + Math.PI / 2;
      p.root.rotation.y = smoothTo(p.root.rotation.y, targetAngle, lerp);
    }

    // Head: tilt
    p.head.rotation.y = this.sHeadY;
    p.head.rotation.z = this.sHeadZ;

    // Hat: sway
    p.hat.rotation.z = this.sHat;

    // Arms: rotation around X axis
    p.armL.rotation.x = -this.sArmL;
    p.armR.rotation.x = -this.sArmR;

    // Boots: spread when sitting
    p.bootL.position.x = -0.3 - this.sBootSpread;
    p.bootR.position.x = 0.3 + this.sBootSpread;

    // Eye blink
    const eyeScale = this.eyesClosed ? 0.2 : 1.0;
    p.eyeL.scale.y = eyeScale;
    p.eyeR.scale.y = eyeScale;

    // Cheek glow
    p.cheekL.visible = this.cheekGlow > 0.2;
    p.cheekR.visible = this.cheekGlow > 0.2;

    // Tool visibility
    p.shovel.visible = this.toolType >= 0.5 && this.toolType < 1.5;
    p.waterCan.visible = this.toolType >= 1.5 && this.toolType < 2.5;
    p.seedBag.visible = this.toolType >= 2.5;
  }

  // ─── Emotion Particles ──────────────────────────────────────

  private emitEmotion(type: EmotionType): void {
    if (this.emotionParticles.length >= MAX_EMOTIONS) return;

    const baseX = this.visualX + 0.5;
    const baseY = this.visualZ + 3.5;
    const baseZ = this.visualY + 0.5;

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
    for (let i = this.emotionParticles.length - 1; i >= 0; i--) {
      const p = this.emotionParticles[i];
      p.age += dt;
      p.x += p.vx * dt;
      p.y += p.vy * dt;
      p.vy *= 0.98;
      if (p.age >= p.maxAge) {
        this.emotionParticles.splice(i, 1);
      }
    }

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

    for (let i = count; i < MAX_EMOTIONS; i++) {
      this.emotionPositions[i * 3 + 1] = -1000;
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

  // ─── Helpers ────────────────────────────────────────────────

  private resetAnimation(): void {
    this.armLAngle = 0;
    this.armRAngle = 0;
    this.headTiltY = 0;
    this.headTiltZ = 0;
    this.hatSway = 0;
    this.bodyLean = 0;
    this.bounce = 0;
    this.squashY = 1;
    this.bodyY = 0;
    this.bootSpread = 0;
    this.toolType = 0;
    this.cheekGlow = 0;
  }

  private getToolTypeFromCode(toolCode: number): number {
    switch (toolCode) {
      case 0: return 1;  // shovel
      case 2: return 2;  // water
      case 1: return 3;  // seed
      case 3: return 1;  // soil (uses shovel animation)
      case 4: return 1;  // stone (uses shovel animation)
      default: return 0; // no tool
    }
  }

  private updatePosition(): void {
    // Sim (x,y,z) Z-up → Three.js (x,z,y) Y-up
    this.parts.root.position.x = this.visualX + 0.5;
    this.parts.root.position.z = this.visualY + 0.5;
    const baseY = this.visualZ + 1.5;
    this.parts.root.position.y = baseY + this.sBounce + this.sBodyY;

    // Glow disc stays flat on the ground at the gnome's feet
    this.glowDisc.position.set(this.visualX + 0.5, this.visualZ + 0.15, this.visualY + 0.5);
  }

  /** React to nearby fauna from sim state. */
  private reactToFauna(nearbyFauna: number, squirrelTrust: number, dt: number): void {
    if (nearbyFauna === 0) return;

    if (Math.random() < dt * 0.3 * nearbyFauna) {
      this.emitEmotion(EmotionType.Heart);
    }

    if (squirrelTrust > 0 && squirrelTrust % 50 < 2 && Math.random() < dt * 0.5) {
      this.emitEmotion(EmotionType.Exclaim);
    }

    if (squirrelTrust >= 180 && Math.random() < dt * 0.4) {
      this.emitEmotion(EmotionType.Sparkle);
    }
  }

  dispose(): void {
    this.parts.root.traverse((obj) => {
      if (obj instanceof THREE.Mesh) {
        obj.geometry.dispose();
        if (obj.material instanceof THREE.Material) obj.material.dispose();
      }
    });
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
