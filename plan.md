# Soil Simulation Plan

## Vision
Soil as a living, evolving system with distinct compositions that affect drainage, nutrient retention, and plant viability. Start "Minecraft crafty" with clear soil types, then layer in evolution mechanics so soil changes over time based on water, roots, bacteria, and organic decomposition.

## Architecture: Separate SoilGrid Resource

Keep the 4-byte Voxel untouched (hot path for water/light stays cache-friendly). Add a parallel `SoilGrid` resource with per-cell soil composition data. Only meaningful for `Material::Soil` voxels.

### New struct: `SoilComposition` (6 bytes per cell)

```rust
pub struct SoilComposition {
    pub sand: u8,       // 0-255, coarse particles — high = fast drainage, low nutrient retention
    pub clay: u8,       // 0-255, fine particles — high = slow drainage, high water retention
    pub organic: u8,    // 0-255, decomposed organic material — increases over time from roots/decay
    pub rock: u8,       // 0-255, stone fragments — inherited from nearby stone, decreases via weathering
    pub ph: u8,         // 0-255 mapped to pH 3.0-9.0 (128 = neutral ~6.0)
    pub bacteria: u8,   // 0-255, microbial activity — grows with moisture + organic matter
}
```

Memory: 108K × 6 = 648KB. Acceptable alongside the 432KB VoxelGrid.

### Derived properties (computed, not stored)

- **Drainage rate**: `(sand * 2 + rock) / 3` — sandy/rocky soil drains fast
- **Water retention**: `(clay * 2 + organic) / 3` — clay/organic holds water
- **Nutrient capacity**: `(clay + organic * 2 + bacteria) / 4` — richer soil feeds plants better
- **Compaction**: `clay > 200 && organic < 30` — compacted clay blocks root growth

### Preset soil types (Minecraft-crafty starting point)

| Type | Sand | Clay | Organic | Rock | pH | Bacteria | Where |
|------|------|------|---------|------|----|----------|-------|
| Rocky | 40 | 20 | 10 | 200 | 128 | 5 | Z=5-7 (near stone) |
| Clay | 30 | 200 | 20 | 30 | 110 | 20 | Z=8-10 (subsoil) |
| Sandy | 200 | 30 | 15 | 40 | 135 | 15 | patches/edges |
| Loam | 100 | 80 | 80 | 30 | 128 | 60 | Z=13-15 (topsoil) |
| Peat | 40 | 40 | 220 | 10 | 80 | 100 | near water |

## Implementation Steps

### Step 1: Add `SoilComposition` + `SoilGrid` types
- New file: `crates/groundwork-sim/src/soil.rs`
- `SoilComposition` struct with the 6 fields
- `SoilGrid` resource (flat Vec, same indexing as VoxelGrid)
- Preset constructors: `SoilComposition::rocky()`, `::clay()`, `::sandy()`, `::loam()`, `::peat()`
- Derived property methods: `drainage_rate()`, `water_retention()`, `nutrient_capacity()`

### Step 2: Generate varied default terrain
- Modify `VoxelGrid::new()` logic — now also populates `SoilGrid`
- Add `SoilGrid::new()` with depth-based composition:
  - Near stone layer: rocky soil
  - Middle layers: clay-heavy subsoil
  - Top layers: loam
  - Noise-based sandy patches and peat near the water spring
- Insert `SoilGrid` as resource in `create_world()`

### Step 3: Soil-aware water absorption
- Modify `soil_absorption` system to read `SoilGrid`
- Replace flat "absorb 2 per tick" with drainage-rate-based absorption
- High clay = absorbs slowly but retains well; high sand = absorbs fast but drains through
- Add soil-to-soil water diffusion (wet soil spreads water to drier adjacent soil, rate based on composition)

### Step 4: Soil evolution system
- New system: `soil_evolution` (runs after root_water_absorption, before light_propagation)
- **Organic matter**: increases when adjacent to Root voxels (+1/tick), slowly decomposes without roots (-1 every 10 ticks)
- **Bacteria**: grows when water_level > 50 AND organic > 30 (+1/tick), dies when dry (-2/tick)
- **pH drift**: organic decomposition makes soil more acidic (ph -= 1 every 20 ticks if organic > 100); mineral weathering from rock fragments buffers toward neutral
- **Rock weathering**: rock fragments slowly decrease (-1 every 50 ticks if water_level > 30), converted to clay (+1 every 50 ticks)

### Step 5: Seed growth reads soil quality
- Modify `seed_growth` to check `SoilComposition` of adjacent soil
- Seeds grow faster in nutrient-rich soil (loam/peat), slower in rocky/sandy
- Very acidic or compacted soil blocks growth entirely

### Step 6: Save format v3
- Bump version to 3
- After focus block, write soil data: 108K × 6 bytes
- Backward compatible: v1/v2 files load with default soil composition generated from depth

### Step 7: Display soil types (CLI + TUI parity)
- `inspect` command shows soil composition details
- Different ASCII chars or colors for soil types:
  - `#` generic soil (current)
  - Could show composition in inspect panel rather than changing the grid char (keeps display clean)
- `status` command includes soil composition summary

### Step 8: Tests
- Unit tests for each derived property calculation
- Soil absorption varies by composition
- Soil evolution over 100 ticks shows organic increase near roots
- Bacteria population dynamics
- Rocky soil weathers into clay over time
- Save/load round-trip with soil data
- Backward compat: v2 files load with generated soil

## What This Does NOT Include (future)
- Specific plant species with soil preferences (separate agent's domain)
- Mycorrhizal networks (future sprint)
- Soil temperature
- Erosion from water flow
- Composting mechanics
