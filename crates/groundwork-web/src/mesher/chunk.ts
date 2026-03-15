/**
 * Chunk manager: tracks which chunks are dirty and need re-meshing.
 * Compares grid snapshots after each tick to detect changes.
 */

import {
  GRID_X, GRID_Y, GRID_Z, VOXEL_BYTES,
} from '../bridge';
import {
  CHUNK_SIZE, chunksX, chunksY, chunksZ,
  meshChunk, type MeshQuad,
} from './greedy';

/** Per-chunk mesh state */
export interface ChunkMesh {
  cx: number;
  cy: number;
  cz: number;
  quads: MeshQuad[];
  dirty: boolean;
}

export class ChunkManager {
  private chunks: ChunkMesh[];
  private prevSnapshot: Uint8Array | null = null;
  private nx: number;
  private ny: number;

  constructor() {
    const nx = chunksX(), ny = chunksY(), nz = chunksZ();
    this.nx = nx;
    this.ny = ny;
    this.chunks = [];
    for (let cz = 0; cz < nz; cz++) {
      for (let cy = 0; cy < ny; cy++) {
        for (let cx = 0; cx < nx; cx++) {
          this.chunks.push({ cx, cy, cz, quads: [], dirty: true });
        }
      }
    }
  }

  /** Get chunk index from chunk coordinates */
  private chunkIndex(cx: number, cy: number, cz: number): number {
    return cx + cy * this.nx + cz * this.nx * this.ny;
  }

  /** Mark all chunks as dirty (for initial mesh or full rebuild) */
  markAllDirty(): void {
    for (const chunk of this.chunks) {
      chunk.dirty = true;
    }
  }

  /**
   * Detect which chunks changed by comparing the grid to the previous snapshot.
   * Only marks changed chunks as dirty.
   */
  detectChanges(grid: Uint8Array): void {
    if (this.prevSnapshot === null) {
      this.markAllDirty();
      this.prevSnapshot = new Uint8Array(grid);
      return;
    }

    // Compare voxel-by-voxel and mark containing chunks dirty
    const totalVoxels = GRID_X * GRID_Y * GRID_Z;
    for (let i = 0; i < totalVoxels; i++) {
      const byteOffset = i * VOXEL_BYTES;
      // Only check material byte (index 0) for mesh changes
      if (grid[byteOffset] !== this.prevSnapshot[byteOffset]) {
        // Convert flat index to (x, y, z)
        const x = i % GRID_X;
        const y = Math.floor(i / GRID_X) % GRID_Y;
        const z = Math.floor(i / (GRID_X * GRID_Y));
        const cx = Math.floor(x / CHUNK_SIZE);
        const cy = Math.floor(y / CHUNK_SIZE);
        const cz = Math.floor(z / CHUNK_SIZE);
        this.chunks[this.chunkIndex(cx, cy, cz)].dirty = true;

        // Also mark neighbor chunks dirty if voxel is on chunk boundary
        // (face culling depends on neighbors across chunk borders)
        if (x % CHUNK_SIZE === 0 && cx > 0)
          this.chunks[this.chunkIndex(cx - 1, cy, cz)].dirty = true;
        if (x % CHUNK_SIZE === CHUNK_SIZE - 1 && cx < this.nx - 1)
          this.chunks[this.chunkIndex(cx + 1, cy, cz)].dirty = true;
        if (y % CHUNK_SIZE === 0 && cy > 0)
          this.chunks[this.chunkIndex(cx, cy - 1, cz)].dirty = true;
        if (y % CHUNK_SIZE === CHUNK_SIZE - 1 && cy < this.ny - 1)
          this.chunks[this.chunkIndex(cx, cy + 1, cz)].dirty = true;
        if (z % CHUNK_SIZE === 0 && cz > 0)
          this.chunks[this.chunkIndex(cx, cy, cz - 1)].dirty = true;
        if (z % CHUNK_SIZE === CHUNK_SIZE - 1 && cz < chunksZ() - 1)
          this.chunks[this.chunkIndex(cx, cy, cz + 1)].dirty = true;
      }
    }

    // Update snapshot
    this.prevSnapshot.set(grid);
  }

  /**
   * Re-mesh all dirty chunks. Returns the chunks that were updated.
   * Call after detectChanges().
   */
  rebuildDirty(grid: Uint8Array): ChunkMesh[] {
    const updated: ChunkMesh[] = [];
    for (const chunk of this.chunks) {
      if (!chunk.dirty) continue;
      chunk.quads = meshChunk(grid, chunk.cx, chunk.cy, chunk.cz);
      chunk.dirty = false;
      updated.push(chunk);
    }
    return updated;
  }

  /** Get all chunks (for initial scene build) */
  allChunks(): readonly ChunkMesh[] {
    return this.chunks;
  }

  /** Count of dirty chunks (for diagnostics) */
  dirtyCount(): number {
    return this.chunks.filter(c => c.dirty).length;
  }
}
