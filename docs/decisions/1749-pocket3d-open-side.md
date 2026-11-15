## #1749 - a whole-model 3D Pocket's centre rides the silhouette edge

**What was decided.** The source-built footprint of a whole-model 3D
Pocket (no picks, no Boundary settings, no tilt) grows past the model's
silhouette by the tool radius less 0.01 mm (`machining3d::open_sides`),
minus the silhouettes of the setup's other parts. The level law's erosion
by the radius then puts the centre up to the silhouette edge where a
level is open to the outside (the "tool centre on boundary" containment),
so a notch open to the part's outside is cut past its open edge instead
of keeping a band one radius wide at the full stock height. Where a
level's material stands flush with the silhouette, the band is narrower
than the erosion and cuts nothing, so a plain block gains no ring.

**Why.** Outside the silhouette no part material stands at any Z, so the
cutter may go there; the old footprint kept the cutter's edge inside it,
which leaves the ridge the issue measured while the op reported ok.

**Considered and dropped.**
- The stock outline as the footprint (the 2D pocket's #12 reading): it
  would clear all the stock around the part down to Bottom Z, a much
  larger change to every whole-model 3D Pocket's output - a feature, not
  this fix. Stock past the silhouette by more than about two radii is
  still not cleared by a whole-model 3D Pocket.
- A full-radius band: it erodes to a zero-width sliver along every flush
  wall, numerically fragile; the 0.01 mm slack keeps it empty.
- A warning only: the cut can be made safely, so it is made.

**Also decided.**
- Multi-Axis Rough takes the same footprint: it is the 3D Pocket's
  ladder stamped with poses, and `operations::multiaxis_rough_peels_the_same_ladder_and_poses_its_walls`
  pins that the two are one ladder.
- The setup's other parts now reach both kinds' solid law
  (`keep_law::neighbour_parts`, the #1621 neighbour law) as a second
  net. It also names a cut through another part standing inside the
  model's own silhouette, which the ladder (islands from its own model
  only) never saw before and which used to read ok.
- `shared_face_bodies::the_sample_as_imported_refuses_the_3d_pocket_by_name`
  (#1706) pinned the old law (centre a radius inside the silhouette) on
  a 6.53 mm strip: no 6 mm tool fit, so the op refused. Under the new
  law the pocket clears the stock along the strip's sides with no gouge,
  so the test now pins that (`..._roughs_the_3d_pocket_beside_its_silhouette`).
  It does not post with the job's tool: see the next entry.

**Not changed.** A picked or Boundary-controlled region stays the user's
boundary; a tilted (3+2) 3D Pocket keeps the old silhouette reading
(owed in #1774).

**Fixtures are not in the keep-out.** The grown band assumes that only
the model and the setup's other parts stand outside the silhouette.
Fixtures (vise jaws, clamps) are not subtracted from the band. The
fixture laws of verify still judge the tool against them, so a cut into
a jaw is a post-blocking `fixture_hit`, not a silent cut. A later
fixture keep-out law starts here.

## Review of PR #1772 (takeover, 2026-11-15): the sample reads ok but does not post

**What was decided.** Option "make the test and the texts say the
truth". The sample test now deletes S1 (so the program gate runs),
generates R1 and verifies. It asserts: status `ok`, the gate ran and
passed, no gouge, only `shank_hit` (on the part) and `rapid_in_stock`
findings, all on R1 and all `blocks_post`, `ok: false`, `held_by: [R1]`.
Then it gives the tool flutes past the part's height (125 mm, shank
6 mm), generates again and asserts a clean verify. The "verifies clean"
claim is removed. The owed behaviour is #1844.

**Why.** The evidence says the findings are real and are not caused by
this PR's footprint:
- The job's tool `EM-0600-4FL` has 15 mm of flute, an 8 mm shank and a
  44 mm holder at 55 mm; the part is 118 mm tall. The 3D Pocket ladder
  keeps only the flute radius off the material above a level, so the
  shank stands 1 mm into each wall that is taller than the flutes.
- The 306 `shank_hit` are on rows inside and on the band, at levels from
  Z 60.85 down. The 30 `rapid_in_stock` are all vertical lifts up out
  of such a cut (the solid law's lift reading): the shank rises along
  the wall. No lateral link rapids through stock; the PR adds no links.
- The same R1 moves, byte for byte, with the long-flute tool give no
  findings and `ok: true`.
- A closed 30 mm-deep cavity (no open side) gives 121 `shank_hit` and
  6 `rapid_in_stock` with main's footprint law and with this PR's law
  alike. #1840 reports the same kind for the 3D Finish S1. So the
  shank blindness is main's, and this PR only gives the sample a path.

**Considered and dropped.**
- Refuse R1 on this sample, as main did: main's refusal came from the
  footprint (no 6 mm tool fit in the eroded strip), not from the reach.
  On main a 3D Pocket beside a tall wall reads `ok` and is held at the
  post by its warnings (the #722 design), as the cavity shows. A
  refusal here would be a new design for every 3D Pocket: that is #1844.
- A fix inside this PR (shank-aware levels): it changes the ladder for
  every 3D Pocket, not this PR's band, and it needs a design choice
  (leave the levels as named rest, or refuse them). Out of scope.
- Exact finding counts in the test: they change with any sampling
  change of the reach laws. The test pins the kinds, the op, the
  blocking flag and the held post instead.

**Cost to change later.** One call in each of generate's Pocket3D and
Multi-Axis Rough arms (`operation::open_side`) and one motion function; the other parts' silhouettes come from the job's
existing `neighbour_parts`.
