## #1726 - The pick door takes an edge pick whose stitched ring has no area

**What was decided.** The pocket family's flat-area door
(`operation::selection_refusal`, used by `SetOperationGeometry`,
`CreateOperation`, `EditOperationLinks` and the kind change) no longer
refuses a selection when the refusal is the collapse of a ring that only
the stitcher closed (its label starts "the stitched rim of"). The picks are
stored. Generate keeps the same refusal, and the Geometry page still shows
the sentence under the picks (`selection_account` is unchanged).

**Why.** Edges are picked one at a time. The stitcher welds open runs where
their ends meet seen from above, so one vertical edge, or an edge with the
edge under it, is a "ring" with no area. In a perspective view such edges
are under the cursor. The door refused the pick, so a plain click on a
vertical edge selected nothing and the first Ctrl+click on a second edge
added nothing. The maintainer saw this as "it just changes between the two
picked edges". The door's own comment says an open run mid-pick is normal.

**Considered and dropped.**
- Matching run ends in 3D in the per-entity walk. A rim whose runs lie at
  different heights and meet only in projection is a ring today, and a 2D
  Contour takes its side from it. In 3D those runs become open runs and the
  cutter rides their line. That changes toolpaths, so it was dropped.
- Not committing a degenerate stitched ring (leave its runs open). This
  changes what Generate says and what a 2D Contour does with a vertical
  run. Dropped for the same reason.
- A structural test (resolve each chain, check that every item is an open
  run) in place of the label prefix. The door already keys on message text
  (`LINE_FROM_ABOVE`); one shared constant now names the rings at both
  stitch sites, so the prefix cannot drift.

**What it does not change.** A wall face pick and a closed chain on a wall
are refused at every door as before. No toolpath or posted output changes:
only the door is more lenient, and Generate is the backstop.

**Cost to change later.** One filter in `selection_refusal` and one
constant. If the door must refuse again, the list must still accept the
picks that build a chain edge by edge.
