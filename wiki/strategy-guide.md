# Strategy Guide: Reading Your Garden

This guide captures the lessons a player learns through observation. Every visible change in the garden has a cause you can trace — this page teaches you to read those signals.

## Visual Health Indicators

### Foliage Color

Leaf color tells you two things: **species identity** and **health status**.

| Species | Healthy Color | Notes |
|---------|--------------|-------|
| Oak | Deep warm forest green | Darkens slightly in shade |
| Birch | Bright spring yellow-green | Lightest of the trees |
| Willow | Silver-sage with blue tint | Distinctly cooler tone |
| Pine | Very dark blue-green | Darkest of all species |
| Fern | Vibrant emerald | Stands out in understory |
| Berry Bush | Warm olive | Earthy tone |
| Holly | Dark pure green | Similar to pine but warmer |
| Wildflower | Pink-purple | Only non-green foliage |
| Daisy | Warm yellow | Bright, easy to spot |
| Moss | Dark muted olive | Hugs the ground |
| Grass | Bright fresh green | Carpets clearings |
| Clover | Yellow-green | Slightly warmer than grass |

**Stress tint:** When a plant's health drops, its foliage shifts toward amber/brown. The red channel increases while green and blue fade. A fully amber canopy means the tree is near death. A healthy canopy is vivid species-colored green.

### Growth Stages

Plants grow visibly through stages. Knowing what to expect helps you gauge progress:

| Stage | What You See | When It Happens |
|-------|-------------|-----------------|
| Seed | Small mound on soil | Immediately after planting |
| Seedling | Tiny trunk stub + root below | ~1.7 seconds (17 ticks) |
| Sapling | Short trunk + first leaf shell | ~3 seconds (first green!) |
| YoungTree | 2/3 height, branches forming, canopy fills | Water + light >= 500 |
| Mature | Full height, dense crown, seeds drop | Water + light >= 3000 |
| OldGrowth | Same as Mature but more seed rain | Age >= 1200 ticks |
| Dead | Brown DeadWood | Health depleted |

Growth between stages is **gradual** — trunk extends upward from the ground, then leaves fill in from the crown downward. New branches sprout leaves continuously as they grow.

### DeadWood

Brown voxels replacing living wood. DeadWood is not waste — it's the start of the next cycle. Seedlings nearby germinate faster (nurse log effect). Beetles accelerate decomposition into nutrient-rich soil.

---

## Weather Events

### Rain

**What happens:** Water splashes across ~20% of the surface every 3 ticks for 30-50 ticks. Soil below gets +15 moisture.

**What you see:** Burst of green. Plants that were stressed brighten up. Seedlings accelerate. Surface pools form.

**Strategic takeaway:** Rain benefits shallow-rooted plants most (groundcover, flowers, seedlings). Deep-rooted trees near the spring barely notice — they already had water.

### Drought

**What happens:** Surface water evaporates (-8 every 5 ticks). Shallow soil dries (-3 every 5 ticks). Lasts 40-70 ticks.

**What you see:** Surface water shrinks. Foliage on exposed plants shifts toward amber. Seedlings and saplings die first (3-4x vulnerability). Mature trees near the spring stay green.

**Strategic takeaway:**
- **Plants near the spring survive.** The spring refills every tick — roots that reach it never run dry.
- **Deep roots protect against drought.** Oak (30 voxel root depth) and pine (30 voxels) access water below the drought-affected surface layers.
- **Willow is drought-vulnerable.** Despite being a "water specialist," willow needs high water intake (threshold 80). Without it, the 2x growth bonus disappears and willow struggles.
- **Holly is drought-proof.** Low water need (threshold 40) + slow growth rate = survives conditions that kill everything else.

### Drought-to-Rain Relief

40% of droughts end with rain. This creates the most dramatic visual moment in the game — an amber-stressed garden suddenly floods with water and green returns. Plants with surviving roots recover fastest.

### The Grace Period

No weather events for the first 500 ticks (~50 seconds). This protects your initial planting. After that, expect an event every 200-400 ticks.

---

## Competition Signals

### "My Seedlings Keep Dying"

**Cause:** Youth vulnerability. Seedlings take 4x stress damage, saplings 3x. In a crowded garden, shade + root competition kills young plants before they can establish.

**What to do:** Space your plantings. Seeds within 6 voxels of an existing trunk won't even germinate (territorial suppression). Give each tree room to develop roots and canopy before planting neighbors.

### "One Tree Is Thriving, the Others Are Dying"

**Cause:** Root water competition. Roots sharing the same soil cells split the available water. The tree with more roots or better placement near water wins. Losers' health drops and foliage ambers.

**What to do:** This is natural thinning — it's how forests work. The survivors will grow faster with less competition. Watch which tree is winning and support it with water, or remove the loser with the shovel to free resources.

### "Nothing Grows Under My Oak"

**Cause:** Shade competition. A mature oak's canopy blocks most light. Sun-loving species (pine, daisy, grass with shade tolerance > 100) wither underneath.

**What to do:** Plant shade-lovers under canopy. Fern (tolerance 30), moss (20), and holly (40) actually get a 1.5x growth bonus in moderate shade. This creates a natural layered forest: oak canopy > fern mid-story > moss ground cover.

### "Nothing Grows Near My Pine"

**Cause:** Pine allelopathy. Pine roots acidify surrounding soil (pH drops to <40), halving growth rate for most species.

**What to do:** Only pine, fern, and moss are acid-tolerant. Plant these near pines — fern especially thrives (shade-tolerant + acid-tolerant). Everything else should be planted well away from pine territory.

---

## Species Strategies

### The Fast Start: Birch

