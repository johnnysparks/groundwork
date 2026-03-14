/**
 * Orthographic orbit camera with WASD fly, underground cutaway, and smooth transitions.
 *
 * Controls:
 *   WASD / Arrow keys  — Pan camera across the garden
 *   Scroll wheel       — Zoom in/out
 *   Left-drag          — Orbit (rotate around center)
 *   Q / E              — Lower / raise cutaway depth (slice underground)
 *   R                  — Reset camera to default view
 *   Shift              — Hold for faster pan speed
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GRID_Z, GROUND_LEVEL } from '../bridge';

/** Default camera state */
const DEFAULT_THETA = Math.PI / 4;      // 45 deg azimuth
const DEFAULT_PHI = Math.PI / 3;        // 60 deg diorama elevation
const DEFAULT_ZOOM = 1.0;
const DEFAULT_CENTER_X = GRID_X / 2;
const DEFAULT_CENTER_Y = GRID_Y / 2;
const DEFAULT_CENTER_Z = GROUND_LEVEL;

/** Camera limits */
const MIN_ZOOM = 0.3;
const MAX_ZOOM = 4.0;
const MIN_PHI = 0.2;
const MAX_PHI = Math.PI / 2 - 0.05;

/** Pan speed (voxels per second at zoom 1.0) */
const PAN_SPEED = 30;
const PAN_SPEED_FAST = 60;

/** Cutaway depth range */
const MIN_CUTAWAY_Z = 0;
const MAX_CUTAWAY_Z = GRID_Z;
const CUTAWAY_SPEED = 20; // voxels per second

export class OrbitCamera {
  readonly camera: THREE.OrthographicCamera;

  /** The clipping plane used for underground cutaway. Applied to materials. */
  readonly cutawayPlane: THREE.Plane;

  /** Spherical angles (radians) — current smoothed values */
  private theta = DEFAULT_THETA;
  private phi = DEFAULT_PHI;

  /** Damped targets */
  private targetTheta = DEFAULT_THETA;
  private targetPhi = DEFAULT_PHI;
  private targetZoom = DEFAULT_ZOOM;
  private targetCenter = new THREE.Vector3(DEFAULT_CENTER_X, DEFAULT_CENTER_Y, DEFAULT_CENTER_Z);

  /** Smoothed center */
  private center = new THREE.Vector3(DEFAULT_CENTER_X, DEFAULT_CENTER_Y, DEFAULT_CENTER_Z);

  /** Cutaway depth: everything above this Z gets clipped. GRID_Z = no cutaway. */
  private targetCutawayZ = MAX_CUTAWAY_Z;
  private cutawayZ = MAX_CUTAWAY_Z;

  /** Camera distance from center */
  private distance = 80;

  /** Damping factor (lower = smoother, 0.08 = ~12 frames to settle) */
  private damping = 0.08;

  /** Currently pressed keys (for continuous movement) */
  private keys = new Set<string>();

  /** Frustum half-size (for resize) */
  private frustumSize = 60;

  constructor(aspect: number) {
    this.camera = new THREE.OrthographicCamera(
      -this.frustumSize * aspect / 2,
      this.frustumSize * aspect / 2,
      this.frustumSize / 2,
      -this.frustumSize / 2,
      0.1,
      500,
    );

    // Cutaway clipping plane: normal pointing down (-Z), clips everything above cutawayZ
    // Plane equation: -z + cutawayZ >= 0, i.e. z <= cutawayZ
    this.cutawayPlane = new THREE.Plane(new THREE.Vector3(0, 0, -1), MAX_CUTAWAY_Z);

    this.updatePosition();
  }

