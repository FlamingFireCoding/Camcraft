## #1894 - Multi-Axis Deburr: a tilted cone flank on the chamfer plane

**Decided.** A new kind, `OpType::MultiAxisDeburr`, is appended at
ordinal 46 in `Section::MultiAxis`. It has every field of the 3D Deburr
and five more: `axis_mode` (`Axis5` or `Axis4`), `lead_deg`, `lag_deg`,
`max_tilt_deg` and `shank_avoid`.

- **The pose law.** The tool lays one cone flank on the chamfer plane of
  each break. The axis is `a = cos(h) g + sin(h) n`, with `n` the chamfer
  plane normal, `h` the cone half-angle and `g` a unit direction in the
  plane. The whole tool then stays on the outer side of the plane, so the
  cut is the flat chamfer and nothing more near the edge. The one free
  angle is the turn of `g` in the plane (a rotation about `n`). The lean,
  the holder escape and the rotary plane all use that angle.
- **Shared edge code.** The weld, the edge pairing and the convexity gate
  moved from `finishing3d/deburr.rs` into `finishing3d/deburr_scan.rs`.
  The 3D Deburr and the new kernel both call it. The 3D Deburr keeps the
  same order of filter calls; its 31 tests pass with no change.
- **Max tilt.** It is measured from the edge's natural pose (no lean, on
  the side whose axis stands up most). `lead - lag` must stay inside it;
  the command door and the kernel refuse a lean outside it.
- **No admitted nominal.** On a vertical edge the natural axis is
  horizontal, which is not admitted. Then the most upright admitted pose
  is first, so that the holder stays high.
- **Holder escape.** The existing stage F test (`avoid::shank_hits_at`)
  is run along each stretch (one tool diameter or less) and at each
  pass. The clear pose nearest the previous one wins. No clear pose:
  the unreachable-edge policy of the 3D Deburr applies (skip with a note,
  or refuse by name).
- **Axis4.** The tool axis stays square to the A axis (about X). The
  lean is then the root of one trigonometric equation per side.
- **Links.** The tool enters and leaves along its own axis. It feeds out
  to 1 mm over the window top before it rapids. The cross rapid states
  the new pose (the post must know the rotary angles before a feed). The
  axis turn is on the plunge-feed descent in the air, where the mid-turn
  sweep law reads it. A run ends where the chamfer plane turns more than
  10 degrees (a corner) or the axis turns more than `LINK_TURN_DEG`.
- **Declared breaks.** Each run declares a framed flat cap (#1891): the
  chamfer normal outward, the chord across, the chord as the width and
  `s cos(theta/2)` as the depth. The sim bound for the kind is the
  largest chord the size law allows.
- **Verify mode.** `Mode::Cone`, the same as the 3D Deburr.
- **The window** bounds the edges, not the tip: a tilted tip stands
  beside the edge, not under it.

**Why.** The flank-on-plane law makes the cut independent of the free
angle, so the holder escape cannot gouge the break. A V-notch pose (the
axis on the bisector) was the other simple choice; it cuts a groove into
the corner, not a chamfer.

**Dropped.**

- Reuse of `escape_axis`: it turns the axis about a ball centre, which
  does not keep a cone on its plane.
- A `Box`ed copy of the Deburr fields: a struct variant lets the many
  `OpParams::Deburr { .. }` sites take a plain or-pattern.
- A rotary axis from the machine: `Axis4` uses an A axis about X. A
  machine whose rotary is about Y needs a follow-up.
- A new icon: the kind uses the Deburr icon.
- Overlap with PR #3058 (Waterjet): it adds a kind at a stale ordinal 44.
  Whichever PR merges second takes the next free ordinal.

**Cost to change.** The project tag `multi_axis_deburr` and ordinal 46
are file contract. The pose law and the link moves are kernel-only.

**Not exercised.** The column stock sim does not model posed moves; it
says so by name (`not_judged`). `Axis4` was not posted on a rotary
machine. A tool with the axis pointing up (`a_z <= 0`) is not admitted,
so bottom edges are skipped with a note.
