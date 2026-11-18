## #2438 - Zero-margin 2D clearing corners note and exterior-only boundary extension

**Chosen:**
- Keep default whole-model (`SrcRef::Entity`) and stock-source (`SrcRef::Stock`) 2D clearing (`2D Pocket`, `2D Adaptive`, `2D Offset`) inside the stock outline, preserving `Containment::Inside` and fixture/vise clearance.
- Add `unreached_corners` and `unreached_corner_note` in `crates/motion/src/machining2d/unreached.rs` (filtering polygon arc noise under `CORNER_HALF_WIDTH = 0.03` mm and clipping residuals to the setup's `stock` outline so extended air corners are ignored). Call it from `keep_law::narrow_noted` via `generate.rs` whenever `cleared_around` holds, and surface non-empty operation `notes` in `script::api::op_view` so MCP `generate` and `verify` expose them.
- In `crates/app/src/operation/boundary_ctx.rs::build`, populate `AreaCtx::blocked` with the union of `slice::material_overhead` across `area::ladder_planes` for `SrcRef::Entity` (`part_solid`) and `SrcRef::Stock` (setup parts) while leaving `islands = false`. In `boundary::resolve`, step 3 (`extension > 0`) subtracts `ctx.blocked` from `reach`, so `extension` expands only past the exterior stock outline without entering standing part/island walls.
- Update `crates/app/tests/keep_law.rs::a_whole_model_extension_keeps_the_part_and_posts` (`extension: 5.0` now avoids the boss/block and posts cleanly, while `offset: 3.0` still exercises `DepthLaw::Keep`), and add end-to-end pins in `crates/app/tests/adaptive_zero_margin_island.rs`.

**Why:**
- In a zero-margin box stock around an island (such as an `R5` boss centered in a `100 x 60` plate with a `10 mm` cutter), keeping the tool inside the sharp box stock inevitably leaves `5 mm` corner cusps (`~0.86 mm` inscribed half-width). Leaving them silent while returning `ok` hid why material remained; naming them in `notes` explains the residual and points to `extension` or a smaller tool without turning a legal contained path into a verify failure.
- Setting `blocked` with `islands = false` makes `extension` grow only the exterior boundary past the stock (clearing the four corners cleanly) without altering `#1603`'s `DepthLaw::Keep` behavior for explicit inward `offset` values.

**Considered and dropped:**
- Growing the default whole-model or stock-source clearing region past sharp stock corners by default: violates `Containment::Inside`, risks hitting workholding along stock sides, and breaks `narrow_band.rs` / `adaptive_pinch.rs`.
- Setting `islands = true` on `SrcRef::Entity` / `SrcRef::Stock`: would mask manual inward `offset` gouges tested by `#1603` (`keep_law.rs`).
- Emitting corner residuals as `Verify` warnings or using `"narrower than the ... no pass cuts"` phrasing: uncut corners under `Containment::Inside` are not verify faults, and existing `band_notes` checks distinguish unreached bands from corner cusps.
- Computing unreached corners without intersecting with the `stock` outline: would falsely report sharp corners in extended air even after `extension` cleared every stock corner.

**Cost to change later:**
Low. `blocked` construction is localized to `boundary_ctx::build`, corner detection lives behind `unreached_corners`/`unreached_corner_note`, `op_view` only appends an optional `"notes"` array when non-empty, and no project file format or crate dependency changed.
