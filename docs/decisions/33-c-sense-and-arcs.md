## #33 - Continuous C-axis live tooling (slice 3: C rotary sense and arcs)

**Decided.**

- **Machine C rotary sense via `"axis": [0, 0, ±1]`.**
  `AxisLimits::c_z_sense` (`crates/post/src/machine/direction.rs`) reads
  `[0, 0, -1]` as clockwise about `+Z` (`-1.0`) and `[0, 0, 1]` (or an
  unstated axis vector) as right-handed about `+Z` (`+1.0`). Non-finite,
  zero, or non-Z vectors (`d[2]^2 < 0.998 * |d|^2`, matching
  `tormach.cps` and `judge::only_x_rotary`) are refused by name in
  `rotary_gate.rs`, `judge.rs`, and `fanuc-lathe.cps`.
- **Program-frame C travel and controller word sign.**
  When C is `-Z`, `rotary_gate::stamp_c_about` maps the machine's stated C
  travel `[lo, hi]` into canonical `+Z` program coordinates `[-hi, -lo]`
  before `cl::unwind_c` or the non-unwinding check;
  `machine::judge::c_letter_problem` checks `sign * c` against the
  machine travel `[lo, hi]`; and `assets/posts/fanuc-lathe.cps` checks
  the radial station in canonical `+Z` coordinates (`p.X`, `p.Y`, `p.C`)
  before posting and rate-capping `outC = liveCSense * p.C`.
- **Setup sheet notice for `-Z` C only.**
  `setup_sheet::sections::render_machine_axes` prints `C rotary turns
  clockwise about +Z (-Z axis) - posted C is negated for this machine`
  under `MACHINE AXES` when `Machine::reversed_c_rotary()` is true,
  keeping `+Z` and unstated machines unchanged.
- **Arcs on a C turn via subdivision (`cl::stamp_c`).**
  Instead of rejecting every `m.arc.is_some()`, `cl::stamp_c` refuses
  non-finite coordinates or non-finite arc centers by name, subdivides
  valid arcs with `Move::segments3(tol * 0.5)`, splits each chord with
  `split(..., tol * 0.5)`, clears `arc: None`, and rotates each chord's
  tool axis with `pose_at` so downstream verification, unwind, and
  `fanuc-lathe.cps` receive chained straight C-turn chords within `tol`.

**Why.** Reuses the existing `.mch` `"axis"` vector convention established
for `-X` rotaries (`judge::only_x_rotary`, #797) and mirrors the `#373`/`#374`
program-frame stroke translation so generation, machine checks, posting,
and the setup sheet never disagree about clockwise C machines. Splitting
arcs inside `cl::stamp_c` keeps the verifier (`sim::verify::c_steel`) and
the post reading one sequence of chords.

**What it changes in posted output.** Only machines whose `C` rotary
states a negative Z axis change: their posted C words and composite-F
deltas negate to match controller coordinates. Standard `+Z` and unstated
machines post byte-identical output. No goldens moved.

**Dropped.**

- Reading linear `"direction": -1` on rotaries: explicitly rejected per
  `axis_direction.rs` (#785).
- Axial/face live-tool station mode (`G12.1` polar interpolation) and
  adding unconfirmed `live_tool` codes to shipped `assets/machines/millturn.mch`:
  deferred to a follow-up slice alongside real-controller verification
  (left open on #33).
- Splitting arcs at post time in `fanuc-lathe.cps`: would bypass
  generator-time `sim::verify` checks.

**Cost to change later.** Self-contained across
`AxisLimits::c_z_sense`, `stamp_c`, `rotary_gate`, `judge::c_letter_problem`,
`render_machine_axes`, and `liveCSense` in `fanuc-lathe.cps`.

**Latest review corrections (PR #2103).**

- `stamp_c` checks the original arc's sagitta against `tol * 0.5` using
  the actual generated chord count before splitting those chords. If the
  subdivision cap prevents that budget, it refuses by move name and error
  budget. Chose bounded refusal over changing the shared tessellation cap;
  splitting an inaccurate chord cannot recover the original arc.
- `c_z_sense` and `fanuc-lathe.cps` scale a finite C direction by its
  largest absolute component before comparing squared lengths. This keeps
  the existing alignment threshold and Z sign. Dropped the raw squared
  norm because finite components can overflow and admit a tilted axis.
