# Build Notes — Starter Garden Drama (Sprint 340)

**Date:** 2026-03-18T14:00:00
**Builder:** Dev agent (Claude)
**Branch:** main

## Sprint 340 — Add ecological drama to starter garden

### Problem
Sprint 339's starter garden was healthy but TOO healthy — all 13,833 leaves thriving, 0 stressed. User feedback mandate: "sim needs drama."

### Investigation
1. Tried tightly-packed competing grove at y=45-50: seeds died (too far from pond, deprivation after 200 ticks)
2. Tried grove at y=36-40: same issue — soil too dry
3. Tried grove at y=24-28 near pond: territorial suppression (6-voxel radius from trunk) killed most seeds
4. Tried wider spacing (8-10 voxels) at pond edge: only 1-2 of 5 seeds survived

### Key Learnings
- **Territorial suppression** prevents seeds within 6 voxels of any trunk from germinating
- **Seed deprivation** kills seeds after 200 ticks without water
- **Pond moisture radius** after 50 pre-ticks is ~10-12 voxels from pond edge
- **Competition needs MATURE trees** (~500+ ticks) — young trees don't have big enough canopies to shade each other
- **Drama in starter garden comes from species mismatch, not crowding** — shade-intolerant pine planted near shade-casting trees dies naturally

### Solution
Added 3 more trees spread around the wet zone:
- Oak west (30, 20) — healthy, contributes to canopy
- Birch east (50, 20) — healthy, pioneer species
- Pine north (40, 14) — **DIES from shade competition** → visible drama!

Increased pre-ticks from 200 to 300.

### Results
- 37,080 leaves on first load (up from 13,833)
- 6 established trees: 1 oak, 1 birch, 1 moss, 1 wildflower, 1 clover, **1 dead pine**
- Dead pine (health=0, stage=Dead) = visible deadwood in the garden
- All 116 Rust tests pass, TypeScript clean

### Design Insight
The starter garden shouldn't show crowding drama (that needs mature trees). Instead, it shows **species mismatch drama** — a pine that couldn't compete for light. This teaches "placement matters" without requiring 500+ ticks of pre-computation.
