# Backlog — Groundwork

_Last updated: 2026-03-11T12:00:00 by Manager_

## P0 — Blocks core proof or makes build unusable

### CLI-01: Fix no-terminal panic on default invocation
- **Owner:** tools
- **Why:** Running `groundwork` with no args in a non-TTY context (agent sandbox, CI, piped output) panics with a stack trace. This blocks agent play entirely since agents don't have terminals.
- **Done when:** `groundwork` with no args in a non-TTY context prints help instead of panicking.
- **Dependencies:** none
- **Risk:** low
- **Scope:** Core agent play path — must work for MVP.

### CLI-02: Add legend to `view` output
- **Owner:** tools
- **Why:** Players (human and agent) cannot interpret the ASCII grid without memorizing symbols. The legend is in CLAUDE.md but not in the tool output. This directly blocks "clarity of cause and effect."
- **Done when:** `view` prints the ASCII legend below the grid: `. air  ~ water  # soil  % wet  @ stone  * root`
- **Dependencies:** none
- **Risk:** low
- **Scope:** Readability is a core design constraint.

## P1 — Strongly improves clarity, feel, or core loop

### CLI-03: Add axis labels to `view`
- **Owner:** tools
- **Why:** After `place water 20 20 16`, the player has no way to verify the placement visually without counting characters. Coordinate labels make the grid navigable.
- **Done when:** `view` prints X tick marks across the top (every 10) and Y tick marks down the left side (every 10).
- **Dependencies:** none
- **Risk:** low — minor formatting concern if terminals are narrow
- **Scope:** Cause-and-effect readability.

### CLI-04: Show change summary after `tick`
- **Owner:** tools
- **Why:** `tick 100` prints only "Tick: 121 (+100)". The player has no idea what happened. This makes ticking feel hollow and forces a `view` after every tick.
- **Done when:** `tick` prints a one-line summary of material count changes (e.g., "water: +32, wet soil: +48").
- **Dependencies:** none
- **Risk:** low
- **Scope:** Core loop feel — confirms the sim is alive.

### CLI-05: Warn on out-of-bounds Z in `view`
- **Owner:** tools
- **Why:** `view --z 50` silently renders an empty grid. Players waste time wondering why their world is empty.
- **Done when:** `view` clamps Z to valid range (0..GRID_Z-1) and prints a warning if the requested Z was out of bounds.
- **Dependencies:** none
- **Risk:** low
- **Scope:** Friction reduction.

### CLI-06: Show value ranges in `inspect`
- **Owner:** tools
- **Why:** "water_level: 115" is meaningless without knowing the range. Players can't judge whether a voxel is nearly full or nearly empty.
- **Done when:** `inspect` prints values as fractions, e.g., "water_level: 115/255".
- **Dependencies:** none
- **Risk:** low
- **Scope:** Cause-and-effect readability.

## P2 — Valuable but not required for MVP

### CLI-07: Add wet-soil count to `status`
- **Owner:** tools
- **Why:** Wet soil is a key visual indicator but `status` only counts raw materials. Seeing how many soil cells are wet gives a quick sense of water penetration.
- **Done when:** `status` includes a "wet soil" count (soil cells with water_level > 100).
- **Dependencies:** none
- **Risk:** low

## P3 — Future / expansion

_(none currently)_
