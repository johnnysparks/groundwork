# Weather

Periodic rain and drought events test ecosystem resilience and create dramatic garden-wide moments.

## States

| State | Duration | Effect |
|-------|----------|--------|
| Clear | 150-250 ticks | No special effects |
| Rain | 30-50 ticks | +30 water to ~20% of surface / 3 ticks, +15 to soil below |
| Drought | 40-70 ticks | -8 surface water / 5 ticks, -3 shallow soil / 5 ticks |

## Transitions (deterministic RNG)

```
Clear -> Rain (30%) | Drought (15%) | Clear (55%)
Rain -> Clear (100%)
Drought -> Rain (40%) | Clear (60%)
```

Drought often ends with rain — creating a dramatic "relief" moment.

## Timing
- Initial grace period: 500 ticks (clear) — protects early game
- Typical cycle: ~200-400 ticks between events
- Full day (100 ticks) = 10 seconds at default speed

## Player Impact
- **Rain:** Burst of growth across the garden. Plants near surface benefit most.
- **Drought:** Surface water evaporates, shallow-rooted plants stress. Deep-rooted plants survive.
- **Strategic:** Plant near water for drought resilience. Deep root species (oak, pine) handle drought better.

## WASM Exports
- `get_weather_state()`: 0=Clear, 1=Rain, 2=Drought
- `get_weather_duration()`: ticks remaining in current state
- JS can use these for rain particles, palette shifts, HUD indicators
