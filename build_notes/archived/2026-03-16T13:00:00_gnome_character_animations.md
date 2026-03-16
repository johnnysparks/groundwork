# Build Notes — Gnome Character Animations

**Date:** 2026-03-16T13:00:00
**Focus:** Infusing the garden gnome with personality through animations

## What Changed

Complete rewrite of `crates/groundwork-web/src/gardener/gardener.ts` — the gnome billboard sprite now has a full personality system.

### New State Machine

- **Idle** with 6 sub-behaviors: Standing (breathing), LookingAround, Yawning, SittingDown, InspectingPlant, WavingAtCamera, Stretching
- **Walking** with directional lean, arm swing, hat sway, cheerful expression
- **Working** with tool-specific animations (dig pumps for shovel, gentle tipping for watering can, sprinkling for seeds)
- **Celebrating** with 3 tiers: TaskDone (quick hop), QueueEmpty (happy dance with music notes), BigMilestone (arms-up triumph after 10+ task streaks)

### Animatable Body Parts (GLSL Shader)

New shader uniforms drive procedural body part animation:
- **Arms** (`uArmL`, `uArmR`): pivot from shoulders, 0=down to 1=up, with capsule SDF rendering
- **Head tilt** (`uHeadTilt`): head and hat follow together, -1 to 1
- **Eye state** (`uEyeState`): 0=open, 1=blink (closed), 2=happy squint
- **Mouth** (`uMouthState`): 0=neutral line, 1=smile arc, 2=yawn (open circle)
- **Tool in hand** (`uToolType`): shovel, watering can, or seed bag rendered at right hand position
- **Hat sway** (`uHatSway`): independent hat tip with rotation
- **Rosy cheeks** (`uCheekGlow`): warm glow when happy/working hard
- **Sitting** (`uSitting`): body lowers, boots spread, head drops
- **Squash/stretch** (`uSquash`): classic animation principle for bounces and impacts
- **Body lean** (`uLean`): tilts entire billboard for walk lean and dance moves
- **Vertical offset** (`uBodyY`): for sitting/crouching without changing world position

### Personality Behaviors

All animations use smooth interpolation (lerp toward target) for buttery transitions.

**Idle cycle**: After 2-6 seconds of idle breathing, the gnome randomly picks a behavior:
- **LookingAround**: Head swivels left and right, alert eyes
- **Yawning**: Arms stretch up, mouth opens wide, eyes close at peak, Zzz particles
- **SittingDown**: Smooth transition to sitting pose, content smile, occasional Zzz
- **InspectingPlant**: Leans forward, peers down, tiny nods, occasional "!" exclamation
- **WavingAtCamera**: Breaks fourth wall, waves right arm, big smile, happy squint
- **Stretching**: Arms reach high, body elongates, yawn at peak

**Walking personality**: Body leans into direction, arms swing opposite, hat sways with movement, cheerful smile.

**Work animations** vary by tool type:
- Shovel: arm pumps up/down, body leans into digs
- Watering can: gentle tipping motion
- Seeds: sprinkling/scattering motion

**Celebrations trigger automatically:**
- Every 5th completed task: small hop with arms up
- Queue emptied: happy dance with body lean, arm waves, music note particles
- 10+ task streak completed: big triumphant arms-up celebration with sparkles

### Emotion Particle System

GPU-accelerated point sprite system (max 20 particles) floats above gnome's head:
- **Hearts**: emitted when planting seeds
- **Sparkles**: emitted on task completion
- **Sweat drops**: emitted during long work streaks (8+ tasks)
- **Zzz**: emitted during yawning and sitting idle behaviors
- **Music notes**: emitted during happy dance celebration
- **Exclamation**: emitted when inspecting plants

Particles rise and fade with custom shader (type-specific colors and shapes).

### Blink System

Automatic eye blinks every ~3 seconds (with random jitter). Blinking is independent of other animations but yields to behavior-driven eye states (happy squint, closed yawn).

## Architecture Notes

- Same public API: `GardenerSprite` class with `update(dt, elapsed, queue) → GardenTask | null`
- No changes to `main.ts`, `queue.ts`, or `ghosts.ts` needed
- All animation state is smoothly interpolated via lerp for silky transitions
- Emotion particles use Three.js Points with custom shaders (no geometry allocation per particle)

## Build Verification

- `npx tsc --noEmit`: 0 errors
- `npx vite build`: success (652KB bundle)
