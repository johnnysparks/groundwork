/**
 * Orthographic orbit camera with WASD fly and smooth transitions.
 *
 * Coordinate convention: Three.js Y-up.
 *   Sim (x, y, z) where Z=up  →  Three.js (x, z, y) where Y=up.
 *   The camera and terrain renderer both use Y-up.
 *
 * Controls:
 *   WASD / Arrow keys  — Pan camera across the garden
 *   Scroll wheel       — Zoom in/out
 *   Left-drag          — Orbit (rotate around center)
 *   R                  — Reset camera to default view
 *   Shift              — Hold for faster pan speed
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GROUND_LEVEL } from '../bridge';

/** Default camera state (Y-up: sim Z → Three.js Y, sim Y → Three.js Z) */
const DEFAULT_THETA = Math.PI / 4;      // 45 deg azimuth
const DEFAULT_PHI = Math.PI / 3;        // 60 deg diorama elevation
const DEFAULT_ZOOM = 1.0;

/** Camera limits */
const MIN_ZOOM = 0.35;
const MAX_ZOOM = 4.0;
const MIN_PHI = 0.55; // ~32° from horizon — prevents seeing under the garden
const MAX_PHI = Math.PI / 2 - 0.05;

/** Pan speed (voxels per second at zoom 1.0) */
const PAN_SPEED = 30;
const PAN_SPEED_FAST = 60;

export class OrbitCamera {
  readonly camera: THREE.OrthographicCamera;

  /** Spherical angles (radians) — current smoothed values */
  private theta = DEFAULT_THETA;
  private phi = DEFAULT_PHI;

  /** Damped targets */
  private targetTheta = DEFAULT_THETA;
  private targetPhi = DEFAULT_PHI;
  private targetZoom = DEFAULT_ZOOM;
  private targetCenter = new THREE.Vector3();

  /** Smoothed center */
  private center = new THREE.Vector3();

  /** Camera distance from center */
  private distance = 100;

  /** Damping factor (lower = smoother, 0.08 = ~12 frames to settle) */
  private damping = 0.08;

  /** Currently pressed keys (for continuous movement) */
  private keys = new Set<string>();

  /** Frustum half-size (for resize) — scales with grid dimensions */
  private frustumSize = 80;

  /** Idle auto-orbit: slow rotation after no interaction for 45s */
  private idleTimer = 0;
  private idleOrbitActive = false;
  private static readonly IDLE_THRESHOLD = 45; // seconds
  private static readonly IDLE_ORBIT_SPEED = 0.04; // radians per second — very slow
  private static readonly IDLE_BREATHE_SPEED = 0.15; // sine wave speed for zoom breathe
  private static readonly IDLE_BREATHE_AMP = 0.03; // ±3% zoom variation
  private idleElapsed = 0; // cumulative time during idle orbit

  constructor(aspect: number, opts?: { mobile?: boolean }) {
    // Compute defaults from live grid dimensions (may have been overridden by demo grid)
    const cx = GRID_X / 2;
    const cy = GROUND_LEVEL;
    const cz = GRID_Y / 2;
    this.center.set(cx, cy, cz);
    this.targetCenter.set(cx, cy, cz);

    // For small grids (WASM 80×80), use grid size as frustum.
    // For large demo grids, cap at 100 so individual trees are visible.
    const maxDim = Math.max(GRID_X, GRID_Y);
    this.frustumSize = maxDim <= 100 ? maxDim : 100;
    this.distance = Math.max(100, maxDim * 0.5);

    // Mobile: start zoomed in ~50% so plants and fauna are visible on small screens
    if (opts?.mobile) {
      this.targetZoom = 1.6;
    }

    this.camera = new THREE.OrthographicCamera(
      -this.frustumSize * aspect / 2,
      this.frustumSize * aspect / 2,
      this.frustumSize / 2,
      -this.frustumSize / 2,
      0.1,
      this.distance * 10,
    );

    this.updatePosition();
  }

