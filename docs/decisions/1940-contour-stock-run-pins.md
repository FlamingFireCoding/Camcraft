## #1940 - Name-pin the three untested #1928 stock open-run refusal branches

**What was decided.** Add three app-level integration tests in
`crates/app/tests/contour_stock_run.rs` next to the existing #1928 pins,
each asserting the named refusal error message as well as a valid
counterpart run:

1. `auto_on_a_stock_run_with_part_changing_sides_between_levels_is_refused_by_name`:
   places upper and lower part blocks on opposite sides of the stock's
   north edge across Z `[5, 10]` and `[0, 5]` with a 5 mm stepdown,
   pinning the `Side::Auto` level-change refusal in `part_left` while
   confirming explicit `Side::Outside` and `Side::Inside` succeed.
2. `a_stock_run_with_non_positive_offset_is_refused_by_name`: uses a
   0.001 mm flat endmill (`r = 0.0005 <= TOL`) with zero stock to leave
   to trigger and pin the non-positive offset guard in
   `contour_open_run_sided`, while confirming adding `0.01` mm stock to
   leave makes the run legal.
3. `a_stock_run_whose_offset_folds_onto_itself_is_refused_by_name`:
   imports a slab with a 3 mm-wide U-cove on its north edge as
   `StockShape::FromModel` (`SetStockSourceOnly`), pinning the shared
   `offset_run` `"folds onto itself"` refusal with a 10 mm tool while
   confirming a 1 mm tool succeeds on the same cove.

**Why.** #1928 shipped with tests covering basic offsetting, no-part
`Side::Auto`, and gouging `Side::Inside`, leaving these three guards in
`contour_run_stock.rs` and `contour_run_side.rs` unpinned at app level.

**Considered and dropped.**
- Using negative `stock_to_leave` (`-5.0`) to drive `r + stock_to_leave <= 0`.
  Dropped: `OpParams::is_sane` already rejects negative `stock_to_leave`
  before motion planning, so a sub-tolerance tool (`d = 0.001` mm) is the
  command-valid path that reaches `contour_open_run_sided`'s guard.
- Adding a shared helper in `crates/app/tests/area_support/mod.rs`.
  Dropped: two open PRs already touch `area_support/mod.rs`, and keeping
  `add_flat_tool` / `notched_stock_mesh` local inside
  `contour_stock_run.rs` avoids cross-PR conflicts while keeping the file
  under 230 lines.
- Pinning the guards only via `crates/motion` unit tests. Dropped: #1940
  specifically asks for the tests in `crates/app/tests/` beside the
  other #1928 pins.

**Not changed.** No production code, commands, goldens, or UI code changed.

**Cost to change later.** Negligible: self-contained integration tests in
one file (`crates/app/tests/contour_stock_run.rs`).
