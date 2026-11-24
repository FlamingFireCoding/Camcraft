## #3671 - 3D Pocket: inter-level rapid stops at feed height above prior floor

**What was decided.** The descent starting plane calculation `from(lvl, a, ramp)`
in `camcraft-motion::machining3d::descent` now adds `ap.plunge_height` (the feed
height) above `mat_top(lvl, a)` on later levels as well as on the first level,
capped under the traverse plane `lvl.tz`. Rapids between levels stop at this feed
height above the prior cut floor, and the remaining descent to the next cut level
is fed (e.g. helical feed or linear ramp).

**Why.** When 3D Pocket descends between levels, rapiding straight down to the floor
of the level above with zero clearance creates collision risks with chips or tool
length tolerances. Standard CAM practice stops the rapid at feed height above the
floor and feeds the rest. The descent logic already applied `plunge_height` on the
first level and in `feed_down`, but `from` returned bare `a.prev_z` on subsequent
levels.

**Considered and dropped.**
- Setting a hardcoded 2.0 mm margin: dropped in favor of reading the operation's
  configured `plunge_height` (feed height).
- Modifying each caller in `ladder.rs`: dropped because `from` is the single source
  of truth for the descent starting plane across all entry arms (helix, ramp, arc).

**Cost to change later.** Small: logic is centralized in `descent::from`.
