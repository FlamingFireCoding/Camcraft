## #2861 - Example case 2d/slots: one block, a Slot and a Trochoidal slot

**What was decided.** The case `examples/catalog/2d/slots` is a
100 x 60 x 12 mm block with two closed slots along X. Each slot is 10 mm
wide, 6 mm deep and 60 mm long. A Slot cuts the slot at `Y15`. A Trochoidal
slot cuts the slot at `Y-15`. The two operations use the same 6 mm flat end
mill, the same width and the same floor (`Z6`).

- Slot: stepover 2 mm, stepdown 3 mm, `F600`. The levels are `Z9` and `Z6`.
- Trochoidal slot: full depth in one level, `max_engagement` 60 degrees,
  `step` 0.8 mm, microlift return of 0.3 mm, `F1200`.

**Why.**

- One part with the two slots lets a reader compare the two programs in one
  file. Only the operation kind and its cutting data are different.
- A 10 mm slot with a 6 mm tool leaves 2 mm of room on each side. The Slot
  needs more than one row, and the Trochoidal slot has room for its loops.
- Each operation uses the cutting data that is usual for it. The Slot cuts
  with the full tool diameter, so it takes two levels at a low feed. The
  Trochoidal slot keeps a small contact angle, so it takes the full depth at
  two times the feed. The README table shows the result: 19 motion blocks
  against 803, and 0.60 min against 1.28 min of feed moves.
- The slots are closed and are inside the block. The gouge check then judges
  each tool path against real walls on all sides.
- The model is a mesh that the generator script writes. No outside geometry
  is used (Clean-Room Protocol). The script checks that the mesh is
  watertight and that the top area is correct.
- The round ends of the model have 64 chords for each half circle. The
  chords are inside the true circle by less than 0.002 mm. `verify` is ok
  with the true 10 mm width.

**Found during the work.** The Slot leaves up to 0.86 mm in its round ends,
and `verify` is ok. The rows touch a round end only at their end points.
#2906 is the bug. The README names it under "What to look at". The case
keeps the 2 mm stepover, because it shows the behaviour clearly. A fix for
#2906 changes `slots.nc`: rebuild the case with the fix.

**Considered and dropped.**

- Two parts or two cases, one for each operation. The comparison needs the
  two programs side by side.
- Slots open at the two ends of the block. The end points are then outside
  the block. This was not tried: a closed slot is the plain job, and its
  round ends show the difference between the two operations.
- The same stepdown and feed for the two operations. A Trochoidal slot at a
  3 mm depth and `F600` does not show why a user selects it.
- A smaller stepover for the Slot to make the leftover in the round ends
  small. The leftover does not go to zero, and the example would hide #2906.
- A check of the embedded mesh for this case in the two catalog tests. The
  check for `2d/contour-tabs` pins a review fix of that case. The script of
  this case removes the model link in the same way.

**Cost to change later.** Small. Change `build.camscript`, run
`cargo xtask catalog --case 2d/slots`, and update the test
`slots_case_posts_a_slot_and_a_trochoidal_slot_to_the_same_floor` and the
README table.

**Pinned by.** `crates/app/tests/example_catalog.rs`:
`slots_case_posts_a_slot_and_a_trochoidal_slot_to_the_same_floor`, and the
two catalog tests that read each case.
