## #2283 - 2D Chamfer in an aligned setup

**Decided.** 2D Chamfer joins `setup_align::verified_kind` and the list in
the refusal message. Its proof is `crates/app/tests/setup_align_chamfer.rs`:
the L prism turned 90 degrees about X and then about Z, a chamfer mill
(`CM-0600-90`), and ONE picked edge on the face that is up in the setup
frame, closed to that face's loop with `operation::extend_to_loop_in` (the
panel's "extend to loop", setup frame since #2281). The turned setup and the
same part with the turn baked into its facets cut the same moves and post
the same program; both verify clean and simulate with no gouge cell. The
cut also stands on the turned top face read analytically (the square
X [10, 20], Y [-5, 5] at Z 15): every cut is within the tool radius of the
square, no deeper than the cone reaches, and the ride is a square centred
on (15, 0). The part as modelled cuts a different path.

**Why.** The chamfer generator reads the same doors as 2D Contour: the
picked region comes from `region_for_job` (picks through
`OpFrame::pose_in_setup`), the machine area is the section of the setup
frame's material (`material_source`), and the top plane resolves through
the shared height law. No generator code had to change; the refusal was
held only for want of a proof (#2276 rule: remove a refusal only with its
proof test).

**Dropped.**
- A face pick instead of an edge pick: the issue asks for a picked edge,
  and the edge path also proves `extend_to_loop_in` in the turned frame.
- A golden `.nc` file for the unaligned case: the door returns early for an
  unaligned setup (`stored` is `None`), so the unaligned program cannot
  change; `unaligned_chamfer_rides_the_modelled_top` pins the open door and
  the modelled ride, and the existing chamfer suites (`mill2d_b`,
  `edge_break_*`) run unchanged.
- The three older tests that used 2D Chamfer as the "still refused"
  example now use Deburr (#2284), which stays refused.

**Cost to change.** One line in `verified_kind` puts the refusal back.
