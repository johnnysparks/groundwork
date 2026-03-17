# Player Learning Arc

The game teaches ecology through play, not tutorials. Each hour of play shifts the player's mental model.

## Hour 1: Mechanics
**"Seeds need soil and water."**

The player paints density zones, adds water, and watches growth. They learn:
- Seeds need adjacent water (level >= 30) and light (level >= 30)
- Growth is visible within ~3 seconds (seed -> trunk at tick 17, first leaf at tick 27-35)
- The pond provides water that flows downhill through soil — dig channels to direct it
- Different species appear based on conditions — groundcover dominates early gardens

**Systems active:** seed_growth, water_flow, light_propagation, water_spring

## Hour 3: Competition
**"The oak's roots are stealing water from the birch."**

With multiple plants growing, the player observes:
- Trees too close together: some die, others thrive (root competition, shade stress)
- Seedlings under canopy die quickly (youth vulnerability 4x)
- Seeds near existing trunks won't germinate (territorial suppression)
- Some species need more light/water than others

**Systems active:** + root_water_absorption (competition), tree_growth (health), self_pruning

## Hour 10: Synergy
**"Clover fixes nitrogen, which feeds the oak, whose canopy shades the fern."**

The player discovers multi-step relationships:
- Planting clover near oaks makes oaks grow 1.5x faster (nitrogen handshake)
- Flowers attract pollinators that boost nearby tree health (pollinator bridge)
- Ferns grow *faster* in shade (canopy effect) — shade is good for some species
- Pine creates an "acid zone" where only fern/moss can grow (allelopathy)
- Dead trees sprout new seedlings nearby (nurse log effect)
- Berry bushes attract birds that spread seeds far away (bird express)

**Systems active:** + fauna_spawn, fauna_effects, pioneer_succession, allelopathy, nurse log

## Hour 20: Ecology as Architecture
**"I can design a self-sustaining loop where every species serves a role."**

The player designs intentional ecosystems:
- Plant grass first to bind soil, then trees for canopy
- Place willows by water, birch in clearings, pine in isolated corners
- Create flower meadows for pollinator swarms
- Plant same-species groves for mycorrhizal support
- Let dead trees nurture the next generation
- Weather cycles test resilience — deep-rooted, well-watered gardens survive drought

**All systems active.** The garden runs itself during idle time.

---

## Emergence Gating

The sim ensures players experience this arc naturally. The player paints density zones — the sim picks species based on environmental fitness and garden maturity:

| Garden State | What Emerges | Why |
|-------------|-------------|-----|
| Empty garden | Groundcover (moss, grass, clover) | 4x emergence multiplier — pioneer species |
| 3+ plants | + Flowers (wildflower, daisy) | Moderate maturity unlocks flowers |
| 10+ plants, 5+ groundcover | + Shrubs (fern, berry bush, holly) | Established groundcover supports understory |
| 20+ plants, 10+ groundcover | + Trees (oak, birch, willow, pine) | Only mature ecosystems can sustain trees |

Species aren't selected from menus. They're *discovered* through the inspect panel after they emerge. Each new species appearance triggers a discovery event.

## Design Philosophy

- **No explicit tutorials.** The quest system provides gentle guidance, but the real learning happens through observation and surprise.
- **Mistakes don't punish.** Dead trees recover with water. Pioneer succession fills bare patches. The garden always bounces back.
- **Surprise rewards observation.** The player who watches idle gets seed rain, pioneer succession, fauna activity. The player who experiments gets nitrogen boosts, pollinator bridges, allelopathy discoveries.
- **Knowledge transfers.** Your tenth garden is different from your first — not because of unlocks, but because you *understand ecology now*.
