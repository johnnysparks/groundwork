/**
 * Orthographic orbit camera for the diorama view.
 * Smooth damped rotation and zoom. Always looks at the garden center.
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GRID_Z, GROUND_LEVEL } from '../bridge';

export class OrbitCamera {
  readonly camera: THREE.OrthographicCamera;

  /** Spherical angles (radians) */
  private theta = Math.PI / 4;   // azimuth (45 degrees)
  private phi = Math.PI / 3;     // elevation (60 degrees = nice diorama angle)

  /** Damped targets (for smooth animation) */
  private targetTheta = this.theta;
  private targetPhi = this.phi;
  private targetZoom = 1.0;

  /** Center of the garden (look-at target) */
  private center = new THREE.Vector3(GRID_X / 2, GRID_Y / 2, GROUND_LEVEL);

  /** Camera distance from center */
  private distance = 80;

  /** Damping factor (lower = smoother) */
  private damping = 0.08;

  constructor(aspect: number) {
    const frustumSize = 60;
    this.camera = new THREE.OrthographicCamera(
      -frustumSize * aspect / 2,
      frustumSize * aspect / 2,
      frustumSize / 2,
      -frustumSize / 2,
      0.1,
      500,
    );
    this.updatePosition();
  }

  /** Update camera to match current angles (call each frame) */
  update(): void {
    // Damped interpolation toward target
    this.theta += (this.targetTheta - this.theta) * this.damping;
    this.phi += (this.targetPhi - this.phi) * this.damping;

    const currentZoom = this.camera.zoom;
    this.camera.zoom += (this.targetZoom - currentZoom) * this.damping;
    this.camera.updateProjectionMatrix();

    this.updatePosition();
  }

  /** Rotate camera by delta angles */
  rotate(dTheta: number, dPhi: number): void {
    this.targetTheta += dTheta;
    this.targetPhi = Math.max(0.2, Math.min(Math.PI / 2 - 0.05, this.targetPhi + dPhi));
  }

  /** Zoom in/out */
  zoom(factor: number): void {
    this.targetZoom = Math.max(0.3, Math.min(4.0, this.targetZoom * factor));
  }

  /** Set the look-at target */
  setCenter(x: number, y: number, z: number): void {
    this.center.set(x, y, z);
  }

  /** Handle window resize */
  resize(aspect: number): void {
    const frustumSize = 60;
    this.camera.left = -frustumSize * aspect / 2;
    this.camera.right = frustumSize * aspect / 2;
    this.camera.top = frustumSize / 2;
    this.camera.bottom = -frustumSize / 2;
    this.camera.updateProjectionMatrix();
  }

  private updatePosition(): void {
    const x = this.center.x + this.distance * Math.sin(this.phi) * Math.cos(this.theta);
    const y = this.center.y + this.distance * Math.sin(this.phi) * Math.sin(this.theta);
    const z = this.center.z + this.distance * Math.cos(this.phi);

    this.camera.position.set(x, y, z);
    this.camera.lookAt(this.center);
  }
}
