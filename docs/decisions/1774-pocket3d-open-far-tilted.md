## #1774 - a whole-model 3D Pocket clears out to the stock outline; a tilted one refuses

**What was decided.**
- **Far stock: cleared.** The open-side band of #1749 grows from the
  stock's outline as well as from the silhouette
  (`machining3d::open_sides(.., stock)`): the footprint is the
  silhouette united with (silhouette ∪ stock outline) grown by the tool
  radius less 0.01 mm, minus the setup's other parts. A whole-model 3D
  Pocket (and Multi-Axis Rough, the same ladder) now clears the stock
  around the model out to the stock's edge, down to Bottom Z, as the 2D
  pocket has since #12. With no stock set up, nothing changes from #1749.
- **Tilted: refused by name.** Neither kind carries a 3+2 index today:
  `OpParams::tilt` answers `None` for `Pocket3D` and `MultiAxisRough`, so
  no frame ever reaches their arms and the issue's case 1 cannot be built.
  The `tilted` branch of `operation::open_side::open_footprint` used to
  keep the old footprint silently; it now refuses
  (`TILTED_OPEN_SIDE`), so if a tilt is ever given to these kinds the op
  says so instead of leaving a ridge along every open edge. The premise
  is pinned (`pocket3d_open_side::no_3d_pocket_kind_carries_a_tilt`).

**Why.** The Prime Directive: the op must never read ok over standing
stock. Clearing is safe here: outside the silhouette no part material
stands at any Z, the ladder's islands still come from the model at every
level, the other parts are cut out of the band and kept a radius off by
the erosion, and the solid law (#1749's second net) still judges them.
The rest-machining mask still trims what earlier ops already cut.

**Considered and dropped.**
- A warning naming the uncleared band: it would fire on nearly every
  whole-model 3D Pocket with a stock margin (the default stock has one),
  while the cut itself can be made safely.
- Building the open-side law in a tilted frame: no kind can be tilted, so
  it would be dead code with no way to test it.

**Changes output.** Every whole-model 3D Pocket / Multi-Axis Rough on a
stock reaching more than about two tool radii past the model now cuts
that stock too (more moves, longer runs); a picked or Boundary-controlled
region is unchanged.

**Cost to change later.** One argument to `open_sides` and the
`stock_region` the generate head already projects.

**Test moved with it.** `level_links::a_helix_under_the_bore_floor_is_never_silent`
asserted that no move of a whole-model 3D Pocket run to the stock's
bottom goes below Z 0 anywhere. With the stock around the part now
cleared to that bottom, moves below Z 0 are expected outside the part;
the test now asserts that each of them keeps its tool disc (5 mm) off
every projected part facet, and keeps its no-move-under-the-bore and
verify-clean asserts.
