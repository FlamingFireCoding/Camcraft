## #2431 - Parallel finish crosses holes that the ball cannot enter

**Decided.** The raster (Parallel, ParallelMulti, Scallop, blend and swarf,
which share `raster_pass_spaced`) joins two pieces of one line into one run
when the gap between them passes three checks:

1. The XY span of the gap is at most the tool diameter (`2r`).
2. The feed climbs or falls no steeper than 45 degrees (`SPLIT_MZ`) plus the
   tolerance. This is the same discontinuity law as the march.
3. The ball, swept along the straight feed, stays `r - eps_chord` clear of
   each facet (exact segment-to-triangle distance, as `column_clear`).

The gap then becomes one straight feed, with no retract and no plunge.

**Why.** `support_at` answers only facet tangency and convex folds between
shallow facets. A hole rim is a shallow-to-steep edge, so the march found no
ground over each hole. Every line hopped to the retract height and plunged
again on the far rim, also over holes much smaller than the tool. The same
gap occurs at a lid edge a hair off the rim (a `sqrt(2 r dz)` stretch).

**The 2r limit.** A hole that the ball fits into is model surface. The
existing pin `finish3d_land::land_finish_flies_over_the_bolt_holes...` holds
that the plain finish does not fly over a 3.4 mm hole with a 3 mm ball; the
land model (`fill_openings`) is the user's switch for that. Over such a
hole the clean hop at the safe height stays, as the maintainer's design
note on the issue says.

**The chord budget.** The slack `eps_chord` (`min(tol/4, 0.005)`) is the
budget that the march already spends on its own chords. It admits a lid
step of up to about `4 * eps_chord` and refuses a real shelf edge.

**Dropped.**
- A bridge of any width: it changes the pinned behaviour of the plain
  finish over large holes and feeds long stretches of air.
- An edge-contact candidate in `support_at` for all edges (the true
  drop-cutter rim ride): it makes the ball dip into small holes, which the
  report calls "weird behaviour", and it changes every support query.
- The `fill.rs` and `edge.rs` changes of the earlier prototype (tilted lids,
  component-local lids, `step_edge_drop`): not on main and not needed for
  the line-break symptom. They are filed as a follow-up issue.
- A stack on PR #1713 (`gap_link`): that PR does not compile alone.

**Cost to change.** `raster_bridge.rs` is one function pair. A wider limit
or a parameter is a change of the `span_ok` line and the pin
`a_hole_the_ball_fits_into_keeps_a_clean_hop`.

**Structure.** `raster.rs` was over 400 lines. The swarf family moved to
`swarf.rs` with no change of behaviour.
