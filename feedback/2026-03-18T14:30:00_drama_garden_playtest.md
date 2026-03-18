# Playtest Feedback — Drama Garden (Sprint 340)

**Date:** 2026-03-18T14:30:00
**Build:** main @ 793860d

## Visual Assessment

Screenshots across the full day/night cycle show a lush, active garden:
- **Hero/dawn/noon/golden/blue-hour**: Dense vegetation, multiple trees with overlapping canopy, rain particles, sparkle effects
- **Close-up**: Gnome visible among tree trunks, varied foliage colors (yellow-green canopy, darker groundcover)
- **Water overlay**: Pond visible with water flow patterns
- **X-ray**: Root networks visible underground

## Quantitative Data

- 37,080 leaf voxels on first load (3x Sprint 339)
- 563 trunk voxels (2.7x Sprint 339)
- 488 root voxels
- 6 trees: oak, birch, moss, wildflower, clover + **1 dead pine**
- Dead pine: health_u8=0, stage=Dead — ecological drama visible!

## What's Working
1. **Ecological drama**: Dead pine in the garden shows competition is real
2. **Lush first impression**: 37k leaves = dense, alive garden
3. **Species variety**: mix of tree, groundcover, and flower
4. **Full atmosphere**: rain, particles, day/night, mist
5. **Underground life**: roots visible in x-ray

## What Could Improve
1. **Dead pine visibility**: The dead tree produces deadwood material but it's small (pine at seedling/sapling size when it died). A larger dead tree would be more dramatic.
2. **More groundcover**: The terrain still has a lot of bare soil visible. More moss/grass would make the garden feel lusher.
3. **Player-triggered drama**: The starter garden shows drama but the player needs to see it happen DURING gameplay — plant densely and watch trees die. This requires the player to actually play.

## Status
- P0: none (starter garden and health tinting both working)
- P1: none active
- P2: mobile drag-to-zone, more groundcover, enhanced dead tree visuals
