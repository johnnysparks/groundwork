# Water System

Water is the fundamental resource. Without it, nothing grows. With too much, areas flood.

## Sources
- **Central spring:** 4x4 pool at grid center, refilled to 255 every tick (permanent)
- **Rain events:** adds +30 water to ~20% of surface every 3 ticks during rain (30-50 tick duration)

## Flow Mechanics
- **Gravity:** water moves down, max 32 scaled units/tick transfer
- **Lateral spread:** when can't flow down, spreads to lower-water neighbors at (diff/5), max 8 scaled units
- **Soil absorption:** soil wicks water from adjacent Water voxels (rate depends on drainage_rate)
- **Uses snapshot buffering** to prevent iteration-order artifacts

## Root Absorption
- Roots extract water from adjacent Soil voxels (max 4 scaled units/tick)
- **Competition:** transfer divided by number of root voxels neighboring each soil cell
- **Root water decay:** roots without adjacent wet soil (water > 20) lose -2 water/tick
- This enforces water dependency — removing water from the garden eventually kills plants

## Weather Integration
See [Weather](weather.md) for rain/drought cycles.

## Balance Notes
- Seed germination requires water >= 30 (own or adjacent)
- Pioneer succession requires soil water >= 20 (or >= 5 near DeadWood)
- Root water decay rate (-2/tick) means isolated roots dry out in ~100 ticks
- The spring provides infinite water at the center — plants near the spring always thrive
