/**
 * Task queue for zone-planned work.
 *
 * Player actions (seed, water, dig, soil, stone) are queued here
 * instead of executing instantly. The gardener gnome drains the queue
 * one task at a time as it walks to each location.
 */

import type { ToolCodeType } from '../bridge';

export interface GardenTask {
  tool: ToolCodeType;
  x: number;
  y: number;
  z: number;
  species?: number; // species index for seed tool
}

export class TaskQueue {
  private tasks: GardenTask[] = [];

  /** Add a task to the queue */
  enqueue(task: GardenTask): void {
    // Don't duplicate tasks at the same position
    const exists = this.tasks.some(t => t.x === task.x && t.y === task.y && t.z === task.z);
    if (!exists) {
      this.tasks.push(task);
    }
  }

  /** Get the next task (peek, don't remove) */
  peek(): GardenTask | null {
    return this.tasks.length > 0 ? this.tasks[0] : null;
  }

  /** Remove and return the next task */
  dequeue(): GardenTask | null {
    return this.tasks.shift() ?? null;
  }

  /** Cancel all tasks at a position (right-click to cancel) */
  cancelAt(x: number, y: number, z: number): void {
    this.tasks = this.tasks.filter(t => !(t.x === x && t.y === y && t.z === z));
  }

  /** Cancel all tasks */
  cancelAll(): void {
    this.tasks = [];
  }

  /** Remove all pending tasks */
  clear(): void {
    this.tasks.length = 0;
  }

  /** Number of pending tasks */
  get length(): number {
    return this.tasks.length;
  }

  /** All tasks (for ghost overlay rendering) */
  get all(): readonly GardenTask[] {
    return this.tasks;
  }
}
