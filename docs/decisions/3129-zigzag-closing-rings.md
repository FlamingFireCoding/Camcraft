## #3129 - Zigzag levels of the avoid pocket and the 3D pocket close with boundary rings

**What was decided.** Two more pass plans now use
`pocket_zigzag::rest_passes` for Zigzag and Auto, as the plain 2D pocket
does after #3021:
- `pocket_avoid` (2D pocket with "Do not touch"), at each level.
- `machining3d::hsm::plan_level`, the 3D pocket's level plan when the
  strategy is not Adaptive and the level has no rotary seam.

The rows stand 0.01 mm inside the level's cutter-center area. One closed
ring then follows each boundary loop of that area: the outer wall, each
island, and each keep-out edge.

**Why.**
- Rows touch a wall or an island only at their ends, so a cusp stays
  between each two ends over the full depth. The pins measured, before
  the change: 0.755 mm on a rounded corner of the avoid pocket (0.919 mm
  with blended turns), and a 2.0 mm strip on the boss wall of a 3D level.
- `rest_passes` is the plan that the plain 2D pocket and the rest pocket
  already use. One plan for all Zigzag levels keeps the wall result the
  same in all three operations.
- The 3D plan already read Auto as Zigzag (`plan_passes`). The change
  keeps that reading.
- `rest_passes` is made `pub(crate)` and re-exported as
  `zigzag_ringed_passes`, so that `machining3d` reads it through the
  `machining2d` door, as it reads `plan_passes`.

- The rotary wrap (Rotary Offset) uses the same `plan_level`, always
  with a seam. Its levels keep the plain rows. The developed area of a
  wrap level ends at the seam, so a ring on each boundary loop would also
  cut the seam line, at both ends of the strip. That is one physical line,
  cut twice, and not a wall.

**Dropped.**
- Rings on the rotary wrap levels in this PR. They need a ring that
  skips the seam edges. A follow-up issue tracks them.
- A change inside `plan_passes`. It has more callers (the 2D Adaptive
  fallback, the 3D rest pass and shells), and #3130 changes its caller in
  `pocket.rs`. A change at the two call sites is smaller and does not
  overlap #3130.
- A crossing-law post for blended turns in `pocket_avoid` (the plain
  pocket's `hold_crossings`). The blended pin shows no crossing pair, and
  `pocket_avoid` rounds its turns with the level area as the legal region.
  If a crossing shows up later, the post check blocks the program, and the
  law can be added then.

- A raw count of posed moves in the Multi-Axis Rough test
  (`posed <= 20`). The closing ring round the boss adds a pose on each
  ring chord. The test now checks that each pose stays within the ball's
  reach of the boss, which is the intent of the old bound.

**Cost to change later.** Low. Each plan selects `rest_passes` in one
branch. The pins in `crates/motion/tests/pocket_wall_finish_levels.rs`
hold the wall result.
