## #2483 - main is red: two stale fixtures after #468/#2386 and #2450

**What was decided.**

1. *`microarc_test_part::a_wrapped_op_on_the_test_part_posts_wrapped_a`.*
   One hunk of the golden
   `crates/app/tests/golden/nc/tormach/wrap_rotary_parallel_microarc4.nc`
   changes: one turning arc (X-1.5625 Z0.2768 A87.24 to X-1.5 Z0.3394
   A87.67) now posts 8 chords, not 7. Only that hunk is copied from a
   `WRITE_GOLDEN=1` run. The rest of the file stays as it was, so that no
   last-digit rounding of the Windows run enters the golden.
   The cause is commit 8f99bcb4c ("fix(threading): align dip motion
   readers", Refs #419, merged with #468). It changed
   `crates/cl/src/cl/moves.rs` `arc_chords`: when the start and end radius
   differ (`ra != rb`), the angle step also obeys
   `sqrt(8 * tol / hypot(r, 2|rb - ra| / d))`. This arc's two radii differ
   in the last float digits, so the bound applies. For a circle the bound
   is `sqrt(8 * tol / r)`, the small-angle form of the old sagitta law
   `2 * acos(1 - tol / r)`, and a little smaller. So the step is a little
   shorter and the arc gets one more chord. Each chord still stays inside
   the chord tolerance; the end points and the end A are the same. The
   inverse-time F values change with the shorter chords. #2386 made the
   golden on a base before 8f99bcb4c, and the two merges met on main.
2. *`stock_source_holes::framed_stock_chamfer_live_refusal_keeps_hint_without_local_move_number`.*
   The test's ride plane moves from the stock top (Z20) to mid-wall (Z10),
   and the added part's top moves with it. The asserts do not change.
   With a 10-degree tilt, the plane Z20 of the operation frame crosses the
   stock's top face along a line, and that line is part of the section
   loop. That line is not an edge: the corner plane Z19 crosses the same
   top face about 1/sin(10 deg) = 5.8 mm away. The #2435 wall law
   (`machining2d::chamfer_ride::ride_loops`, #2450) refuses such a loop by
   name. That refusal is correct: a cone that rides the line cuts a groove
   in the top face, not a chamfer. Before #2450 the ride took the line
   with no question, and the test pinned that as "previously clear". At
   Z10 the plane and the corner plane cut only the four walls. The walls
   lean 10 degrees, so the two loops stand tan(10 deg) = 0.18 mm apart,
   inside the 1 mm width. The ride is accepted, and the test reaches the
   live refusal that it pins (the part added under the chamfer).
3. *`guide_projects::demo_projects_build_and_load_with_valid_paths`.* No
   change. It passes on 2c789ed6d (after #2499).

**Why.** The two failures are fixtures that the merged law changes made
stale. Neither is a defect in the posted G-code or in the refusal. No
assert is relaxed and no refusal is made wider.

**Considered and dropped.**
- *Exempt a stock-source chamfer from the wall law.* It would make the
  old fixture pass. It would also post again a cone that rides a line
  that is no edge. Dropped.
- *A zero-degree tilt.* It would pass, but the test would not exercise a
  turned operation frame any more. Dropped.
- *Compare `ra` and `rb` with a tolerance in `arc_chords`.* A circle would
  then keep the old chord count. But it changes motion code under the
  safety floor in a crate that #2483 does not touch, and the new chords
  are not wrong: they are only one more. Dropped from this PR; filed as a
  P7 follow-up.
- *Write the whole golden again from the Windows run.* It brings many
  last-digit changes that are inside the compare tolerance and are not
  part of the cause. Dropped.

**Follow-ups.**
- `arc_chords` decides by an exact float compare (`ra != rb`), so the
  chord count of a circular arc depends on float noise in its two radii.
- The wall-law refusal says "would ride into the part" also when the
  source is the stock and no part is near. The text could name the stock.
  Wording only (P7).

**Cost to change later.** Low. The golden is generated output. The test
constant `MID` is local to the one test.
