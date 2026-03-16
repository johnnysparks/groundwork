# Balance Philosophy

## Principles

1. **Cozy, not punishing.** Mistakes recover. Dead trees come back with water. Pioneer succession fills gaps. The garden always bounces back.

2. **Competition must be visible.** Crowded trees must actually die. This requires: shade penalty > partial recovery rate. Currently shade ~0.006/tick vs partial recovery 0.002/tick = net -0.004/tick death.

3. **Water must matter.** Removing water from the garden must eventually kill plants. Root water decay (-2/tick without wet soil) ensures this.

4. **First payoff within 3 seconds.** Competitive benchmark: cozy games deliver visual feedback in 1-5 seconds. Our seed -> trunk at ~1.7s, first leaf at ~3s.

5. **Each species must feel different.** No two species should serve the same ecological role. If they do, one is redundant.

6. **Idle time must produce change.** A player who watches for 2 minutes should see: seed rain, pioneer succession, fauna movement, weather changes, health fluctuation.

## Key Tuning Values

| Parameter | Value | Why |
|-----------|-------|-----|
| Seed growth rate | 12/tick | First trunk in ~17 ticks (1.7s) |
| Seedling -> Sapling threshold | 80 | First leaf in ~8 ticks after trunk |
| Initial accumulated resources | 40 | Head start on first stage transition |
| Health recovery (full resources) | +0.02/tick | Garden stays mostly green |
| Health recovery (partial) | +0.002/tick | Must be < shade penalty |
| Shade penalty | ~0.006/tick | Must be > partial recovery |
| Youth vulnerability | 4x/3x/2x/1x | Natural thinning of crowded seedlings |
| Root water decay | -2/tick | Isolated roots dry in ~100 ticks |
| Pioneer succession check | Every 50 ticks | Visible idle-time activity |
| Weather event frequency | ~200-400 ticks | Dramatic but not constant |

## Competitive Benchmarks

From the competitive analysis (Stardew Valley, Tiny Glade, Equilinox, Timberborn):

| Metric | Benchmark | Our Value | Status |
|--------|-----------|-----------|--------|
| Time to first visual change | 1-5 seconds | 1.7s (trunk) | OK |
| Time to first payoff (leaf) | 3-10 seconds | ~3s | OK |
| Water removal consequence | Growth stops or dies | Root decay -> death | OK |
| Drought consequence | Visible stress | Health decline + weather system | OK |
| Idle-time activity | Visible changes | 34 events / 600 ticks | OK |

## Anti-Patterns to Avoid

- **Recovery > stress:** If partial recovery nearly cancels shade penalty, competition doesn't work and trees never die. Keep partial < penalty.
- **Instant everything:** Growth that's too fast removes the satisfaction of watching progression. 17 ticks is the floor.
- **Invisible interactions:** Every interaction chain should produce a visible result the player can trace. If an interaction only changes internal numbers, it's not discoverable.
- **Symmetric species:** If oak and birch play identically, one is redundant. Every species needs a unique strategic niche.

## Known Issues / Open Tuning Questions

### Self-pruning may be too weak
Feedback: "Two oaks with overlapping canopies produce 0 deadwood." The self_pruning system requires shade_stress to exceed species.prune_threshold (200 for oak). With current light attenuation, overlapping canopies may not create shade dark enough to trigger pruning. Consider lowering prune_threshold or increasing leaf attenuation further.

### Root growth is invisible
Roots appear at ~tick 75 (underground) but the player has no visual indicator without x-ray mode. Consider: subtle ground-level darkening where roots spread, or a particle effect when a root system expands. This is a renderer task, not a sim task.

### Deadwood competition isn't narrated
When competition kills branches, it's just a brown voxel change. An event feed message ("An oak lost a branch to shade") would teach the player that competition is happening. This is a UI task.

### Growth feels exponential, not linear
Tick 100-200 is explosive compared to tick 0-100 due to sqrt accumulation with increasing root/leaf counts. This is ecologically correct but may confuse new players who expect steady growth. Not a bug, but worth monitoring in playtests.

### Trunk-to-canopy ratio feels wrong
Feedback: "Trees look like tall brown sticks with green blobs on top." The leaf sphere radius around branch tips (1/2/3 voxels for seedling/young/mature) may need increasing, or we need more branch tips per tree to create denser canopy coverage. This is a `tree_rasterize` tuning issue — specifically the `leaf_r` values and the number of attraction points generated per growth stage.

### Leaf health encoding must use non-zero default
Leaf/branch voxels encode tree health in `water_level` (0=dead, 255=healthy). A bug was found where `water_level=0` caused the renderer to tint all foliage amber ("dead" color). **New leaves must always have health > 0.** The tree_rasterize system writes `(tree.health * 255.0) as u8` — if health is very low but non-zero, this rounds to 0. Consider using `.max(1)` to ensure leaves always show as at least minimally alive.
