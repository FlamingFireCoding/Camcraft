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
  law the pocket clears the stock along the strip's sides and verifies
  clean, so the test now pins that (`..._roughs_the_3d_pocket_beside_its_silhouette`).

**Not changed.** A picked or Boundary-controlled region stays the user's
boundary; a tilted (3+2) 3D Pocket keeps the old silhouette reading.

**Cost to change later.** One call in each of generate's Pocket3D and
Multi-Axis Rough arms (`operation::open_side`) and one motion function; the other parts' silhouettes come from the job's
existing `neighbour_parts`.