Birch grows 1.5x faster in open ground (pioneer vigor). It's your fastest tree — but the bonus disappears once other trunks appear within 8 voxels. Use birch to establish the first canopy, then plant oaks and willows nearby as successors.

### The Anchor: Oak + Clover

Plant clover around your oak's base. When 3+ clover leaf voxels sit within 5 voxels of the oak's root position, the oak gets a 1.5x growth boost (nitrogen handshake). This is the game's signature synergy — the foundation of every productive garden.

### The Water Engine: Willow + Stream

Place willows near the central spring or stream bed. With water_intake > 50, willows grow at 2x rate. Their wide crowns (1.5m radius, widest in the game) create excellent shade canopy for fern and moss underneath.

### The Pollinator Loop: Flowers + Trees

Plant a cluster of 5+ wildflowers or daisies. This doubles pollinator (bee/butterfly) spawn rates. Each pollinator within 10 voxels of a tree gives +0.005 health recovery (max +0.02 from 4 pollinators). Stressed trees near flower meadows recover.

### The Bird Express: Berry Bush Network

Berry bushes attract birds. Birds pick up seeds from nearby trees and drop them 10-20 voxels away — plus fertilizing the drop site (+3 nutrients, +2 organic). A berry bush near an oak creates a slow, reliable spread of oak seedlings across the garden.

### The Pine Fortress

Pine is slow (0.7x growth) but creates exclusive territory through soil acidification. A mature pine surrounded by fern and moss is a self-sustaining biome that no other tree can invade. Use pine to "claim" a corner of the garden.

### The Survivor: Holly

Holly grows slowly (0.6x) but handles shade and drought better than anything else. Plant holly where nothing else will grow — dry hilltops, deep shade. It won't impress, but it won't die.

---

## Recovery: What Happens When Things Go Wrong

### Overplanting (Too Dense)

Natural thinning kicks in. Young plants die first (4x vulnerability). Survivors get more water and light. After 300-400 ticks, you'll have fewer but healthier plants. Dead trees become DeadWood, which nurtures the next generation.

### Flooding (Too Much Water)

Water flows downhill and spreads laterally. Excess pools in low areas. Seeds can't germinate in water cells. But nearby soil absorbs the water, and roots benefit from saturated soil. Flooding is rarely fatal — it just slows growth in the pooled area.

### Drought Damage

Surface plants amber and die. But roots underground still hold water for ~50 ticks (root water decays at -4/tick from max ~200). If rain returns, dead trees with wet roots revive: +0.006 health/tick, then snap back to Sapling at health 0.3. This means **drought damage is reversible** if you act quickly with the watering can.

### Bare Patch

Leave it. Pioneer succession activates every 50 ticks: bare moist soil grows moss, moss enables grass, grass enables wildflowers. Within 200 ticks, a bare patch is a meadow. Near DeadWood, succession happens even in drier conditions (moisture threshold drops from 20 to 5).

### Pine Poison

If pine has acidified an area, you can't just plant anything. Remove the pine (shovel) and wait — soil pH recovers over time as soil_evolution normalizes pH toward 80. Or embrace it: plant fern and moss, which thrive in acid soil.

---

## The Idle Garden

A mature garden left running shows visible activity:

| Timeframe | What Happens |
|-----------|-------------|
| Every 3 ticks | Water flows, soil absorbs moisture |
| Every 5 ticks | Fauna move, gnome-fauna proximity updates |
| Every 10 ticks | Fauna effects (beetles decompose, birds spread seeds) |
| Every 20 ticks | New fauna spawn checks at 9 garden sample points |
| Every 30 ticks | Tree health colors refresh (foliage brightens or ambers) |
| Every 50 ticks | Pioneer succession samples 20 positions for new growth |
| ~200-400 ticks | Weather event (rain burst or drought stress) |

A healthy idle garden generates ~34 visible events per 600 ticks. If you stop clicking and watch, you'll see: pollinators drifting, trees gaining leaves, seedlings appearing where birds dropped seeds, and bare patches filling with moss.

---

## The Longest Chain

The deepest ecological interaction you can build:

```
Plant clover near oak roots
  -> Nitrogen handshake: oak grows 1.5x faster
    -> Oak develops full canopy
      -> Canopy shades the ground below
        -> Fern thrives in shade (1.5x canopy effect)
          -> Fern holds moisture, soil stays wetter
            -> Moss colonizes nearby (pioneer succession)
              -> Dense groundcover triggers Tier 1 unlock
                -> Flowers become available
                  -> Wildflower cluster attracts bees
                    -> Bees boost oak health (+0.02/tick)
                      -> Oak reaches OldGrowth
                        -> 2x seed rain creates oak offspring
                          -> Same-species roots share health (mycorrhizal network)
```

13 steps from a single clover placement. This is the "twentieth hour" discovery — designing ecosystems where every species serves a role and the garden sustains itself.

---

## Quick Reference: What Kills Plants

| Threat | Severity | Who Dies First | Who Survives |
|--------|----------|---------------|-------------|
| No water | High | Seedlings (4x damage), shallow-rooted species | Deep-rooted trees near spring |
| No light (shade) | Medium | Sun-lovers: pine, daisy, grass | Shade-lovers: fern, moss, holly |
| Drought event | High | Surface plants, seedlings | Deep roots, well-watered trees, holly |
| Root competition | Medium | Smaller/newer trees | Established trees with more root mass |
| Pine allelopathy | Area denial | Everything except pine/fern/moss | Pine, fern, moss |
| Crowding | Gradual | Youngest plants (4x vulnerability) | Best-positioned survivors |
| Soil bacteria decline | Slow | Dense forest stands | Spaced-out plantings |
