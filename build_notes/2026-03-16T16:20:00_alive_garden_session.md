# Build Notes: Living Garden Session (Sprints 114-122)

**Date:** 2026-03-16T16:20:00
**Sprints:** 114-122 (9 sprints)
**Theme:** Make the garden more alive, not more polished

## Session Summary

This session transformed the garden from a static ecological simulator into a living, breathing diorama. Every sprint was guided by the principle: "When in doubt, make the garden more alive."

## Sprint Log

| Sprint | Feature | Type | Key Files |
|--------|---------|------|-----------|
| 114 | Denser canopy (8 stubs + 15 nodes/tick) | Sim | tree.rs, systems.rs |
| 115 | Visible squirrels (3D model + glow + animation) | Web | models/fauna.ts, rendering/fauna.ts |
| 116 | Ecological event messages (squirrel caching, pollination) | Web | main.ts |
| 117 | Dusk fireflies (40 golden particles) | Web | rendering/fireflies.ts |
| 118 | Falling leaves (20 ambient canopy particles) | Web | rendering/leaves.ts |
| 119 | Night crickets (procedural chirp chorus) | Web | audio/ambient.ts |
| 120 | Nitrogen handshake visual (green shimmer) | Web | rendering/ecology.ts |
| 121 | Plant hover tooltip (species name on mouseover) | Web | main.ts |
| 122 | Wind-responsive falling leaves | Web | rendering/leaves.ts |

## What the Garden Looks/Sounds Like Now

### Visual Layers (always active)
- Foliage billboard sprites with wind sway
- Falling leaves drifting from canopy (wind-responsive)
- 6 fauna types with 3D models, glow halos, and pollen trails
- Ecology particles: nitrogen shimmer, water absorption, decomposition, pollination
- Seed golden sparkle particles
- Growth burst particles

### Visual Layers (time-of-day)
- Golden hour + blue hour: 40 fireflies with blinking glow
- Rain: 800 droplet particles + gusty wind
- Drought: warm amber fog haze

### Audio Layers
- Always: water spring trickle
- Periodic: bird chirps, bee buzzes (8-20s interval)
- Growth: ascending shimmer sound
- Rain: band-pass noise patter
- Dusk/night: dual-cricket chirp chorus
- Player actions: plant pat, water splash, dig scrunch
- Milestones: warm chime
- Fauna arrivals: bird calls, bee/butterfly buzz

### Interactive
- Hover tooltip: species name + material type on mouse-over
- HUD event feed: weather changes, fauna arrivals, succession stages, ecological interactions
- Contextual ecology tips based on garden state
- X-ray underground view with species-colored roots

## Big Yeses Status

| Interaction | Sim Status | Visual Status |
|-------------|-----------|---------------|
| Nitrogen Handshake | Working (1.5x boost) | Green shimmer particles + tip |
| Pollinator Bridge | Working (health boost) | Pollen trails + event messages |
| Root War | Working (competition) | X-ray colored roots |
| Bird Express | Working (seed dispersal) | Event messages |
| Squirrel Cache | Working (acorn → oak) | 3D model + dig animation + messages |
| Pioneer Succession | Working (moss → grass → flower) | Succession messages |
| Canopy Effect | Working (shade boost) | No explicit visual yet |

## What's Next (Recommended)

1. **Canopy Effect visual** — shade-tolerant species near tall trees should show a subtle indicator (P1 candidate: it's the last Big Yes without visual feedback)
2. **Squirrel trust follow** — gnome-squirrel trust at 180+ should make squirrel visually follow the gnome (fun and rewarding)
3. **Wild plant notification** — "An oak seedling appeared where a squirrel cached!" when spontaneous plants appear
4. **Playtest at golden hour** — many new features (fireflies, crickets, wind leaves) are best experienced at dusk. Screenshot harness starts at noon; consider a golden-hour screenshot angle.
