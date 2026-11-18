## #2232 - Clearing operations with outside containment clear from stock outline to selected boundary

**What was decided:** Implement Option 1 in `crates/app/src/operation/boundary.rs`
(`reach_past_edge`), `crates/app/src/operation/generate/run_job.rs`
(`name_outside_entry_refusal`), and `crates/motion/src/machining2d/pocket.rs`
(`non_sliver_center`, shared with `adaptive_troch.rs`):

1. When a clearing operation (`Family::Clearing`: `2D Adaptive`, `2D Pocket`,
   `2D Offset`, `3D Pocket`) runs with `Containment::Outside`, no open
   boundary, and setup stock present (`AreaCtx::outside_clearing_stock`), the
   machined area expands to the setup stock outline grown by `2 * r` (rounded
   corners via `stock.offset(2.0 * r)`) minus holes and standing part islands,
   unless part walls enclose the entire perimeter of the selection (`rim`
   check: `pre_free` is non-empty and the `1e-2` mm rim outside `filled` is
   completely blocked).
2. **"The inside" for outside containment:** Under `Containment::Outside` with
   setup stock, the region "the inside" of the machined area is the free
   annular/outer region between the stock side (extended by one tool radius
   in cutter-center space so outer passes run in air) and the selected face
   or edge boundary (stopped at `r` from the part wall). As the maintainer
   stated on 2026-11-16:
   > "outside machining like that should be entering from the inside and work up to the selected face or edge.."
   > "2D Adaptive; Does not understand how to \"clear\" bulk material from outside of a model up to outer selected edge. Planning for bulk removal using an adaptive strategy.. Containment \"outside\" states \"Center area has no room for entry\" which makes no sense if its going to clear from outside in.."
   > "Decision on 2232; Option1.."
3. **Where the entry goes:** Inside that free region between the stock side and
   the selected boundary, `pocket_hsm::choose_entry` places the helix/plunge
   entry at the kernel or widest point of least geodesic eccentricity in the
   cutter-center region (`region.offset(-r)`), away from the selected face or
   edge; `OffsetRings` fallback likewise seeds in the deepest eroded ring
   inside that region (`k` descending) and works outward so the final pass
   finishes at the selected face or edge (`r` from the part wall).
4. **Without stock or with flush stock:** When no setup stock exists or the
   stock leaves no room past the selection, `non_sliver_center` drops
   sub-tolerance (`<= 0.01 mm^2` area and `<= 0.008 mm` half-width) Clipper
   arc-chord slivers (decomposing pinched offset loops with
   `Region::from_slices`) so narrow `2r` bands do not spin on empty bores or
   emit zero-move toolpaths, and `name_outside_entry_refusal` annotates narrow
   or missing-entry refusals to name `Extension (Boundary)` and setup stock.
5. **Verify and post gate unchanged:** Verification (`ShankHit`,
   `FixtureHit`) and the post gate are untouched; air-band passes hitting a
   tagged fixture still block posting.

**Why:** Previously `reach_past_edge` only grew the selection by `2 * r` even
when setup stock was present, leaving a cutter-center area of width zero (or
sub-tolerance Clipper chord slivers) and refusing `2D Adaptive` with
`adaptive: the center area has no room for an entry`. Expanding to
`stock.offset(2.0 * r)` matches the open-boundary air-band law while using
round corners at the stock boundary so flush square stock corners do not leave
4 triangular corner pockets. Placing `non_sliver_center` in `pocket.rs`
(`pocket_entered`) avoids overlapping open PR #2035 in `pocket_hsm.rs`.

**Considered and dropped:**
- Option 2 (better refusal text only without clearing to stock) was rejected by
  the maintainer because it does not clear bulk material from stock to the
  selected edge.
- Using `stock.offset_sharp(2.0 * r)` was dropped because on flush square stock
  its mitered outer corners extend `sqrt(2) * 2r`, leaving `5.36 mm^2`
  corner wedges after `-r` erosion instead of a clean narrow-area refusal.
- Filtering slivers inside `pocket_hsm::hsm_passes` was dropped in favor of
  `pocket::non_sliver_center` to protect all clearing strategies (`Adaptive`,
  `OffsetRings`, `Trochoidal`) and avoid overlapping PR #2035.

**Cost to change later:** The stock-reach expansion is localized to
`reach_past_edge` and `AreaCtx::outside_clearing_stock` in `boundary.rs`, the
sliver filter to `non_sliver_center` in `pocket.rs`, and the refusal message
to `name_outside_entry_refusal` in `run_job.rs`. The pinned refusal in
`crates/app/tests/shoulder_adaptive.rs` (`containment_outside_clears_past_the_edge_never_the_recess`)
was converted into a generated-toolpath pin per the maintainer's condition.
