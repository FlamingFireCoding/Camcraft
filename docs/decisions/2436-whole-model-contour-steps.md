## #2436 - Whole-model 2D Contour continues across Z-varying outlines

**What was decided**

- Split the single-band body of `crates/motion/src/machining2d/contour.rs` into shared helpers (`PreparedBand`, `prepare_band`, `emit_band`, `finish_contour`) and added multi-band continuation in `crates/motion/src/machining2d/contour_whole.rs` (`contour_whole_bodies`, `contour_whole_bodies_with`).
- When a solid-body whole-model `OpParams::Contour` (`SrcRef::Entity`, `!contour_open`, `Side::Outside`) runs, `crates/app/src/operation/generate.rs` retains the boundary-control context in `whole_contour_area`, walks each band with `contour_whole_bodies_with`, and unions `contour_reach` across all walked bands for `carry_need!`.
- Whenever `contour_depth::ladder` stops an outer loop before `Bottom Z` — or stops an inner loop above a level whose section opens an additional hole (such as a counterbore narrowing below `Top Z`) — `contour_whole` trims the upper band with `trim_ladder`, slices the part's overhead silhouette at the next remaining Z (`material_overhead_of` via `whole_region_at`), re-applies `area_at`, and continues downward.
- Any unsliceable Z band (`check_unread` on `Ladder::unread`) or unmachinable continuation band refuses with `GenError::Model` / `GenError::Invalid` (carrying the partial toolpath as a ghost when moves were emitted), naming the stopped Z level and `Bottom Z`, instead of emitting a partial toolpath silently.
- Updated `crates/app/tests/real_tip_contour.rs` via `boss_top` so those boss-fillet tests select the boss top face (`SrcRef::Selection`) instead of the whole `part` entity.

**Why**

- Whole-model `2D Contour` previously sectioned the part once at `Top Z`, then let `ladder` drop any loop whose downward path hit a chamfered skirt, shoulder, or closed slot floor, finishing as `OpStatus::Ok` with missing deeper passes and missing tabs.
- Slicing the overhead silhouette at each new band boundary preserves the part's cumulative overhang shadow from `z` upward, so deeper bands clear wider upper bosses without gouging while following the newly exposed outer (or narrower bore) profile down to `Bottom Z` with tabs intact.
- Keeping explicit face/edge selections on single-band `contour_on_part` / `contour_on_bodies` preserves the rule that a user-selected contour walks only its picked rings.
- Scoping `real_tip_contour.rs` to `boss_top` matches the test fixture's intent (testing the boss fillet tip contact) now that whole-model `SrcRef::Entity` intentionally continues around the lower plate past the boss foot.

**What was considered and dropped**

- **Refusing stopped whole-model contours without continuing lower bands**: simple, but fails whenever a user selects the whole part (`SrcRef::Entity`) on chamfered bosses, stepped islands, or open-ended slots whose floors appear at an intermediate Z.
- **Unifying band `Region`s before calling `contour_reach`**: produces the wrong `carry_need!` region when stock is fitted around both an upper island and a lower plate, because the plate's reach sits outside stock while the upper island's reach sits inside stock; unioning per-band `contour_reach` results avoids this.
- **Replacing the whole `VerifyContext.region` with the bottom plate**: would fail upper-band reach and tangency checks against the upper island footprint, whereas `top_region <= R(z)` keeps `top_region` valid across downward-growing footprints.

**What it would cost to change later**

- Moderate and localized to `contour_whole.rs` and the single call site in `crates/app/src/operation/generate.rs`; explicit selection paths, `Side::Inside`, and flat/uniform contours share `PreparedBand`/`emit_band` and are unaffected.