  /** Call each frame with delta time in seconds */
  update(dt: number): void {
    // --- Keyboard-driven panning ---
    this.applyKeyboardPan(dt);

    // --- Keyboard-driven cutaway ---
    this.applyKeyboardCutaway(dt);

    // --- Damped interpolation ---
    this.theta += (this.targetTheta - this.theta) * this.damping;
    this.phi += (this.targetPhi - this.phi) * this.damping;

    this.center.x += (this.targetCenter.x - this.center.x) * this.damping;
    this.center.y += (this.targetCenter.y - this.center.y) * this.damping;
    this.center.z += (this.targetCenter.z - this.center.z) * this.damping;

    const currentZoom = this.camera.zoom;
    this.camera.zoom += (this.targetZoom - currentZoom) * this.damping;
    this.camera.updateProjectionMatrix();

    this.cutawayZ += (this.targetCutawayZ - this.cutawayZ) * this.damping;
    this.cutawayPlane.constant = this.cutawayZ;

    this.updatePosition();
  }

  /** Rotate camera by delta angles (from mouse drag) */
  rotate(dTheta: number, dPhi: number): void {
    this.targetTheta += dTheta;
    this.targetPhi = Math.max(MIN_PHI, Math.min(MAX_PHI, this.targetPhi + dPhi));
  }

  /** Zoom in/out by factor */
  zoom(factor: number): void {
    this.targetZoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, this.targetZoom * factor));
  }

  /** Set the look-at target directly */
  setCenter(x: number, y: number, z: number): void {
    this.targetCenter.set(x, y, z);
  }

  /** Reset camera to default diorama view */
  reset(): void {
    this.targetTheta = DEFAULT_THETA;
    this.targetPhi = DEFAULT_PHI;
    this.targetZoom = DEFAULT_ZOOM;
    this.targetCenter.set(DEFAULT_CENTER_X, DEFAULT_CENTER_Y, DEFAULT_CENTER_Z);
    this.targetCutawayZ = MAX_CUTAWAY_Z;
  }

  /** Notify that a key was pressed */
  keyDown(key: string): void {
    this.keys.add(key.toLowerCase());
  }

  /** Notify that a key was released */
  keyUp(key: string): void {
    this.keys.delete(key.toLowerCase());
  }

  /** Get the current cutaway Z level (for UI display) */
  getCutawayZ(): number {
    return this.cutawayZ;
  }

  /** Whether cutaway is active (not at maximum) */
  isCutawayActive(): boolean {
    return this.targetCutawayZ < MAX_CUTAWAY_Z - 0.5;
  }

  /** Handle window resize */
  resize(aspect: number): void {
    this.camera.left = -this.frustumSize * aspect / 2;
    this.camera.right = this.frustumSize * aspect / 2;
    this.camera.top = this.frustumSize / 2;
    this.camera.bottom = -this.frustumSize / 2;
    this.camera.updateProjectionMatrix();
  }

  /** Compute forward/right directions projected onto the XY ground plane */
  private getViewDirections(): { forward: THREE.Vector3; right: THREE.Vector3 } {
    // Forward = direction the camera is looking at, projected onto XY plane
    const forward = new THREE.Vector3(
      -Math.cos(this.theta),
      -Math.sin(this.theta),
      0,
    ).normalize();

    // Right = perpendicular to forward on XY plane
    const right = new THREE.Vector3(
      -Math.sin(this.theta),
      Math.cos(this.theta),
      0,
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
      // Clamp to grid bounds with some padding
      this.targetCenter.x = Math.max(-10, Math.min(GRID_X + 10, this.targetCenter.x));
      this.targetCenter.y = Math.max(-10, Math.min(GRID_Y + 10, this.targetCenter.y));
    }
  }

  private applyKeyboardCutaway(dt: number): void {
    const move = CUTAWAY_SPEED * dt;

    if (this.keys.has('q')) {
      this.targetCutawayZ = Math.max(MIN_CUTAWAY_Z, this.targetCutawayZ - move);
    }
    if (this.keys.has('e')) {
      this.targetCutawayZ = Math.min(MAX_CUTAWAY_Z, this.targetCutawayZ + move);
    }
  }

  private updatePosition(): void {
    const x = this.center.x + this.distance * Math.sin(this.phi) * Math.cos(this.theta);
    const y = this.center.y + this.distance * Math.sin(this.phi) * Math.sin(this.theta);
    const z = this.center.z + this.distance * Math.cos(this.phi);

    this.camera.position.set(x, y, z);
    this.camera.lookAt(this.center);
  }
}
