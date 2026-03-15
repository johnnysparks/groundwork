# Decision: Replayability Model — Knowledge Transfer and Biome Variety

**Date:** 2026-03-15
**Status:** Active
**Context:** The experiential vision doc defines what a single garden session should feel like. This decision defines what brings the player *back* — what makes garden #11 as exciting as garden #1, and garden #50 more exciting than both.

## The Two Pillars of Replayability

Replayability in Groundwork comes from exactly two sources, working in concert:

### Pillar 1: Knowledge Transfer — "I learned something. Let me try again."

Each garden teaches the player something they carry forward. Not an unlock. Not a power-up. An *understanding*.

- Garden #1: "Seeds need soil and water." You discover basic mechanics.
- Garden #3: "Clover first — it fixes nitrogen for everything else." You discovered symbiosis.
- Garden #5: "Space the trees or their roots fight." You discovered competition.
- Garden #8: "Flowers near water bring pollinators, which spread everything faster." You discovered fauna triggers.
- Garden #12: "I can design self-sustaining loops where every species serves a role." You're composing ecosystems.

The player starts each new garden *differently* because they understand more. Not because they have better tools — because they have better intuition. The feeling is: "Last time I learned X. This time I'm going to build around that from the start, and see what *new* thing I discover."

This is the Outer Wilds model: knowledge is the only progression system. But unlike Outer Wilds (which has a single mystery to solve), Groundwork's interaction web is rich enough that there's always another layer. The player who has mastered symbiosis hasn't yet mastered succession timing. The one who has mastered succession hasn't yet learned to read canopy light patterns. Discovery is deep, not wide.

**Kill metric:** Does the player's tenth garden look meaningfully different from their first — not because of unlocks, but because they *plant differently based on what they learned*?

### Pillar 2: Biome Variety — "New organisms, new rules, new vibes."

Each biome is a complete ecosystem with its own species, interactions, niches, and visual identity. Biomes are *both* content packs *and* difficulty curves. They are not ordered — you can start in any biome and move to any other.

**Why biomes are content packs:**
- Each biome has its own species roster (12-20 species per biome). Desert has cacti, succulents, and drought-adapted shrubs. Wetland has reeds, water lilies, and mangroves. Boreal has spruce, lichen, and lingonberry.
- Each biome has its own fauna. Desert has lizards and hawk moths. Wetland has frogs and dragonflies. Boreal has woodpeckers and snow hares.
- Each biome has its own visual identity — art style, color palette, lighting mood, particle effects, ambient atmosphere. The player *wants to see* what the coral reef looks like. What does the boreal fog feel like? What's the savanna golden hour? Visual identity is a pull motivator — finishing one biome makes you curious about the next just to experience its aesthetic.
- Each biome has its own signature interactions and named moments (its own "Nitrogen Handshake," its own "Bird Express"). The interaction web is biome-specific.

**Why biomes are difficulty curves:**
- Each biome teaches different ecological principles. Temperate teaches symbiosis and competition. Desert teaches water economy and spacing. Wetland teaches nutrient cycling and hydrology. Tundra teaches resilience and slow succession.
- Mastery doesn't transfer cleanly. The clover-oak nitrogen handshake doesn't exist in the desert. The player who mastered temperate forest composition arrives in the desert and must re-learn: What fixes nitrogen here? What disperses seeds when there are no berry-eating birds? What does "too much water" even mean in a place with almost none?
- But *systemic intuition* transfers. The player who learned to diagnose problems underground in the temperate biome will think to look underground in the desert. The player who learned that spacing matters for roots will apply that principle to cacti — even though the specifics are different. The *framework* transfers; the *recipes* don't.
- Each biome resets "The Slow Bloom" — the feeling of getting better over time. You're an expert in temperate, but a beginner in desert. That beginner feeling — with the confidence of knowing *how to learn* — is the sweet spot.

**Why biomes are not ordered:**
- Any biome can be a player's first. The temperate biome is the gentlest introduction (most forgiving, most familiar species), but a player who starts in desert or wetland will learn just as effectively — they'll just learn different principles first.
- There's no "complete temperate to unlock desert." Biomes are parallel, not sequential. The player chooses based on curiosity and aesthetic pull, not progression gates.
- A player might alternate: temperate garden, then desert garden, then back to temperate with desert-learned intuition about water conservation. Cross-biome knowledge creates surprising "aha" moments.

**Kill metric:** Does a temperate-expert player feel like a *curious beginner* when they enter a new biome? Do they carry intuition but not recipes? Do they want to see the new biome's visual world?

## How the Two Pillars Interact

The magic is in the interplay:

1. **Within a biome:** Knowledge transfer drives "one more garden." Each garden teaches something that makes the next one richer. This is the hour-to-hour loop.

2. **Across biomes:** Biome variety drives "one more biome." Mastering one ecosystem makes you curious about how a completely different one works — and what it *looks and feels* like. This is the week-to-week loop.

3. **Cross-pollination:** Knowledge from one biome subtly informs another. You learned water conservation in the desert; now your temperate gardens use water more intentionally. You learned layered canopy design in the rainforest; now your boreal gardens have more vertical structure. The player's overall ecological intuition deepens across all biomes.

