# Soil System

Soil is a 6-byte parallel grid tracking composition. It evolves over time based on roots, water, and ecological activity.

## Composition (per cell)

| Component | Range | Effect |
|-----------|-------|--------|
| sand | 0-255 | Fast drainage, low retention |
| clay | 0-255 | Slow drainage, high water retention, high nutrient capacity |
| organic | 0-255 | Decomposed matter, builds near roots |
| rock | 0-255 | Stone fragments, weathers into clay when wet |
| ph | 0-255 | 0=pH 3.0 (acid), 128=pH 6.0, 255=pH 9.0 |
| bacteria | 0-255 | Microbial activity, grows in moist organic soil |

## Derived Properties
- `drainage_rate` = (sand*2 + rock) / 3
- `water_retention` = (clay*2 + organic) / 3
- `nutrient_capacity` = (clay + organic*2 + bacteria) / 4
- `is_compacted` = clay > 200 && organic < 30 (blocks root growth)

## Terrain Initialization (depth layers)

| Layer | Depth | Dominant | Character |
|-------|-------|----------|-----------|
| Rocky | Near stone floor | rock=200 | Poor, drains fast |
| Clay subsoil | 0.5m below surface | clay=200 | Slow drainage, holds water |
| Transition | 0.3-0.5m | Mixed | Balanced |
| Loam topsoil | 0-0.3m | sand=100, clay=80, organic=80 | Ideal for growth |
| Peat (near spring) | Near water | organic=220, bacteria=100 | Very fertile |
| Sandy (edges) | Grid margins | sand=200 | Challenging, drains fast |

## Evolution (runs every 10 ticks)

### Organic Matter
- **Near roots:** +10 per adjacent root per cycle (capped at 255)
- **No roots nearby:** -1 per cycle (slow decay)

### Bacteria
- **Moist + organic-rich:** grows (+10/cycle if water > 50, organic > 30)
- **Dry soil:** dies (-20/cycle if water < 10)
- **Dense canopy (carrying capacity):** -3/cycle if 4+ adjacent roots

### pH Drift
- **Pine allelopathy:** -5/cycle near pine roots (pH < 40 inhibits non-tolerant seeds)
- **Organic decomposition:** -1/cycle if organic > 100
- **Rock buffering:** +1/cycle if rock > 50 (resists acidification)

### Rock Weathering
- Wet soil: rock -1, clay +1 per cycle if water > 30 (very slow, long-term)

### Grass/Clover Soil Binding
- Adjacent grass (species 10) or clover (species 11) roots: clay +1/cycle
- Improves water retention over time

### Nutrient Generation
- Bacteria decompose organic matter: nutrient generation rate = (bacteria * organic) / 6400
- Nutrients cap at `nutrient_capacity` for the soil type
- These nutrients feed seed germination (soil_bonus in growth rate)
