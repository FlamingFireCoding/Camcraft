## #2996 - A model at reduced accuracy is judged at its own chord error, up to 0.02 mm

The maintainer decided on 2026-11-20 to widen the verdict tolerance for a
model that opens at reduced accuracy (#2976, PR #2994). This entry records
how the agent built that decision.

### The rule

A model's chord error `c` is the deviation that its kept mesh has from the
true surface. The importer promises `PROMISED_CHORD` = 0.005 mm. The
excess is `e = c - 0.005`. Each gouge law holds a tolerance `t`. The law
judges against the mesh at `t'`:

| Chord `c` | `t'` | Real surface bound for `t = DEV_TOL` |
|---|---|---|
| up to 0.005 mm (full fidelity) | `t` | 0.015 mm (today) |
| 0.005 to 0.010 mm | `t - e` (#864, no change) | 0.015 mm (today) |
| 0.010 to 0.020 mm (`WIDE_CHORD_LIMIT`) | `t - 0.005` | `c + 0.005`, at most 0.025 mm |
| over 0.020 mm | `t - e` (#864), no tolerance left: refused by name | - |

- For the program verdict (`t = DEV_TOL`), the tolerance in the scale of
  `DEV_TOL` is the chord error itself: `max(DEV_TOL, c)`. The header
  note and the model note give that number.
- `t'` is never larger than `t`. A law whose own tolerance is 0.005 mm or
  less (a tight finish tolerance) is still refused by name on such a
  model.
- `WIDE_CHORD_LIMIT` is a constant in `camcraft_sim::verify`. No file,
  facet budget or vouch can raise it.
- The rule is in one function, `verify::excess_floor`. Each gouge reader
  reads it, so the readers cannot disagree.

### Safety, for each direction

- **A cut into the part.** On a convex wall, a facet stands inside the
  true surface by up to `c`. A tool that touches the mesh can cut up to
  `c` into the real part. The law does not count that in favour of the
  tool. It takes `0.005` from `t` and states that the real surface is
  judged to `max(DEV_TOL, c)`, in the same scale as at full fidelity.
  A real gouge deeper than `c + t'` is always convicted, and the post is
  blocked. Pins: `a_real_gouge_deeper_than_the_verdict_tolerance_still_blocks_a_widened_model`,
  `a_cut_deeper_than_the_widened_tolerance_into_a_convex_wall_is_still_convicted`.
- **Material left on the part.** On a concave wall, a facet stands in the
  air by up to `c`. The mesh shows a gouge that the real part does not
  have. That is the safe side: a false gouge blocks the post. The rule
  does not change it.

### Each reader of the verdict tolerance

- **Gouge laws** (`VerifyContext::tightened`, `loop_tol`, `finish.rs`
  gate, `rapids.rs`, `rapids_posed.rs`, `section.rs`, the turning margin
  `revolve::tightened`, the plasma kerf law, `mark_law`, the
  `generate.rs` allowance): they read the widened `t'`.
- **Collision bands** (`band_growth` in sim and in `fidelity_law`: the
  reach law, `rapids_radial`, `cross_stock`, `drill_reach`,
  `drill_station`): they grow by the WHOLE excess, also in the widened
  range. A wall can stand the whole excess nearer than its facets show.
  Before this change, the band growth was "what `tightened` takes out",
  which was the excess. That reading would now give only 0.005 mm, so
  `band_growth` is now its own function.
- **`stock_top`**: it grows by `model_excess` directly. No change.
- **Stock simulation** (`model_fidelity::verdict_tol`, `program_tol`):
  it judges at `t'` for `DEV_TOL`.
- **Program gate** (`op_problem`): it refuses an operation when
  `excess_floor(floor_of(op), e)` is `None`. Under the limit, an
  operation whose floor is above 0.005 mm now posts.
- **Keep law, rest material**: they do not read the excess. No change.
- **Fixtures**: they are judged by a box grown by the chord. No change.

### What the user sees

- The posted program has a header comment for each widened model:
  `Reduced accuracy: model <name> was checked to 0.019 mm, not 0.010 mm`.
  The engine writes each note through the script's own `onComment`,
  after `onOpen` (`ProgramId::notes`).
- The model page and the MCP `facts` note add the same fact in plain
  words.

### What the user loses

- On a widened model, a gouge of the real surface up to about `c + 0.005`
  mm (at most 0.025 mm) is not found. At full fidelity, the bound is
  0.015 mm.
- A model already flagged with a chord between 0.010 and 0.020 mm (a
  small facet budget) is now judged looser than before (a tolerance of
  0.005 mm against the mesh instead of `0.015 - c`). It also gets the
  header note. The previous pin of `TIGHT` (0.012 mm) is updated for that.

### Considered and dropped

- **Widen only where #864 has no tolerance left.** A coarser model would
  get a looser verdict than a finer one (non-monotone). Dropped.
- **Read `max(t, c)` as the tolerance for each law.** That makes a tight
  finish tolerance looser than the user asked. Dropped: `t'` is never
  larger than `t`.
- **Use `VouchModelFidelity`.** The issue excludes it: the chord is a
  measured value.
- **Stack on PR #2994.** The law reads any recorded chord, so the code
  and its pins do not need #2994's files. The PR stays independent.

### Cost to change later

- The limit is one constant. The rule is one function, with its pins in
  `crates/sim/tests/model_excess.rs` and
  `crates/app/tests/model_fidelity_wide.rs`.
