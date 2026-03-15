/**
 * Screenshot capture utility.
 *
 * Captures the Three.js canvas to a PNG blob. Supports:
 * - Manual download via keyboard shortcut (F2) or HUD button
 * - Programmatic capture via window.captureScreenshot() for automated agents
 */

let _canvas: HTMLCanvasElement | null = null;

/** Initialize with the renderer's canvas element */
export function initScreenshot(canvas: HTMLCanvasElement): void {
  _canvas = canvas;

  // Expose global API for programmatic capture (player agent, devtools)
  (window as any).captureScreenshot = captureScreenshot;
}

/**
 * Capture the canvas and return the PNG as a Blob.
 * Also triggers a browser download with a timestamped filename.
 *
 * Returns the Blob for programmatic use (e.g. attaching to PRs).
 */
export async function captureScreenshot(): Promise<Blob | null> {
  if (!_canvas) {
    console.warn('Screenshot: canvas not initialized');
    return null;
  }

  return new Promise((resolve) => {
    _canvas!.toBlob((blob) => {
      if (!blob) {
        console.warn('Screenshot: toBlob returned null');
        resolve(null);
        return;
      }

      // Auto-download
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `groundwork-${timestamp()}.png`;
      a.click();
      URL.revokeObjectURL(url);

      console.log(`Screenshot saved: ${a.download}`);
      resolve(blob);
    }, 'image/png');
  });
}

function timestamp(): string {
  const d = new Date();
  return d.toISOString().replace(/[:.]/g, '-').slice(0, 19);
}
