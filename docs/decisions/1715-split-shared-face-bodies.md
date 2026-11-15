## #1715 - The as-imported sample's 3D Finish runs in six level bands

**What was decided.** The release-tier test
`the_sample_as_imported_still_generates_the_3d_finish` (one 3D Finish
generate and one verify over the whole 118 mm Z span of
`sample_ap214.step`, about 28 min in release) is replaced by six tests in
the new binary `crates/app/tests/shared_face_bodies_finish.rs`. Each test
sets the S1 top and bottom heights to one sixth of the model's Z span
(offsets from `top_of_model`; the last band ends on `bottom_of_model`),
generates S1 alone and verifies it. Each test keeps the old asserts: one
generated row, status `ok`, no `gouge` problem in the verify gate. The
bands tile the span with no gap; a default-suite test
(`the_3d_finish_bands_tile_the_whole_span`) pins that.

**Why.** The issue asks for a split "by level band". Six bands is the
release gate's `--test-threads=6`, so the bands run at the same time. A
new sibling binary, not new tests in `shared_face_bodies.rs`, keeps the
edit away from the 3D Pocket test that open PRs #1772 and #1780 change.
The heavy-step planner gives each binary with an ignored test its own
step, so the new binary has a step of its own (`app heavy:
shared_face_bodies_finish`).

**What it does not prove.** A band program is not the whole-span
program. The waterline ladder starts its levels at each band's top, so
the level Z values move by less than one stepdown at a band boundary, and
the links from one band to the next are not generated or verified. Each
band's own levels and links are generated and verified. The gouge law is
local to each cut, so a gouge in the whole program shows as a gouge in
the band that holds the cut, except for a gouge that only a cross-band
link makes.

**Found on the way (#1840).** The job keeps the never-generated 3D Pocket
R1, so `verify` skips its gate ("an operation is stale or never
generated") and `gate.problems` is empty: the no-gouge assert checked
nothing in the old test, and checks nothing in the bands. The split keeps
the assert unchanged (no assert may change in a split); #1840 owns making
it real.

**Dropped.** (1) Keep the whole-span test and add the bands: the 28 min
test stays, against the issue's "Done means" (no test over 5 min).
(2) A lighter fixture: no smaller model has the stacked shared-face walls
that #1407 is about. (3) Move the helpers into a shared module: that edits
the top of `shared_face_bodies.rs` too; the new file has its own copy of
the four small helpers (about 50 lines).

**Cost to change later.** Test-only. `BANDS` and the macro rows change
together.
