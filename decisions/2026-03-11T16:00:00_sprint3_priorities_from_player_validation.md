# Decision: Sprint 3 Priorities from Player Validation Round 1

**Date:** 2026-03-11T16:00:00
**Decision by:** Manager
**Input:** 6 player sessions across 5 personas (Seed Growth Tester, Ecologist, Optimizer, Garden Designer, Weekend Gardener, Spelunker)

## Context

Sprint 2 shipped seed growth. We ran the first player validation round with 6 sessions to answer: "Is the seed→root loop the core delight?"

**Answer: Yes, but the path to delight is mined.**

Every session confirmed the "one more seed" pull when the first root appeared. But 3/6 sessions reported near-quit moments before reaching that point. The two biggest problems:

1. **Destructive placement** (4/6 sessions): The most intuitive action (place water on seed to water it) destroys the seed. No warning, no undo. Ecologist proved the water spring itself can be permanently destroyed cell by cell.

2. **Invisible growth** (6/6 sessions): Zero feedback between placing a seed and it becoming a root. 40-tick black box. Players can't tell if a seed is growing, stuck, or broken. `inspect` shows `water_level: 0` on growing seeds (confusing because growth uses neighbor water, not own water).

## Decisions Made

### 1. Promoted VIS-02 (seed growth indicator) from P2 to P0
**Rationale:** Every single session asked for this. The growth loop is the core mechanic and it's invisible. This is a readability problem, not a nice-to-have. The design constraint "readability over realism" demands that the core mechanic be readable.

### 2. Created CLI-08 (placement validation) as P0
**Rationale:** A game where the obvious action destroys your work violates the "cozy" promise. This isn't a bug — it's a trust violation. Casual players quit over this. The fix is cheap (CLI validation only, no sim changes).

### 3. Kept underground play (horizontal light/water) at P3
**Rationale:** Spelunker made a strong case for horizontal light scatter and water pooling. These would unlock an entire playstyle. However: (a) the surface loop isn't proven safe yet, (b) implementing light scatter is a significant system change, (c) underground play is an expansion of the core loop, not the core loop itself. Prove the surface garden works first.

### 4. Kept batch placement at P2 (but marked as next-to-promote)
**Rationale:** 5/6 sessions flagged this as the #1 UX friction. However, it's friction — not breakage. Players can still play, just tediously. The P0s (safety and readability) unblock delight; batch placement unblocks scale. Do P0 first, then batch.

### 5. Root water absorption stays P1 (SIM-03)
**Rationale:** Creates first ecological chain AND introduces resource pressure. Optimizer specifically noted infinite water kills the optimization puzzle. Ecologist noted roots are inert. This is the next sim feature after Sprint 3's trust/readability fixes.

## What This Means for the Team

- **Dev Sprint 3:** CLI-08 (placement safety) + VIS-02 (growth visibility) + three quick P1 wins (default Z=15, lower wet-soil threshold, dark air indicator)
- **Player next session:** Test the safety nets and growth visibility
- **After Sprint 3:** SIM-03 (root water absorption) becomes the priority — first real ecological interaction
