## #2996 - Widen verdicts only for models limited by the file tolerance

The maintainer decided on 2026-11-20 to accept a true-surface bound up to
0.030 mm for a model made coarse by the file's stated tolerance. The
program header must state that bound. A model made coarse by the facet
budget keeps the strict law of #864.

### The rule

A model's chord error `c` bounds the deviation of its kept mesh from the
true surface. The importer promises `PROMISED_CHORD` = 0.005 mm. The
excess is `e = c - 0.005`. Each gouge law holds a tolerance `t` and judges
against the mesh at `t'`.

The persisted `file_tolerance` record permits widening only when the file
tolerance required the coarse mesh and the facet budget did not coarsen
it further. Missing records, budget coarsening and mixed causes keep the
strict law. A selection can widen only when every reduced source has the
permitted cause. Import, project save/load, copies and undo preserve the
record. This prevents one eligible part from widening another part.

For an eligible model:

| Chord `c` | Mesh tolerance `t'` | True-surface bound for `t = DEV_TOL` |
|---|---|---|
| up to 0.005 mm | `t` | at most 0.015 mm |
| 0.005 to 0.010 mm | `t - e` (#864) | 0.015 mm |
| over 0.010 to 0.020 mm (`WIDE_CHORD_LIMIT`) | `t - 0.005` | `c + 0.005`, at most 0.025 mm |
| over 0.020 mm | `t - e` (#864), no tolerance left: refused by name | - |

An ineligible model uses `t - e` at every reduced chord. At `c = 0.012`
mm, `DEV_TOL = 0.010` mm gives a mesh tolerance of 0.003 mm. At or above
`c = 0.015` mm, that law has no tolerance left and refuses the operation.

- The mesh tolerance and the true-surface bound are different quantities.
  At `c = 0.019` mm, an eligible model has a mesh tolerance of 0.005 mm
  and a true-surface bound of 0.024 mm. The header and model note state
  0.024 mm, rather than the chord or the mesh tolerance.
- `t'` never exceeds `t`. A law whose own tolerance is 0.005 mm or less
  is still refused by name in the widened range.
- `WIDE_CHORD_LIMIT` is a constant in `camcraft_sim::verify`. No file,
  facet budget or vouch can raise it.
- The widened arithmetic remains in `verify::excess_floor`. The cause
  record selects whether a reader may use it or must use
  `verify::strict_floor`.

### Safety, for each direction

- **A cut into the part.** On a convex wall, a facet can stand inside
  the true surface by `c`. The mesh allowance adds `t'`, so the stated
  true-surface bound is `c + t'`. A deeper gouge is convicted and blocks
  the post. Pins:
  `a_real_gouge_deeper_than_the_verdict_tolerance_still_blocks_a_widened_model`,
  `a_cut_deeper_than_the_widened_tolerance_into_a_convex_wall_is_still_convicted`.
- **Material left on the part.** On a concave wall, a facet can stand in
  the air by `c`. A false gouge can block the post. The rule keeps that
  conservative result.

### Each reader of the verdict tolerance

- **Gouge laws** use the widened mesh allowance only for eligible source
  models. Other source models use #864's strict allowance.
- **Laws that also guard a body other than the cutting edge** keep
  `verify::strict_floor` and `fidelity_law::tightened_strict`. These are
  the turning meridian margin, lathe axial-hole tolerance,
  `revolve::gouge_tol_on`, rotary field allowances and probe stylus slack.
  The cutting-edge laws in `VerifyContext` can use the eligible widened
  allowance. The program gate still uses the strict law for turning,
  rotary and probe operations. These operations are refused when the
  excess uses their floor.
- **Collision bands** grow by the whole excess, including the widened
  range. This applies to the reach law, `rapids_radial`, `cross_stock`,
  `drill_reach` and `drill_station`. Their growth does not come from the
  smaller deduction made by the widened mesh law.
- **`stock_top`** grows by `model_excess` directly.
- **Stock simulation and the program gate** use the same cause record
  and tolerance law as the operation. The gate refuses an operation
  when the selected law has no positive allowance.
- **Keep law and rest material** do not read the excess.
- **Fixtures** are judged by a box grown by the chord.

### What the user sees

- Each widened model adds a program header comment. For `c = 0.019` mm,
  it states a true-surface bound of 0.024 mm. It does not state that the
  model was checked to its chord alone.
- The engine emits the note after `onOpen` through `onComment`. Each
  shipped dialect uses its own valid comment delimiters and preserves
  the complete bound. A custom post may omit the note, as the maintainer
  permitted.
- The model page and MCP facts state the same true-surface bound. The
  import consequence describes the permission only for the eligible
  file-tolerance cause.

### What the user loses

For an eligible widened model, the checks can miss a true-surface gouge
up to `c + 0.005` mm, at most 0.025 mm. This stays below the accepted
0.030 mm bound. The full-fidelity bound is at most 0.015 mm. Models
coarsened by the budget lose no protection from #864.

### Considered and dropped

- **Widen all coarse models from the chord alone.** Dropped. The
  maintainer permits only the file-tolerance cause. Old records cannot
  establish that cause and therefore keep the strict law.
- **Report the chord as the check bound.** Dropped. It excludes the
  additional mesh allowance and understates the possible real gouge.
- **Spend the full accepted 0.030 mm bound.** Dropped. Keeping the
  existing 0.005 mm mesh allowance gives a smaller bound, at most
  0.025 mm, and preserves the existing widened arithmetic.
- **Widen only where #864 has no allowance left.** Dropped. A coarser
  model could then receive a looser verdict than a finer model.
- **Use `max(t, c)` as each law's tolerance.** Dropped. It can loosen a
  finish tolerance beyond the user's request.
- **Use `VouchModelFidelity`.** Dropped. The chord is a measured value.
- **Widen every body guard.** Dropped. Turning, rotary and probe guards
  must still protect the shank, holder, bar and stylus.

### Cost to change later

The limit and widened arithmetic stay in the simulation crate. A change
to eligible causes must update import, persistence, copies and undo, with
their lifecycle pins. A numerical change must also update the separate
mesh-allowance and true-surface-bound pins and the warning text.
