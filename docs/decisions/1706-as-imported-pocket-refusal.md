## #1706 - The as-imported sample's 3D Pocket refusal is correct; the heavy test asks for the 3D Finish only

**Decided:** in `crates/app/tests/shared_face_bodies.rs` the heavy-lane
test `the_sample_as_imported_still_generates_both_operations` becomes
`the_sample_as_imported_still_generates_the_3d_finish`: it generates S1
(the ball 3D Finish) alone and verifies it. The 3D Pocket R1 refusal is
kept and pinned in the default suite by
`the_sample_as_imported_refuses_the_3d_pocket_by_name` (about 0.1 s).
No product code changed.

**Why the refusal is correct:** a 3D Pocket's machine area is the
model's own XY silhouette (`operation::material_source`, the `Pocket3D`
arm), and each level erodes the area left after the part's section by
the cutter radius (`machining3d::ladder`). The sample as imported
(cylinder axis horizontal) casts a strip from above only 6.53 mm wide
(Y -4.94 .. 1.59 mm). With the section taken out, no 6 mm end mill fits
at any level, so the ladder refuses with "nothing to cut: the model
fills the pocket at every level". Nothing is cut and nothing posts:
that is the Prime Directive's "visibly refused". The stock does not
change the result: a 70 x 12 x 120 mm box stock round the lifted part
gives the same refusal (measured, then dropped from the change).

**Cause:** no merge changed the result. The test commit itself
(`6726c1b81`, PR #1419) and its merge (`25df19ded`) refuse in the same
way, and a `git bisect --first-parent` from that merge to `origin/main`
(`101d82f9d`) refuses at each of the seven merges it ran. The test is
`#[ignore]`d, so its first run was the v0.20.0 release gate. The
importer candidates (PR #1671, #1576, #1638) do not change the sample:
it imports with no notes and as three `Outward` bodies.

**Considered and dropped:**
- Let the 3D Pocket cut outside the model's silhouette. That changes a
  machining-area law that every 3D Pocket reads; it is design work, not
  a cut fix.
- Give R1 a tool small enough for the strip. The test would then pin a
  0.5 mm end mill 118 mm deep, which is not a real job.
- Delete the heavy test. The as-imported 3D Finish over the full Z span
  is still worth a run at the cut.

**Follow-up:** the message says "the model fills the pocket" when the
real reason here is "the area is narrower than the tool". Issue #1709
(`P7`) asks for the two cases to be named apart.
