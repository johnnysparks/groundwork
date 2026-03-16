/**
 * Garden gnome — charming 3D model with personality.
 *
 * A toylike gnome figurine built from soft primitives (spheres, cones,
 * cylinders) that walks to task locations and executes garden work.
 * Feels like a hand-painted collectible placed in the diorama.
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
import { buildGnomeModel, type GnomeParts } from '../models/gnome';

// ─── State Machine ───────────────────────────────────────────────

enum GnomeState {
  Idle,
  Walking,
  Working,
  Celebrating,
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

const WALK_SPEED = 8.0;
const WORK_DURATION = 0.4;
const BLINK_INTERVAL = 3.0;
const BLINK_DURATION = 0.15;

/** Base scale for the gnome model — oversized so it's visible at default zoom
 *  among trees and foliage. At 1.8x the gnome is ~8 units tall. */
const GNOME_BASE_SCALE = 1.8;

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

// Emotion particle shader (kept from previous implementation — works great as point sprites)
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
  private idleTimer = 0;
  private idleBehaviorTimer = 0;
  private idleElapsed = 0;

  // Celebration
  private celebType = CelebType.TaskDone;
  private celebTimer = 0;
  private celebElapsed = 0;

  // Animation targets (smoothly interpolated)
  private armLAngle = 0;    // arm rotation in radians
  private armRAngle = 0;
  private headTiltY = 0;    // head rotation around Y
  private headTiltZ = 0;    // head lean
  private hatSway = 0;
  private bodyLean = 0;
  private bounce = 0;
  private squashY = 1;
  private bodyY = 0;
  private bootSpread = 0;   // extra X offset for sitting
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

  // Walk direction
  private walkDirX = 0;
  private walkDirY = 0;
  private facingAngle = 0;

  constructor() {
    this.group = new THREE.Group();
    this.group.name = 'gardener';

    this.x = GRID_X / 2;
    this.y = GRID_Y / 2;
    this.z = GROUND_LEVEL + 1;

    // Build 3D gnome model
    this.parts = buildGnomeModel();
    this.group.add(this.parts.root);

    // Emotion particle system (same as before — point sprites work well)
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
    // Blink system
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

    // Smooth interpolation + apply to 3D model
    this.applyAnimation(dt);

    // Emotion particles
    this.updateEmotions(dt);

    this.updatePosition();
    return completedTask;
  }

  // ─── State Updates ──────────────────────────────────────────

  private updateIdle(dt: number, elapsed: number, queue: TaskQueue): GardenTask | null {
    if (queue.length > 0) {
      this.currentTask = queue.peek();
      this.state = GnomeState.Walking;
      this.idleBehavior = IdleBehavior.Standing;
      this.bootSpread = 0;
      this.toolType = this.getToolType(this.currentTask);
      return null;
    }

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
      this.x = tx;
      this.y = ty;
      this.state = GnomeState.Working;
      this.workTimer = WORK_DURATION;
      this.squashY = 0.85;
      this.bounce = -0.05;
      return;
    }

    const ndx = dx / dist;
    const ndy = dy / dist;
    this.walkDirX = ndx;
    this.walkDirY = ndy;
    this.x += ndx * Math.min(speed, dist);
    this.y += ndy * Math.min(speed, dist);

    // Face direction of movement
    this.facingAngle = Math.atan2(ndy, ndx);

    // Walking animation
    const walkPhase = this.x * 3.0 + this.y * 3.0;
    this.bounce = Math.abs(Math.sin(walkPhase * 2.0)) * 0.3;
    this.squashY = 1.0 + Math.sin(walkPhase * 4.0) * 0.04;
    this.bodyLean = Math.sin(walkPhase * 2.0) * 0.15;
    this.hatSway = Math.sin(walkPhase * 2.5) * 0.2;

    // Arms swing
    const swing = Math.sin(walkPhase * 2.0);
    this.armLAngle = 0.3 + swing * 0.4;
    this.armRAngle = 0.3 - swing * 0.4;

    this.cheekGlow = 0.3;
  }

  private updateWorking(dt: number, queue: TaskQueue): GardenTask | null {
    this.workTimer -= dt;

    const workPhase = (WORK_DURATION - this.workTimer) / WORK_DURATION;
    const pump = Math.sin(workPhase * Math.PI * 4);

    if (this.toolType >= 0.5 && this.toolType < 1.5) {
      // Shovel: dig motion
      this.armRAngle = 0.5 + pump * 0.6;
      this.armLAngle = 0.2;
      this.bodyLean = pump * 0.1;
      this.bounce = pump * 0.1;
    } else if (this.toolType >= 1.5 && this.toolType < 2.5) {
      // Watering can: gentle tipping
      this.armRAngle = 0.8 + Math.sin(workPhase * Math.PI * 2) * 0.3;
      this.armLAngle = 0.3;
      this.bodyLean = Math.sin(workPhase * Math.PI * 2) * 0.08;
    } else {
      // Seed bag: sprinkling
      this.armRAngle = 0.6 + Math.sin(workPhase * Math.PI * 6) * 0.2;
      this.armLAngle = 0.4;
      this.bodyLean = Math.sin(workPhase * Math.PI * 3) * 0.05;
      this.bounce = Math.abs(pump) * 0.06;
    }

    this.cheekGlow = 0.5;

    if (this.tasksCompletedStreak > 8 && Math.random() < dt * 0.5) {
      this.emitEmotion(EmotionType.Sweat);
    }

    if (this.workTimer <= 0) {
      const completed = queue.dequeue();
      this.currentTask = null;
      this.tasksCompletedStreak++;

      if (queue.length === 0) {
        this.startCelebration(
          this.tasksCompletedStreak > 10 ? CelebType.BigMilestone : CelebType.QueueEmpty
        );
        this.tasksCompletedStreak = 0;
      } else if (this.tasksCompletedStreak % 5 === 0) {
        this.startCelebration(CelebType.TaskDone);
      } else {
        this.currentTask = queue.peek();
        this.state = GnomeState.Walking;
        this.toolType = this.getToolType(this.currentTask);
      }

      if (completed) {
        if (completed.tool === ToolCode.Seed) {
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
      if (queue.length > 0) {
        this.currentTask = queue.peek();
        this.state = GnomeState.Walking;
        this.toolType = this.getToolType(this.currentTask);
      } else {
        this.state = GnomeState.Idle;
        this.idleTimer = randomRange(1.0, 3.0);
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
    if (this.state === GnomeState.Walking) {
      // In Three.js: sim X maps to world X, sim Y maps to world Z
      // facingAngle is atan2(dy, dx) where dy=sim Y, dx=sim X
      // We want the gnome to face in that direction
      const targetAngle = -this.facingAngle + Math.PI / 2;
      p.root.rotation.y = smoothTo(p.root.rotation.y, targetAngle, lerp);
    }

    // Head: tilt
    p.head.rotation.y = this.sHeadY;
    p.head.rotation.z = this.sHeadZ;

    // Hat: sway (rotation around Z relative to head)
    p.hat.rotation.z = this.sHat;

    // Arms: rotation around X axis (forward/backward swing)
    p.armL.rotation.x = -this.sArmL;
    p.armR.rotation.x = -this.sArmR;

    // Boots: spread when sitting
    p.bootL.position.x = -0.3 - this.sBootSpread;
    p.bootR.position.x = 0.3 + this.sBootSpread;

    // Eye blink: scale eyes to 0 when closed
    const eyeScale = this.eyesClosed ? 0.2 : 1.0;
    p.eyeL.scale.y = eyeScale;
    p.eyeR.scale.y = eyeScale;

    // Cheek glow: visibility based on intensity
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

    const baseX = this.x + 0.5;
    const baseY = this.z + 3.5;
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

  private getToolType(task: GardenTask | null): number {
    if (!task) return 0;
    switch (task.tool) {
      case ToolCode.Shovel: return 1;
      case ToolCode.Water: return 2;
      case ToolCode.Seed: return 3;
      case ToolCode.Soil: return 1;
      case ToolCode.Stone: return 1;
      default: return 0;
    }
  }

  private updatePosition(): void {
    // Sim (x,y,z) Z-up → Three.js (x,z,y) Y-up
    this.parts.root.position.x = this.x + 0.5;
    this.parts.root.position.z = this.y + 0.5;
    // Y position set by applyAnimation (includes bounce + bodyY)
    const baseY = this.z + 1.5;
    this.parts.root.position.y = baseY + this.sBounce + this.sBodyY;
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
