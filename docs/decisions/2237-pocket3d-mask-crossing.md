## #2237 - Prevent XY crossings at shared depths in 3D Adaptive microlifts, helices, and offset-shell floor fillets

### What was decided

1. **Exact final Z on helical descents (`crates/motion/src/entry.rs::helix_about`)**: land the final half-turn at exact `z1` (and clamp intermediate `lattice` half-turns monotonically against `z1`) instead of `lattice(z0 - span * frac)`.
2. **Depth-aware and prior-path-checked microlifts (`crates/motion/src/machining2d/microlift.rs::link_at`)**: cap/scale `ret.lift` monotonically strictly beneath the nearest earlier horizontal `Feed` cut plane above `z` (falling back to a `hop_z` rapid + `Plunge` retract when `gap <= 0.02 mm`), probe candidate microlift moves against prior `Feed` moves via `moves_cross_prior` (`machining2d::arc_cross`), promote a low crossing link to a `high: true` down-only ramp when that clears prior moves, and fall back to a `hop_z` rapid + `Plunge` retract when even the `down` leg crosses.
3. **Crossing-safe shell `feed_leg` and `push_helix` (`crates/motion/src/machining3d/{level_ctx.rs, descent.rs}`)**:
   - In `level_ctx::feed_leg`, probe each shell-subdivided chord against `b.moves()` via `seg_crosses_prior` and hop over (`Rapid` at `lvl.hop_z` + `Plunge` to `zq`) any chord that would cross a prior cut at shared depth (e.g., when a penultimate level clamps its `floor_z` down onto `bottom_z` or a flat terrace and the bottom level cuts shifted fillet offsets at the exact same Z).
   - In `descent::push_helix`, probe the planned helix arcs against already-posted `b.moves()` before committing `traverse`, search an uncrossed candidate around `along_pass` / `dive_room` across allowed helix radii if it collides with an earlier level's sloped shell move, and fall back cleanly via `fallback(..., true)` if no uncrossed bore fits.
   - Share `z_depths_overlap`, `proper_cross_touch`, `seg_crosses_prior`, and `moves_cross_prior` from `machining2d::arc_cross` so all three checks follow `verify::crossing`'s exact `CROSSING_TOUCH = 1e-9` and depth-overlap semantics.
4. **Guard the crossing-hop re-entry (PR #2289 review)**: call the existing `plunge_law::guard` with the actual destination `zq`, tool radius, and material top before posting the axial plunge. An uncleared column refuses a non-centre-cutting tool through the ladder's existing refusal path; a centre-cutting tool or a previously cleared column remains allowed.

### Why

On masked 3D Pockets and `OffsetRings + shell` parts with small stepdowns, `verify::crossed_pairs` failed on four independent mechanisms:
- A 1-ULP difference between `lattice(-0.44999999999999996)` (`-0.45`) and the cut level depth left a `5.55e-17` non-planar tail that overlapped the next level's descending helix.
- `microlift::link_at` posted `0.30..1.50 mm` microlifts without checking whether prior cut levels sat within `1.5 mm` above `z` or whether the microlift crossed same-level entry ramps/helices or earlier microlifts.
- `bore_meets_lifted_leg` only checked `lvl.passes` evaluated with the prior `pz`, missing lifted moves already emitted by earlier levels into `b.moves()`. Keeping this check inside `push_helix` also avoids overlapping PR #2035 (`descent.rs` edits outside `push_helix`).
- Penultimate `OffsetRings + shell` levels clamped `floor_z = (z - stepdown).max(bottom_z)` down to `bottom_z` across the floor before the `bottom_z` level ran shifted fillet offsets across the same XY floor at the same exact Z.
- A crossing with an earlier cut does not prove that the hop's destination column is cleared. Reusing the existing plunge guard preserves the centre-cutting law without introducing a second stock-clearance proof.

### What was considered and dropped

- **Dropping the final `bottom_z` level in `offset_shells`**: rejected because the bottom level still owns wall/fillet collar passes missed when penultimate levels are offset away from steep floors, and existing `offset_shells` contracts assert every level vertex remains on the plain pass segments.
- **Changing `adaptive::lift_link` planning parameters alone**: rejected because the 2D microlift planner operates per slice without multi-level prior-Z context or entry helix/ramp geometry, whereas guarding `link_at` protects both 2D Adaptive and 3D HSR `hsm::microlift`.
- **Editing `descent::helix` directly**: dropped both because `b.moves()` belongs to `PathBuilder` at post time and because `#2035` refactors `descent::helix` into `descent_place.rs`.
- **A new ramp or helix for an unsafe crossing-hop re-entry**: dropped for this review fix because it would require a new entry proof; the existing named refusal safely handles the unproved column. This localized guard can later be replaced by a proved entry without changing the public parameters.

### What it would cost to change later

Localized to four leaf functions (`helix_about`, `microlift::link_at`, `level_ctx::feed_leg`, `descent::push_helix`) and two `pub(crate)` helpers in `arc_cross`; each guard can be relaxed or moved into an upstream multi-level spatial index without affecting project schema, commands, or downstream crates.
