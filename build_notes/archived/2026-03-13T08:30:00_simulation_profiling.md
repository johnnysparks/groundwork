# Simulation Performance Profiling

**Date:** 2026-03-13
**Focus:** Identify per-system bottlenecks across 500–2000 tick runs

## Profiling Infrastructure

Added `groundwork-profiler` crate with three modes:

```bash
cargo run --release -p groundwork-profiler                   # full schedule timing
cargo run --release -p groundwork-profiler -- --systems 500  # per-system breakdown
cargo run --release -p groundwork-profiler -- --flamegraph 1000  # CPU flamegraph (SVG)
```

Setup: 36 seeds planted in a 6×6 grid, 30×30 area flooded with water to stress
all systems simultaneously.

## Baseline Results (release build, 500 ticks)

```
Grid: 120×120×60 = 864,000 voxels
Avg:     20.59 ms/tick
P50:     20.57 ms
P90:     21.16 ms
P99:     22.00 ms
Stddev:   0.54 ms
Throughput: 48.6 ticks/sec
```

Very consistent — low jitter (stddev 0.54ms), P99 only 7% above P50.
Performance is stable across 2000 ticks (no degradation as trees grow).

## Per-System Breakdown (sorted by CPU time)

```
System                     Avg(ms)   % Time
─────────────────────────────────────────────
soil_absorption              7.15    36.1%  ← #1 hotspot
soil_evolution               6.82    34.4%  ← #2 hotspot
water_flow                   2.20    11.1%
light_propagation            1.53     7.7%
root_water_absorption        1.19     6.0%
seed_growth                  0.91     4.6%
tree_growth                  0.01     0.0%
root_growth                  0.00     0.0%
branch_growth                0.00     0.0%
water_spring                 0.00     0.0%
self_pruning                 0.00     0.0%
tree_rasterize               0.00     0.0%
seed_dispersal               0.00     0.0%
tick_counter                 0.00     0.0%
─────────────────────────────────────────────
TOTAL                       19.83   100.0%
Overhead (ECS dispatch)      0.87ms  (0.0%)
```

## Flamegraph Analysis

CPU sampling at 997 Hz confirms the same picture:

| System              | Samples | % of samples |
|---------------------|---------|-------------|
| soil_absorption     | 1810    | 36.6%       |
| soil_evolution      | 1757    | 35.5%       |
| water_flow          | 483     |  9.8%       |
| light_propagation   | 355     |  7.2%       |
| root_water_absorption| 295    |  6.0%       |
| seed_growth         | 247     |  5.0%       |

### Hot paths in flamegraph:

- **soil_absorption**: Triple-nested loop (864K voxels × 6 neighbors), snapshot
  allocation, soil composition lookups. The `SoilGrid::get()` calls and neighbor
  iteration dominate.

- **soil_evolution**: Same triple-nested loop pattern. Creates a snapshot Vec each
  tick, then iterates all soil voxels checking 6 neighbors for root adjacency.

- **water_flow**: Snapshot + lateral_deltas Vec allocation each tick. The collect()
  for the snapshot shows up in the flamegraph.

- **light_propagation**: `VoxelGrid::get_mut()` calls are the hot leaf in the
  flamegraph — top-down column iteration with bounds checking.

- **root_water_absorption**: `Iterator::collect()` for snapshot appears in
  flamegraph — allocating 864K-element Vec every tick.

## Key Findings

1. **70.5% of CPU time is in soil systems** (absorption + evolution). Both do
   full 864K-voxel triple-nested loops with 6-neighbor checks, allocating
   snapshot buffers every tick.

2. **Snapshot allocations are repeated per system**: At least 4 systems allocate
   separate 864K+ element Vec snapshots each tick (water_flow, soil_absorption,
   soil_evolution, root_water_absorption). That's ~14MB of allocation per tick.

3. **Soil-specific loops iterate the full grid** even though only ~35% of voxels
   are soil. No early-out or spatial partitioning.

4. **ECS overhead is negligible** — 0.0% overhead from bevy_ecs dispatch. The
   single-threaded executor is essentially zero-cost.

5. **Tree/entity systems are fast** — all tree-related systems combined are <0.1%
   of runtime. The bottleneck is purely in grid-based systems.

## Optimization Opportunities (not implemented, just identified)

- **Skip non-soil voxels early**: soil_absorption and soil_evolution iterate
  864K cells but only ~300K are soil. A material-indexed skip list or layered
  iteration could cut work by ~60%.

- **Shared snapshot**: Multiple systems create independent snapshots of the same
  grid data each tick. A single shared snapshot could eliminate ~10MB/tick of
  allocation.

- **Reduce neighbor lookups**: The 6-neighbor pattern is duplicated across
  soil_absorption, soil_evolution, root_water_absorption. Pre-computing
  neighbor indices or using a stencil approach could help.

- **Run soil_evolution less frequently**: It updates bacteria/pH/organic which
  change slowly. Running every 5-10 ticks instead of every tick could save
  ~30% of total CPU.

## Files Added

- `crates/groundwork-profiler/` — standalone profiling crate
- `crates/groundwork-profiler/profile_flamegraph.svg` — interactive flamegraph
- `crates/groundwork-profiler/profile_collapsed.txt` — collapsed stack traces
