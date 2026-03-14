/**
 * Input controls: binds keyboard shortcuts and mouse click-to-place.
 *
 * Coordinates between the HUD (tool/species selection), the raycaster
 * (mouse → voxel), and the bridge (place_tool).
 */

import * as THREE from 'three';
import { ToolCode, type ToolCodeType, placeTool, isInitialized } from '../bridge';
import { Hud, TOOLS } from './hud';
import { raycastVoxel, type VoxelHit } from './raycaster';

export interface ControlsConfig {
  hud: Hud;
  camera: THREE.Camera;
  terrainGroup: THREE.Group;
  canvas: HTMLCanvasElement;
  /** Called after a tool is placed (to trigger re-mesh of affected chunks) */
  onToolPlaced?: (hit: VoxelHit) => void;
}

/**
 * Set up all input bindings: keyboard shortcuts, mouse click-to-place.
 * Returns a cleanup function to remove event listeners.
 */
export function setupControls(config: ControlsConfig): () => void {
  const { hud, camera, terrainGroup, canvas } = config;

  // --- Mouse state (to distinguish clicks from drags) ---
  let mouseDownX = 0;
  let mouseDownY = 0;
  let mouseDownTime = 0;
  const DRAG_THRESHOLD = 5; // pixels
  const CLICK_MAX_MS = 300;

  function onMouseDown(e: MouseEvent): void {
    if (e.button !== 0) return; // left click only
    mouseDownX = e.clientX;
    mouseDownY = e.clientY;
    mouseDownTime = performance.now();
  }

  function onMouseUp(e: MouseEvent): void {
    if (e.button !== 0) return;

    // Ignore if clicking on HUD
    if (hud.containsElement(e.target)) return;

    // Check if this was a click (not a drag)
    const dx = e.clientX - mouseDownX;
    const dy = e.clientY - mouseDownY;
    const dist = Math.sqrt(dx * dx + dy * dy);
    const elapsed = performance.now() - mouseDownTime;

    if (dist > DRAG_THRESHOLD || elapsed > CLICK_MAX_MS) return;

    handleClick(e.clientX, e.clientY);
  }

  function handleClick(screenX: number, screenY: number): void {
    const placing = hud.isPlacingTool();
    const hit = raycastVoxel(screenX, screenY, camera, terrainGroup, placing);
    if (!hit) return;

    if (isInitialized()) {
      placeTool(hud.state.activeTool, hit.x, hit.y, hit.z);
    } else {
      // Mock mode: apply to the local grid directly (handled in main.ts callback)
    }

    config.onToolPlaced?.(hit);
  }

  // --- Keyboard shortcuts ---

  function onKeyDown(e: KeyboardEvent): void {
    // Don't capture if user is typing in an input
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

    switch (e.key) {
      // Tool selection: 1-5
      case '1': case '2': case '3': case '4': case '5': {
        const idx = Number(e.key) - 1;
        if (idx < TOOLS.length) {
          hud.selectTool(TOOLS[idx].code);
        }
        break;
      }

      // Species cycling: Q/E
      case 'q':
      case 'Q':
        hud.cycleSpecies(-1);
        break;
      case 'e':
      case 'E':
        hud.cycleSpecies(1);
        break;

      // Auto-tick toggle is handled in main.ts (it owns the autoTick state)
    }
  }

  // --- Bind events ---
  canvas.addEventListener('mousedown', onMouseDown);
  canvas.addEventListener('mouseup', onMouseUp);
  document.addEventListener('keydown', onKeyDown);

  // --- Cleanup ---
  return () => {
    canvas.removeEventListener('mousedown', onMouseDown);
    canvas.removeEventListener('mouseup', onMouseUp);
    document.removeEventListener('keydown', onKeyDown);
  };
}