4. **Visual motivation:** Each biome is a world you want to *inhabit*. The warm golden hour of temperate, the stark beauty of desert dawn, the misty intimacy of boreal twilight, the luminous underwater shimmer of wetland. Finishing one biome's learning curve doesn't just prepare you for another — it makes you *want to see* another.

## The Replayability Arc

**Hours 1-10 (Single Biome Mastery):**
The player works through the knowledge-as-progression curve in their first biome. Mechanics → competition → synergy → ecosystem architecture. Each garden is better because they understand more. The "one more garden" hook is discovery.

**Hours 10-20 (Biome Curiosity):**
The player has built expert-level gardens in their first biome. They've seen most of its named interactions. They notice a new biome in the menu. The art preview pulls them in — what does *that* world look like? They start, and suddenly they're a beginner again. But a confident beginner. They know *how* to learn (look underground, watch fauna, space plants, observe before acting). The new species are unfamiliar, the interactions are different, but the discovery arc reignites.

**Hours 20-50 (Cross-Biome Intuition):**
The player has played 2-3 biomes. Their understanding of ecology is now *general*, not biome-specific. They start seeing patterns: every biome has a nitrogen-fixer, a pioneer colonizer, a canopy-maker, a dispersal agent. But the *specific organisms* that fill those niches are different. The player thinks like an ecologist now — in terms of *roles* and *niches*, not species names. Going back to an earlier biome with this meta-understanding produces new discoveries they missed the first time.

**Hours 50+ (Ecological Architect):**
The player designs gardens in any biome with deep intentionality. But the simulation still surprises them — because each biome's interaction web is rich enough that there's always a chain they haven't triggered, a succession pattern they haven't seen, a fauna behavior they haven't witnessed. The game is inexhaustible not because it's infinite, but because the combinatorial space of species interactions within each biome is genuinely deep.

## Implications for MVP and Beyond

**MVP (Current Scope):**
- One temperate biome, fully realized. This is where the knowledge-transfer pillar is proven.
- The sim architecture must be *biome-ready* even if only temperate ships first. Species, interactions, fauna, and visual palettes should be data-driven, not hardcoded to temperate assumptions.
- The temperate biome must be deep enough to sustain hours 1-10 on its own. If the knowledge-transfer pillar doesn't work within a single biome, adding more biomes won't save it.

**Post-MVP (Biome Expansion):**
- Each new biome is a major content release — new species, new fauna, new interactions, new art, new music, new named moments.
- Biomes are designed as *ecological puzzles with different constraints*: desert (water scarcity, heat, spacing), wetland (water abundance, nutrient cycling, flooding), boreal (cold, slow growth, resilience), tropical (competition, density, vertical layering).
- The autonomous simulation loop should run per-biome: each biome gets its own thousand runs to surface *its* 3-5 magical interactions. A winning interaction in desert ("The Cactus Reservoir") is different from temperate ("The Nitrogen Handshake").
- Cross-biome discovery tracking: the loop should test whether skills from biome A produce interesting (not optimal — *interesting*) behavior in biome B.

## Design Constraints This Creates

1. **Sim architecture must be biome-parameterized.** Species rosters, interaction rules, growth rates, soil types, water behavior, light patterns, fauna triggers — all must be configurable per biome, not hardcoded.

2. **Visual system must support biome palettes.** Material colors, lighting presets, sky gradients, particle effects, ambient atmosphere — all driven by biome configuration. Switching biomes should feel like entering a different painting.

3. **Named interactions must be biome-specific.** Each biome discovers and names its own winning interactions. The loop runs independently per biome. Cross-biome comparisons are for meta-learning, not for homogenizing.

4. **No biome should be "harder" in a punitive sense.** Desert is harder than temperate in that resources are scarcer and spacing matters more — but recovery is still generous, experimentation is still safe, and the garden still composes itself. Cozy applies to all biomes. A desert garden recovers from drought just as a temperate garden recovers from flooding.

5. **Biome selection must be pull-motivated.** The player chooses a biome because they're *curious about it* — the art, the organisms, the feel. Not because they "completed" the previous one. No completion gates, no achievement requirements. Just: "That looks beautiful. I want to garden *there*."

## Relationship to Existing Design

This decision extends, not replaces, the existing vision:

- **Knowledge-as-Progression (Big Yes #3)** is Pillar 1. It now explicitly spans multiple biomes — systemic intuition transfers, specific recipes don't.
- **The Slow Bloom (Target Moment #3)** happens within each biome AND across biomes. "This desert garden is better than my last desert garden" AND "My temperate gardens improved because I learned water conservation in the desert."
- **The Canonical Question** gains a second dimension: not just "does ecological discovery drive one more garden?" but "does ecological *variety* drive one more biome?"
- **MVP scope remains temperate-first.** But the architecture should anticipate biome variation from the start, so that adding desert or wetland later is a content task, not a rewrite.
- **Biomes are no longer auto-P3.** They are a core part of the replayability model. The *implementation* of additional biomes is post-MVP, but the *design* accounts for them now. Sim architecture, visual systems, and the autonomous loop should all be biome-aware.
