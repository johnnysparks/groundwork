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
  // Warm directional sun — golden hour side-light (Y-up)
  const cx = GRID_X / 2;
  const cz = GRID_Y / 2;
  const sun = new THREE.DirectionalLight(0xffe4b5, 2.0); // warm, strong
  sun.position.set(cx + 60, GROUND_LEVEL + 40, cz - 20);  // front-right, golden hour
  sun.target.position.set(cx, GROUND_LEVEL, cz);

  // Shadows: tuned for the 80×80 garden diorama.
  // The shadow camera frustum covers the full garden + a margin for
  // trees near the edge. Bias prevents shadow acne on flat surfaces.
  sun.castShadow = true;
  const shadowSize = Math.max(GRID_X, GRID_Y) * 0.8;
  sun.shadow.camera.left = -shadowSize;
  sun.shadow.camera.right = shadowSize;
  sun.shadow.camera.top = shadowSize;
  sun.shadow.camera.bottom = -shadowSize;
  sun.shadow.camera.near = 1;
  sun.shadow.camera.far = 200;
  sun.shadow.mapSize.width = 2048;
  sun.shadow.mapSize.height = 2048;
  sun.shadow.bias = -0.002;
  sun.shadow.normalBias = 0.5;

  scene.add(sun);
  scene.add(sun.target);

  // Second fill light from the opposite side to reduce harsh contrast
  const fill = new THREE.DirectionalLight(0xddc8a0, 0.6);
  fill.position.set(cx - 40, GROUND_LEVEL + 20, cz + 30);
  fill.target.position.set(cx, GROUND_LEVEL, cz);
  scene.add(fill);
  scene.add(fill.target);

  // Hemisphere light: warm from above, cool green from below
  const hemisphere = new THREE.HemisphereLight(
    0xffeedd, // warm sky
    0x556644, // green-tinted ground bounce
    1.0,      // strong fill
  );
  scene.add(hemisphere);

  // Generous ambient — shadows should never be dark, just tinted
  const ambient = new THREE.AmbientLight(0xaa99cc, 0.4);
  scene.add(ambient);

  return { sun, hemisphere, ambient };
}
