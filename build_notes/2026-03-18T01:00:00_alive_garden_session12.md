# Build Notes: Alive Garden Session 12 (Sprints 221-230)

**Date:** 2026-03-18T01:00:00
**Sprints:** 221-230
**Theme:** Fauna behavior particles + weather audio + night atmosphere

## Sprint Log

| Sprint | Feature | Type | Key Files |
|--------|---------|------|-----------|
| 221 | Sunbeam shafts during daytime | Web | particles.ts, main.ts |
| 222 | Garden whisper harmonic in dense gardens | Web | sfx.ts, main.ts |
| 223 | Pollen count visible in sunbeams | Web | particles.ts, main.ts |
| 224 | Bee waggle dance particles | Web | particles.ts, main.ts |
| 225 | Butterfly pollen trail | Web | particles.ts, main.ts |
| 226 | Bird nesting particles | Web | particles.ts, main.ts |
| 227 | Moonbeam shafts at night | Web | particles.ts, main.ts |
| 228 | Rain amplifies frog chorus | Web | ambient.ts, main.ts |
| 229 | Drought cicada drone | Web | ambient.ts, main.ts |
| 230 | Water surface fog wisps | Web | particles.ts, main.ts |

## Key Technical Decisions

- **Pollen in sunbeams (223)**: `activePollinators` hoisted to closure scope so sunbeam block can read pollinator count. `emitSunbeam()` gains optional `pollen` param (0-1) that shifts color deeper gold and increases horizontal drift. Emission rate scales 1.5→3.5/sec with pollinators.

- **Bee waggle dance (224)**: New `emitBeeWaggle()` emits 2 particles in opposing directions to suggest figure-8 pattern. Honey-gold color. Only when bee is in Acting state (pollinating).

- **Butterfly pollen trail (225)**: Soft yellow motes that float downward behind butterflies during Seeking/Acting. Lower rate than bee waggle (0.6/sec vs 1.2/sec) — more ephemeral.

- **Bird nesting (226)**: Brown-tan twig particles tumble from perched birds (Idle state). Comes from canopy height. Very low rate (0.3/sec) — occasional not constant.

- **Moonbeam shafts (227)**: Night counterpart to sunbeams. Cool blue-white particles, slower drift, dimmer. Active during 0.85-1.0 and 0.0-0.15 (deep night). Requires foliage > 500 for canopy filtering effect.

- **Rain frog boost (228)**: `setFrogChorus()` gains optional `raining` parameter. Frogs now croak during daytime rain (not just dusk/night), and at higher gain (0.035 vs 0.02).

- **Cicada drone (229)**: New `createCicadaDrone()` — two detuned sawtooth oscillators through narrow bandpass at 4300Hz with 6Hz amplitude modulation pulse. Active during drought daytime (0.25-0.75). Very quiet (0.012 gain).

- **Water fog (230)**: `emitWaterFog()` creates long-lived (3-5s) cool blue-white particles hovering above GROUND_LEVEL. Dawn only (0.10-0.30), requires water > 20. Slow horizontal drift suggests evaporation.

## Cross-System Connections Added This Session

| System A | System B | Connection |
|----------|----------|------------|
| Pollinators | Sunbeams | Pollen count changes beam density and color |
| Bees | Particles | Waggle dance shows active pollination |
| Butterflies | Particles | Pollen trail makes movement path visible |
| Birds | Particles | Nesting behavior from perched state |
| Moonlight | Particles | Moonbeams mirror sunbeams at night |
| Rain | Frog audio | Frogs amplified and extended during rain |
| Drought | Audio | Cicada drone marks drought periods |
| Water | Particles | Fog wisps above water at dawn |

## Infrastructure

- Fixed `scripts/sync-check.sh` POSIX compatibility (macOS `sed` doesn't support `\s`, replaced with `[[:space:]]`)
- Ran `cargo fmt` on new sync guard tests from PR #96
- All new code passes sim↔web sync check (0 errors, 5 warnings for unused exports)

## Total Sprint Count: 230
