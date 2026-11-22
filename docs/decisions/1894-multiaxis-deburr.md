## #1894 - Multi-Axis Deburr: a tilted cone flank on the chamfer plane

**Decided.** A new kind, `OpType::MultiAxisDeburr`, is appended at
ordinal 46 in `Section::MultiAxis`. It has every field of the 3D Deburr
and five more: `axis_mode` (`Axis5` or `Axis4`), `lead_deg`, `lag_deg`,
`max_tilt_deg` and `shank_avoid`.

- **The pose law.** The tool lays one cone flank on the chamfer plane of
  each break. The axis is `a = cos(h) g + sin(h) n`, with `n` the chamfer
  plane normal, `h` the cone half-angle and `g` a unit direction in the
  plane. This law defines the local chamfer. It does not prove clearance
  from a floor, another face, or a feature beyond the edge. The one free
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
- **Tool clearance.** Continuous outer convex polyhedra enclose the cone
  and each physical tool band during the cut, entry and exit. Facet
  clipping must prove that the cone and its cutting flute continuation
  meet only the finite requested chamfer prism. The shank and holder
  receive no removal exemption.
  Arithmetic that cannot prove clearance rejects the pose. No clear pose:
  the unreachable-edge policy applies (skip with a note, or refuse by name).
  The app supplies the physical shank and holder even when `shank_avoid`
  is false. A stored option must not disable the collision guard.
- **Axis4.** The tool axis stays square to the A axis (about X). The
  lean is then the root of one trigonometric equation per side.
- **Links.** The tool enters and leaves along its own checked axis line.
  It feeds out above the highest part face plus the largest physical tool
  radius and 1 mm before it rapids. This air floor keeps every intermediate
  orientation above the part. A clearance below it is refused by name.
  The cross rapid states the new pose. The first plunge ends at the air
  floor so that the verifier judges the turn in air. A pose change or a
  change in the chamfer cross section ends the run.
- **Declared breaks.** Each run declares a framed flat cap (#1891): the
  chamfer normal outward, the chord across, the chord as the width and
  `s cos(theta/2)` as the depth. The sim bound for the kind is the
  largest chord the size law allows.
- **Verify mode.** `Mode::Cone`, the same as the 3D Deburr.
- **The window.** Each candidate clips the edge along its line until
  both the apex and the lowest cone-rim point stay at or above `bottom_z`
  on every pass. A segment that has no remaining span is skipped or refused.
- **The entry and exit.** The continuous guard checks the complete axis-line
  sweeps. A nearly flat axis or a reach above 100 tool diameters is refused.
  Clearance endpoints must put the whole tool above the part. Pose changes
  and gaps split runs; an unchecked contact link is not emitted.

**Why.** The flank-on-plane law keeps the intended local chamfer independent
of the free angle. The separate continuous guard prevents contact with
other part material. A V-notch pose (the
axis on the bisector) was the other simple choice; it cuts a groove into
the corner, not a chamfer.

**Dropped.**

- Endpoint sampling for cone and holder clearance: it can miss a thin
  obstacle between samples. The continuous outer envelope can reject a
  clear pose conservatively, but it must not admit an unsafe one.
- A cone bounding sphere: it overlaps intended chamfer material and rejects
  clear edges unnecessarily. The tangent cone planes preserve clear cuts.
- Reuse of `escape_axis`: it turns the axis about a ball centre, which
  does not keep a cone on its plane.
- A `Box`ed copy of the Deburr fields: a struct variant lets the many
  `OpParams::Deburr { .. }` sites take a plain or-pattern.
- A rotary axis from the machine: `Axis4` uses an A axis about X. A
  machine whose rotary is about Y needs a follow-up.
- A shared icon: the icon table needs one glyph for each kind, so the
  kind has its own original glyph (`op-multiaxis-deburr`).
- Overlap with PR #3058 (Waterjet): it adds a kind at a stale ordinal 44.
  Whichever PR merges second takes the next free ordinal.

**Cost to change.** The project tag `multi_axis_deburr` and ordinal 46
are file contract. The pose law and the link moves are kernel-only.

**Not exercised.** The column stock sim does not model posed moves; it
says so by name (`not_judged`). `Axis4` was not posted on a rotary
machine. A tool with the axis pointing up (`a_z <= 0`) is not admitted,
so bottom edges are skipped with a note.
