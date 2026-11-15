## #1842 - G76 schedule readings checked against published sources

**What was decided.** Three of the four open G76 readings in
`assets/posts/fanuc-lathe.cps` (decision #417) agree with two published
control manuals of two different makers. The post and its goldens stay as
they are. New tests in `crates/app/tests/lathe_post_g76_sources.rs` pin the
three readings. The fourth reading stays open on #1842.

Sources (behaviour only; no post-processor text or output of another CAM
product was read):

- S1: FANUC Series 0i-MODEL F Operator's Manual, Lathe System,
  B-64604EN-1/01, section 4.2.7 "Multiple Threading Cycle (G76)", text and
  Fig. 4.2.7 (a) "Cutting path in multiple threading cycle" and
  Fig. 4.2.7 (b) "Detail of cutting". The same text and figures are in
  FANUC 0i-D Descriptions B-64302EN/01, section 5.36.7 (same maker, so
  not counted as a second source).
- S2: Siemens SINUMERIK ISO Turning Programming Manual 07/2022,
  A5E52211627B AA, section 4.1 "Multiple-thread cutting cycle (G76)",
  Fig. 4-25 "Cutting path ..." and Fig. 4-26 "Infeed during thread
  cutting", pages 87-89. The SINUMERIK 808D ISO manuals (6FC5398-0DP40)
  have the same text.
- Contrast only: Hurco WinMax Lathe "G76 - Multiple Threading Cycle"
  ("Z(w) - Z end position of the cycle", "single-edge"); Haas lathe G76
  ("The leading edge of the cutter does all of the cutting"; a different
  word format); GSK988TA G76 (used by #417); LinuxCNC G76.

Readings:

1. **Start point and Z shift - settled, the post agrees.** S1 Fig. (a)/(b)
   and S2 Fig. 4-25/4-26: the start point A is above the crest point B at
   the same Z; each pass starts on the line from B at a/2 to the radial
   direction, deeper passes further along the direction of travel. S2 and
   S1 both say one edge cuts. The post starts at the record's crest
   station `c_last + k*tan(a/2)`, and pass n starts `depth_n*tan(a/2)`
   from it along the travel. Pinned by
   `each_pass_starts_on_the_crest_flank_line_along_the_travel`.
2. **End point - settled, the post agrees.** S1: "X_, Z_: Coordinates of
   the cutting end point (point D ...)"; S2: "Z, W: Endpoint of thread in
   the direction of Z axis"; both figures end every pass on the one
   vertical line through D. The post writes the root pass's end as Z. The
   kernel lays a shallower pass short of D by `(k - depth)*tan(a/2)`; the
   control runs it to D. That extra length is at a smaller depth inside
   the groove that the root pass cuts to D, so it removes no material
   beyond the verified root pass and does not move toward a shoulder.
   Pinned by `every_pass_runs_to_the_posted_end_z_inside_the_root_pass`.
   The simulation detail is tracked in #1896.
3. **Minimum depth - settled, the post agrees.** S1: "When the cutting
   depth of one cycle operation becomes smaller than this limit, the
   cutting depth is clamped at this value." S2: the clamp applies "when
   the cutting depth during a cycle operation (Δd - Δd-1) becomes less
   than this limiting value". The quantity clamped is the cut of one pass
   below the pass before it. A clamp against the unclamped rung
   `dd*sqrt(n-1)` would give cuts smaller than dmin after the first clamp,
   which contradicts "clamped at this value". Pinned by
   `the_minimum_depth_clamps_the_cut_of_one_pass`, which also proves that
   the two readings give different ladders on its fixture.
4. **Internal rounding - open.** S1's unit table and S2's note 5 confirm
   that dmin, dd and k are in the least input increment with no decimal
   point, and that R(d) takes a decimal point. Neither source says how the
   control rounds `dd*sqrt(n)` inside the cycle. The post's guard (R from
   the rounded ladder, refusal beyond one increment) stays. Only a run on
   a control or its simulator can settle this.

**Why.** The launching task permits a reading to be "settled" only on two
independent published sources that agree with the post. S1 and S2 are from
two makers. No reading showed the post wrong, so no posted G-code changed.

**Considered and dropped.** Refusing or warning on flank G76 until the
readings are verified (the issue's open question): dropped, because
readings 1 and 2 are now settled. Editing the post's header comment to cite
the sources: dropped, to keep the diff away from the open draft PRs #1866
and #468 that also edit the post.

**Cost to change later.** If a control run shows reading 4 differs, only
`g76Allowance` in the post and `control_depths` in
`crates/app/tests/common/g76.rs` change.
