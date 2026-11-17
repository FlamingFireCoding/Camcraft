## #2301 - Bore contour: the axial dive under carried stock starts over the core

**What was decided.** The cause is the dive start, not the verify reading.
A bore-side contour cuts only its ring. When the Ramp entry's chord is
blocked on a later level, the substituted dive lands at a point along the
first pass, a wall margin inward from the cut path. Its footprint then
reaches the core inside the ring, which no level cuts. The core stands at
the top of the material (with carried stock, `material_top`, Z 5 in the
issue's case). The old code rapided down to the plane above plus the
plunge height (Z 2), into that core. Verify was right to convict it.

The bore side now reads the material's top at the dive point
(`contour_bore::column_top`):

- the plane above, where the dive point is on the level's path (the
  lead-in and the ring, which every level above cut), or where an
  earlier plunge at that point went down to the plane above;
- else the top of the material (`entry_top()`), so the rapid stops the
  plunge height over the core and the rest is a plunge feed.

The law applies to the substituted dive and to the Plunge arm's dive.

**Why.** It makes the dive start over the stock (the issue's "Done
means") and keeps the verify law as it is. On the later levels the dive
point's column is already open from the first dive, so they still start
the plunge height over the plane above: no long feed through air.

**Considered and dropped.**

- Acquit the rapid in verify: wrong, the rapid really enters the core.
- Always start every off-path dive at the material's top: safe, but
  each later level then feeds through the air that the first dive left.
- Move the dive point back onto the path: the wall margin keeps the
  dwell off the finished wall (M11.3), so this would mark the wall.
- An operation parameter: there is no strategy choice here, only one
  correct start height.

**Not covered.** The Helix arm can also place its start along the pass
with an offset. No test case showed a rapid into stock there, so it is
not changed. PR #2013's skipped case in `one_descent.rs` is not on
`main`; that PR can drop its `known` skip when it takes this fix.

**Cost to change later.** Local to `contour_bore.rs`; one helper.
