## #2389 - A tilted op's reference heights are read along its tool axis

**Decided:** under a non-upright 3+2 tilt, every reference height
(Top/Bottom of stock, Top/Bottom of model, Work offset) is measured along
the turned tool axis: the highest/lowest point of the referenced geometry
in the local frame (`TiltedFrame::z_local` over its facets), handed to the
existing fold as `z_local + origin.z`, so the fold's unchanged `- origin.z`
lands on the local value. One function (`operation::fold_height`) serves
the generation heights (`collect_job`), the section plane of the
Contour/Pocket/Trace families (`section_z_of`) and the area-controls cut
plane (`boundary_ctx`). Fixed heights keep the old `h - origin.z` law
(the tilt suites pin it), and so does a Selection reference (a tilted op
refuses a picked source by name first). An untilted op and an upright
tilt (A0 B0) take the setup resolve verbatim, so every prismatic path and
golden keeps its bits.

**Why:** the reported defect is wrong output (P1): a Slot at A 90 on a
10 x 30 bar took "Top of stock" as 15 along the axis instead of 5. The
smallest correct place is the resolve itself; #763's single
machining-frame refactor was explicitly not started.

**Considered and dropped:**
- Removing the `- origin.z` fold and returning pure local heights: it
  touches every arm of `run_job` and `source_region`, and changes what a
  Fixed height means on a tilted op (files on disk would cut elsewhere).
- Using `OpFrame::height_z_local` as is: it reads Fixed as already local,
  which disagrees with the shipped `h - origin.z` law for Fixed.
- Box-corner projection of the stock AABB (as `stock_top_in_frame` does
  for the level cap): it over-reads a round bar at A 45 by up to sqrt(2);
  the facets give the real extent.
- Doing the viewport planes and the "(= Z ...)" echo in the same PR:
  display only, a separate UI change; filed as #2416 (P3).

**Not changed:** `material_local.top` (still `material.top - origin.z`),
the post's `ClearancePlane` parameter (`posting.rs`, setup Z), and the
UI's `region_z` reads - #2416.

**Cost to change later:** one function; #763's seam can absorb it.
