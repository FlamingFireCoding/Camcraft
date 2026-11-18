## #2426 - main red: keep the eroded pocket area as-is when no sliver is dropped

**What was decided:** `machining2d::pocket::non_sliver_center` (added by
#2383 for #2232) now returns the plain `region.offset(-erode)` unchanged
whenever its sliver scan drops nothing. The `Region::from_slices` rebuild is
used only when at least one sub-tolerance sliver is actually removed.

**Why:** every multi-piece area (two or more outer loops) went through the
rebuild even with nothing to drop, and the Clipper normalisation restarts
each loop at a different vertex. That moved the pass starts of multi-piece
pockets: the narrow second piece of `adaptive_entry_pieces` started at
(56, 6) instead of (56, 5), which turned the post-merge batch gate red. The
toolpath was still entered by a named ramp (no gouge), but the change was
unintended, it was not part of #2232's decision, and it would also have moved
any golden of a multi-piece pocket. With the fix, every region without a
sliver comes out exactly as it did before #2383, and #2232's sliver drop
still works where a sliver exists (`shoulder_adaptive` still passes).

**Considered and dropped:**
- Updating the test to expect (56, 6): this accepts an unintended side
  effect and leaves every other multi-piece start dependent on Clipper's
  vertex order.
- Reverting #2383: drops the maintainer's chosen Option 1 for #2232. The
  cause is one fallthrough and the fix is a few lines.

**Cost to change later:** local to `non_sliver_center`. It is pinned by
`machining2d::pocket::non_sliver_tests::a_multi_piece_center_without_slivers_is_the_plain_offset`
and by `adaptive_entry_pieces::a_narrow_second_piece_names_its_ramp_stand_in_at_each_level`.
