# Simulation Performance Optimization

**Date:** 2026-03-13
**Branch:** `claude/profile-simulation-performance-kX4Ab`
**Commits:** 4 (profiling infra + 3 optimization rounds)

## Results Summary

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Avg ms/tick | 20.6 | 11.0 | **1.87x faster** |
| Throughput | 48.6 ticks/sec | 91.0 ticks/sec | **+87%** |
| P50 | 20.57ms | 10.8ms | -47% |
| P99 | 22.00ms | 11.5ms | -48% |

All 69 tests pass. World state at tick 500 matches exactly between pre- and post-optimization.

## What Was Done

### Round 1: soil_evolution tick-skipping (20.6ms → 13.2ms, 1.56x)
- `soil_evolution` ran every tick but updates slowly-changing properties (bacteria, pH, organic)
- Changed to run every 10 ticks with 10× scaled increments
- Eliminated snapshot allocation, uses direct grid cell reads
- Inline neighbor checks with direct index arithmetic

### Round 2: Hot path micro-optimizations (13.2ms → 11.2ms, 1.18x)
- **soil_absorption**: Interleaved `(u8, u8)` snapshot, deferred water accumulator, cached drainage_rate
- **light_propagation**: Direct cell indexing, pre-computed attenuation constants
- **water_flow**: Direct index arithmetic, eliminated per-neighbor bounds checks
- **root_water_absorption**: Single tuple snapshot, inline neighbor macros
- **seed_growth**: Collect seed positions first instead of 864K snapshot

### Round 3: Attempted and reverted
- Tried soil-index approach (collect soil cell indices to skip ~65% non-soil cells)
- **Regressed** from 5.9ms to 6.5ms — index decomposition and cache disruption cost more than branch-predicted `continue`
- Reverted with explanatory comment

## Current Per-System Breakdown

```
System                     Avg(ms)   % Time
─────────────────────────────────────────────
soil_absorption              5.71    52.0%
water_flow                   2.07    18.8%
light_propagation            1.24    11.3%
root_water_absorption        1.01     9.2%
seed_growth                  0.28     2.5%
soil_evolution               0.33     3.0%
(others)                     0.34     3.2%
─────────────────────────────────────────────
TOTAL                       10.98   100.0%
```

## Key Learnings

1. **Sequential scan with branch prediction beats indexed access** for this workload (864K voxels, ~35% soil). The CPU prefetcher handles the linear scan perfectly.
2. **Interleaved tuple snapshots `(u8, u8)`** outperform split `Vec<u8>` pairs — one cache line fetch vs two.
3. **Tick-skipping** is the highest-leverage optimization for slowly-changing systems.
4. **soil_absorption at 5.7ms appears near memory-bandwidth limits** (~3.3ns per neighbor lookup). Further gains would require SIMD or parallel execution.

## Files Changed

- `crates/groundwork-sim/src/systems.rs` — all system optimizations
- `crates/groundwork-profiler/` — new profiling crate (3 modes: full, per-system, flamegraph)
- `build_notes/2026-03-13T08:30:00_simulation_profiling.md` — baseline profiling report