  /** Call each frame with delta time in seconds */
  update(dt: number): void {
    // --- Follow mode: gently track the target each frame ---
    if (this.followTarget) {
      const tx = this.followTarget.getX();
      const tz = this.followTarget.getY(); // sim Y → Three.js Z
      // Smooth follow — lerp target center toward the followed entity
      this.targetCenter.x += (tx - this.targetCenter.x) * 0.05;
      this.targetCenter.z += (tz - this.targetCenter.z) * 0.05;
      this.targetCenter.y = GROUND_LEVEL;
    }

    // --- Keyboard-driven panning ---
    this.applyKeyboardPan(dt);

    // --- Idle auto-orbit ---
    this.idleTimer += dt;
    if (this.idleTimer >= OrbitCamera.IDLE_THRESHOLD) {
      this.idleOrbitActive = true;
      this.targetTheta += OrbitCamera.IDLE_ORBIT_SPEED * dt;
      // Subtle zoom breathe — the garden "breathes" while idle
      this.idleElapsed += dt;
      const breathe = Math.sin(this.idleElapsed * OrbitCamera.IDLE_BREATHE_SPEED);
      const baseZoom = this.targetZoom;
      this.camera.zoom = baseZoom * (1 + breathe * OrbitCamera.IDLE_BREATHE_AMP);
      // Gentle phi oscillation — camera slowly nods up and down
      const phiCenter = (MIN_PHI + MAX_PHI) * 0.45; // slightly above middle
      const phiWave = Math.sin(this.idleElapsed * 0.08) * 0.12; // ±0.12 rad, very slow
      this.targetPhi = phiCenter + phiWave;
    }

    // --- Damped interpolation ---
    this.theta += (this.targetTheta - this.theta) * this.damping;
    this.phi += (this.targetPhi - this.phi) * this.damping;

    this.center.x += (this.targetCenter.x - this.center.x) * this.damping;
    this.center.y += (this.targetCenter.y - this.center.y) * this.damping;
    this.center.z += (this.targetCenter.z - this.center.z) * this.damping;

    const currentZoom = this.camera.zoom;
    this.camera.zoom += (this.targetZoom - currentZoom) * this.damping;
    this.camera.updateProjectionMatrix();

    this.updatePosition();
  }

  /** Rotate camera by delta angles (from mouse drag) */
  rotate(dTheta: number, dPhi: number): void {
    this.resetIdle();
    this.targetTheta += dTheta;
    this.targetPhi = Math.max(MIN_PHI, Math.min(MAX_PHI, this.targetPhi + dPhi));
  }

