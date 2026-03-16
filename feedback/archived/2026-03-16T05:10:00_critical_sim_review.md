# Feedback: Critical Sim Review — Where's the Drama?

**Date:** 2026-03-16T05:10:00
**Build:** Post 32-sprint session, full game loop
**Reviewer:** Player + designer (joint review of live play + screenshots)

---

## The Problem

The garden looks like a mess. Everything grows on top of everything else. There's no spatial logic, no competition, no tension. A randomly-placed garden looks identical to a carefully-planned one. The game rewards clicking, not thinking.

## Specific Failures

### 1. Trees grow on top of each other
The zone-fill tool dumps 8+ seeds in a tight 4-voxel radius. Multiple trees germinate in the same spot and their canopies merge into one indistinguishable green blob. There's no visual distinction between "one healthy oak" and "five stunted trees fighting for the same space."

**Expected:** Trees need territory. A mature oak should suppress seedlings within its crown radius. Only one tree should dominate a given area.

### 2. No light competition
Tall trees cast no meaningful shadow on shorter plants. A fern under an oak canopy gets the same light as a fern in open ground. The light system propagates top-down but doesn't create real shade zones that affect growth.

**Expected:** Canopy shade should slow or stop understory growth. Only shade-tolerant species (fern, moss) should survive under trees. Sun-loving flowers should die in deep shade.

### 3. No water competition
Every plant's roots absorb water independently. An oak with 300 roots next to a wildflower with 3 roots doesn't steal the wildflower's water. There's no resource scarcity.

**Expected:** Roots should compete for the same water. A thirsty oak near a small flower should drain the soil dry around the flower, stressing or killing it. The player must space plants based on water needs.

### 4. No crowding death
Nothing ever dies from competition. Once a seed sprouts, it lives forever (or until the player digs it). There's no natural thinning, no "survival of the fittest" in a crowded zone.

**Expected:** Plants in crowded conditions (low light, low water, too many neighbors) should decline in health and eventually die, leaving deadwood. This creates natural spacing and teaches the player about carrying capacity.

### 5. No visual stress indicators
There's no way to tell if a plant is thriving vs. struggling. Every plant looks the same shade of green regardless of health. A waterlogged root system and a drought-stressed one are visually identical.

**Expected:** Stressed plants should show it — yellowed/sparse foliage, thinner trunks, drooping canopy. Thriving plants should look lush and full. This is the visual feedback that teaches placement strategy.

### 6. Zone-fill is too dumb
The zone tool fills every cell in a radius with seeds, regardless of what's already there. It doesn't check for existing plants, doesn't space seeds intelligently, doesn't consider light/water availability.

**Expected:** Smart zone placement: space seeds 3-5 voxels apart (species-appropriate), skip cells near existing mature plants, prefer cells with adequate water. The zone tool should be an advisor, not a stamp.

## What Good Looks Like

A well-played garden at tick 300 should show:
- **Distinct trees** with clear spacing — each tree has its own territory
- **Canopy layers** — tall oaks with ferns underneath, flowers in sunny gaps
- **Visible competition** — a struggling birch next to a dominant oak (yellowish foliage, smaller canopy)
- **Dead zones** — areas where crowded plants died and left deadwood, with beetles decomposing it
- **Strategic water access** — plants clustered near water sources, bare dry patches where nothing can survive
- **Pioneer succession in gaps** — moss/grass colonizing the spaces between mature trees

## Priority Order

1. **Smart zone spacing** (sprint-sized) — seeds spread 3-5 voxels apart, skip occupied cells
2. **Crowding death** (sprint-sized) — plants with low health from shade/drought die after sustained stress
3. **Light competition** (sprint-sized) — canopy blocks light below, shade-intolerant species suffer
4. **Water competition** (needs verification — may already work via root absorption)
5. **Visual stress** (sprint-sized) — foliage color shifts yellow/brown based on tree health

## The Test

After these fixes, plant 10 oaks in a tight cluster. After 200 ticks, only 2-3 should survive — the rest should have died from shade/water competition, leaving deadwood. The survivors should be visibly healthier (greener, fuller) than a lone oak that had no competition.
