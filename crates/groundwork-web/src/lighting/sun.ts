/**
 * Lighting setup: warm directional sun + hemisphere fill.
 * "Shadows are never black — always tinted blue or purple."
 */

import * as THREE from 'three';
import { GRID_X, GRID_Y, GROUND_LEVEL } from '../bridge';

export interface Lights {
  sun: THREE.DirectionalLight;
  hemisphere: THREE.HemisphereLight;
  ambient: THREE.AmbientLight;
}

/**
 * Create the default "golden hour" lighting setup.
 */
export function createLighting(scene: THREE.Scene): Lights {
  // Warm directional sun — golden hour angle
  const sun = new THREE.DirectionalLight(0xffe4b5, 1.5); // moccasin warm
  sun.position.set(GRID_X * 0.7, -GRID_Y * 0.3, GROUND_LEVEL + 50);
  sun.target.position.set(GRID_X / 2, GRID_Y / 2, GROUND_LEVEL);

  // Shadow setup
  sun.castShadow = true;
  sun.shadow.mapSize.width = 2048;
  sun.shadow.mapSize.height = 2048;
  const shadowSize = 80;
  sun.shadow.camera.left = -shadowSize;
  sun.shadow.camera.right = shadowSize;
  sun.shadow.camera.top = shadowSize;
  sun.shadow.camera.bottom = -shadowSize;
  sun.shadow.camera.near = 1;
  sun.shadow.camera.far = 200;
  sun.shadow.bias = -0.001;

  scene.add(sun);
  scene.add(sun.target);

  // Hemisphere light: warm from above, cool green from below
  // Simulates light bouncing off vegetation
  const hemisphere = new THREE.HemisphereLight(
    0xffeedd, // warm sky
    0x445533, // green-tinted ground bounce
    0.6,
  );
  scene.add(hemisphere);

  // Subtle ambient to ensure nothing is pitch black
  // (tinted slightly blue-purple for cozy shadow color)
  const ambient = new THREE.AmbientLight(0x8888aa, 0.15);
  scene.add(ambient);

  return { sun, hemisphere, ambient };
}