  /** Zoom in/out by factor */
  zoom(factor: number): void {
    this.resetIdle();
    this.targetZoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, this.targetZoom * factor));
  }

  /** Reset idle timer — call on any user interaction */
  resetIdle(): void {
    this.idleTimer = 0;
    this.idleOrbitActive = false;
    this.idleElapsed = 0;
    // User interaction cancels follow mode
    this.followTarget = null;
  }

  /** Whether the camera is in idle auto-orbit mode */
  get isIdleOrbiting(): boolean {
    return this.idleOrbitActive;
  }

  /** Set the look-at target directly (Three.js Y-up coords) */
  setCenter(x: number, y: number, z: number): void {
    this.resetIdle();
    this.targetCenter.set(x, y, z);
  }

  /** Set focus using sim coordinates (x, y) — converts to Three.js Y-up.
   *  Smooth camera pan to the target. */
  setFocus(simX: number, simY: number): void {
    this.setCenter(simX, GROUND_LEVEL, simY);
  }

  // --- Follow mode: smooth drone-like tracking of a moving target ---
  private followTarget: { getX: () => number; getY: () => number } | null = null;

  /** Start following a target. The camera smoothly tracks it each frame.
   *  Params are sim coords (x, y). Any user input cancels follow. */
  startFollow(target: { getX: () => number; getY: () => number }): void {
    this.followTarget = target;
    this.resetIdle();
  }

  /** Stop following */
  stopFollow(): void {
    this.followTarget = null;
  }

  /** Whether the camera is currently following a target */
  get isFollowing(): boolean {
    return this.followTarget !== null;
  }

  /** Gently nudge camera toward an event (doesn't reset idle).
   *  Moves 20% of the way toward the target, creating a subtle attention pull. */
  nudgeToward(x: number, y: number, z: number): void {
    this.targetCenter.x += (x - this.targetCenter.x) * 0.2;
    this.targetCenter.y += (y - this.targetCenter.y) * 0.2;
    this.targetCenter.z += (z - this.targetCenter.z) * 0.2;
  }

  /** Reset camera to default diorama view */
  reset(): void {
    this.resetIdle();
    this.targetTheta = DEFAULT_THETA;
    this.targetPhi = DEFAULT_PHI;
    this.targetZoom = DEFAULT_ZOOM;
    this.targetCenter.set(GRID_X / 2, GROUND_LEVEL, GRID_Y / 2);
  }

  /** Snap camera instantly to current targets (skip damping animation) */
  snap(): void {
    this.theta = this.targetTheta;
    this.phi = this.targetPhi;
    this.center.copy(this.targetCenter);
    this.camera.zoom = this.targetZoom;
    this.camera.updateProjectionMatrix();
    this.updatePosition();
  }

  /** Notify that a key was pressed */
  keyDown(key: string): void {
    this.resetIdle();
    this.keys.add(key.toLowerCase());
  }

  /** Notify that a key was released */
  keyUp(key: string): void {
    this.keys.delete(key.toLowerCase());
  }

  /** Current azimuth angle (for forest culling) */
  getTheta(): number {
    return this.theta;
  }

  /** Get the camera center position (for proximity checks) */
  getCenter(): THREE.Vector3 {
    return this.center;
  }

  /** Approximate pan speed (units/sec) — for reactive effects */
  getPanSpeed(): number {
    const dx = this.targetCenter.x - this.center.x;
    const dz = this.targetCenter.z - this.center.z;
    return Math.sqrt(dx * dx + dz * dz) / Math.max(this.damping, 0.01);
  }

  /** Handle window resize */
  resize(aspect: number): void {
    this.camera.left = -this.frustumSize * aspect / 2;
    this.camera.right = this.frustumSize * aspect / 2;
    this.camera.top = this.frustumSize / 2;
    this.camera.bottom = -this.frustumSize / 2;
    this.camera.updateProjectionMatrix();
  }

  /** Compute forward/right directions projected onto the XZ ground plane (Y-up) */
  private getViewDirections(): { forward: THREE.Vector3; right: THREE.Vector3 } {
    // Forward = direction the camera is looking at, projected onto XZ plane
    const forward = new THREE.Vector3(
      -Math.cos(this.theta),
      0,
      -Math.sin(this.theta),
    ).normalize();

    // Right = perpendicular to forward on XZ plane (cross product: up × forward)
    const right = new THREE.Vector3(
      Math.sin(this.theta),
      0,
      -Math.cos(this.theta),
    ).normalize();

    return { forward, right };
  }

  private applyKeyboardPan(dt: number): void {
    if (this.keys.size === 0) return;

    const speed = this.keys.has('shift') ? PAN_SPEED_FAST : PAN_SPEED;
    // Scale pan by inverse zoom so movement feels consistent
    const scaledSpeed = speed / Math.max(this.targetZoom, 0.3);
    const move = scaledSpeed * dt;

    const { forward, right } = this.getViewDirections();
    const delta = new THREE.Vector3();

    if (this.keys.has('w') || this.keys.has('arrowup')) delta.add(forward.clone().multiplyScalar(move));
    if (this.keys.has('s') || this.keys.has('arrowdown')) delta.add(forward.clone().multiplyScalar(-move));
    if (this.keys.has('d') || this.keys.has('arrowright')) delta.add(right.clone().multiplyScalar(move));
    if (this.keys.has('a') || this.keys.has('arrowleft')) delta.add(right.clone().multiplyScalar(-move));

    if (delta.lengthSq() > 0) {
      this.targetCenter.add(delta);
      // Clamp to grid bounds with some padding (X and Z are horizontal)
      this.targetCenter.x = Math.max(-10, Math.min(GRID_X + 10, this.targetCenter.x));
      this.targetCenter.z = Math.max(-10, Math.min(GRID_Y + 10, this.targetCenter.z));
    }
  }

  /** Spherical coords (Y-up): Y = vertical axis */
  private updatePosition(): void {
    const x = this.center.x + this.distance * Math.sin(this.phi) * Math.cos(this.theta);
    const y = this.center.y + this.distance * Math.cos(this.phi);
    const z = this.center.z + this.distance * Math.sin(this.phi) * Math.sin(this.theta);

    this.camera.position.set(x, y, z);
    this.camera.lookAt(this.center);
  }
}
