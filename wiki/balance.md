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
