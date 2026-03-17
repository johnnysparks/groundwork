# Water System

Water is the fundamental resource. Without it, nothing grows. With too much, areas flood. Players don't place water directly — they **dig irrigation channels** from natural water sources using the shovel. See the [Irrigation Decision](../decisions/2026-03-17T12:00:00_irrigation_replaces_watering_can.md).

## Sources
- **Pond:** Spring-fed pool at the top of the slope (`POND_X`, `POND_Y`), refilled to 255 every tick (permanent). Water flows downhill from here.
- **Rain events:** adds +30 water to ~20% of surface every 3 ticks during rain (30-50 tick duration)

## Flow Mechanics
- **Gravity:** water moves down, max 32 scaled units/tick transfer
- **Lateral spread:** when can't flow down, spreads to lower-water neighbors at (diff/5), max 8 scaled units
- **Soil absorption:** soil wicks water from adjacent Water voxels (rate depends on drainage_rate)
- **Uses snapshot buffering** to prevent iteration-order artifacts

## Root Absorption
- Roots extract water from adjacent Soil voxels (max 4 scaled units/tick)
- **Competition:** transfer divided by number of root voxels neighboring each soil cell
- **Root water decay:** roots without adjacent wet soil (water > 20) lose -4 water/tick
- This enforces water dependency — removing water from the garden creates visible drought stress within ~50 ticks

## Weather Integration
See [Weather](weather.md) for rain/drought cycles.

## Balance Notes
- Seed germination requires water >= 30 (own or adjacent)
- Pioneer succession requires soil water >= 20 (or >= 5 near DeadWood)
- Root water decay rate (-4/tick) means isolated roots dry out in ~50 ticks (5 seconds)
- The pond provides infinite water at the top of the slope — plants near it always thrive

## Irrigation Design

Inspired by Timberborn's terrain-shaping water management. Players dig channels from the pond to their plants using the shovel — the terrain *is* the irrigation system. No watering can tool.

**Player flow:**
1. Start near the pond — seeds planted close get water naturally
2. Want to expand? Dig a channel from the pond toward new planting zones
3. Use soil tool to build berms and dams to direct flow
4. Discover: willows along channels stabilize banks, moss colonizes damp banks

**Drought tiers:**
- **Tier 1 (Stardew-style):** Plants stop growing when soil moisture drops. No death, just pausing.
- **Tier 2:** After sustained drought, roots lose water (-4/tick), health declines, deadwood appears. Pioneer succession fills gaps = recovery feature.
- The pond is permanent and recovery is always possible — cozy, not punishing.
