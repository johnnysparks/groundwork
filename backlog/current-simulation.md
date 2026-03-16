# Backlog — Simulation Enhancement Workstream
**Last updated:** 2026-03-16T12:10:00
**Theme:** Bring the garden to life through simulation depth

---

## Just Shipped ✓
- Canopy shade competition (doubled leaf attenuation)
- Territorial seed suppression (no germination near trunks)
- Youth vulnerability (seedlings die 3-4× faster under stress)
- Root water competition (shared pool)
- Species-aware seed spacing
- **Dead tree decomposition** — DeadWood passively decays into nutrient-rich soil (~2500 ticks). Moisture accelerates. Beetles further accelerate via fauna_effects.
- **Canopy Effect** — Shade-tolerant species (fern, moss, holly) get 1.5× growth boost in moderate shade (light 5-30). Creates undergrowth niche.
- **Pollinator Bridge** (SIM-12) — Bees/butterflies near trees boost health recovery +0.005/pollinator. Creates flower→bee→healthier tree loop.
- **Pine Allelopathy** (SIM-16) — Pine roots acidify soil (pH drops 5x faster). Non-tolerant seeds grow at half speed in acidic soil. Pine/fern/moss immune.
- **Bird Express Enhancement** — Birds carry species-specific seeds (not just oak), enrich soil with droppings, spawn more readily near berry bushes.
- **Drought Recovery** — Dead trees with wet roots slowly revive as Saplings. "My tree came back!"
- **Willow Water Affinity** — Willows grow 2× faster near water. "Plant willows by the stream."
- **Birch Pioneer Vigor** — Birch grows 1.5× faster in open ground (no nearby trunks).
- **Berry-Bird Symbiosis** — Berry bushes grow 1.5× faster near birds. Completes the bird loop.
- **Grass/Clover Soil Binding** — Grass and clover roots increase soil clay (better water retention).
- **Nurse Log Effect** (SIM-17) — Seeds near DeadWood germinate 2× faster. Pioneer succession sprouts in drier conditions near dead wood.
- **Overgrowth Carrying Capacity** — Dense root zones (4+ adjacent roots) suppress soil bacteria. Self-regulating ecosystem density.
- **OldGrowth Seed Rain** — OldGrowth trees disperse seeds 2× more frequently. Visible autonomous garden spread.
- **Flower Meadow Effect** — Flower clusters double pollinator spawn probability and support more pollinators (5 vs 3).
- **Mycorrhizal Network** — Same-species trees with nearby roots share health. Healthier trees support struggling neighbors.
- **Wind Seed Drift** — Seeds high in air drift with rotating wind direction. Creates directional spread patterns.
- **Squirrel Acorn Caching** — New fauna type. Spawns near oaks/berry bushes. Caches acorns that sprout into oak seedlings at random locations.
- **Competition Tuning** — Reduced partial recovery (+0.005→+0.002) so shade stress is lethal. Crowded clusters now thin naturally.
- **Acceptance Test** — `crowded_oak_cluster_thins_naturally` validates core competition promise.
- **Growth Pacing Fix (P0)** — Seed growth 5→12/tick, Seedling→Sapling 200→80, Sapling→YoungTree 800→500. First leaves at ~tick 25 (was 100+).
- **Seasonal Day Phase (SIM-15)** — DayPhase resource (0-99): dawn=75%, day=100%, dusk=75%, night=50% growth. WASM exports for JS sync.
- **Ecological Milestones** — EcoMilestones resource: tier 0 groundcover → tier 1 flowers → tier 2 shrubs → tier 3 trees. WASM exports for JS species picker gating.
- **Weather System** — Rain/drought events every ~200-400 ticks. Rain floods surface; drought evaporates water. 500-tick initial grace period. WASM exports for JS visual effects.
- **Root Water Decay** — Roots without wet soil neighbors lose 2 water/tick. Makes drought lethal; enforces water placement strategy.
- **First Leaf Acceleration** — New trees start with accumulated=40 (was 0). First leaf at ~tick 30 (was ~tick 130).

---

## Workstream Complete — All P0/P1 Items Closed

The simulation enhancement workstream is **complete** after 21 sprints.

**All P1 items closed:**
- SIM-14 (root war viz data) — shipped: pack_tree_stats() WASM export ✓
- Species picker milestone integration — shipped: discovery + milestones wired to HUD ✓
- Root water decay + growth pacing — shipped ✓

**Cross-team items for visual/default teams:**
- Weather visual effects (rain particles, palette shift) — use get_weather_state() WASM export
- Root territory circles in x-ray mode — use pack_tree_stats() WASM export

**P2 — Future (if workstream reopens):**
- Fire mechanics
- Disease spread
- Squirrel domestication UI (gnome interaction)
