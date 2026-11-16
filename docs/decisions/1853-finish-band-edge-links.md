## #1853 - 3D Finish band-edge levels and cross-band links

- **What was decided**: Add a default-suite integration test (`the_3d_finish_band_split_levels_move_within_stepdown_and_links_cross_edge` in `crates/app/tests/shared_face_bodies_finish.rs`) running `3D Finish` (`BN-0600`, kind defaults) on `Milling-test.step` (`70 x 70 x 27` mm box stock at `z = -7`) across two adjacent bands (`[20.0, 18.0]`, `[18.0, 16.0]`) and the continuous `[20.0, 16.0]` span across `edge_z = 18.0`.
- **Why**: Splitting the `sample_ap214.step` finish test into six bands (#1846) meant neither band-edge level shifts (`< scallop_stepdown` relative to the forced split boundary at `edge_z`) nor cross-band clearance `Rapid` + `Plunge` link generation and verification were ever asserted. `Milling-test.step` exercises real STEP-to-toolpath generation, `posting::verification_report`, and `stock_sim::simulate_document` with zero warnings and zero gouges in ~6.6s inside the default suite.
- **What was considered and dropped**:
  - Narrow-window `sample_ap214.step` boundary test: hits pre-existing `ShankHit` warnings and a small raster chord gouge (#1840) unrelated to cross-band links, and runs far slower.
  - Another `#[ignore]` heavy-lane test: skipped by `cargo xtask gate quick` and normal PR runs.
  - Synthetic 2:1 frustum mesh: introduces a wall scan/dexel artifact unrelated to band splitting.
- **What it would cost to change later**: Test-only addition in `crates/app/tests/shared_face_bodies_finish.rs`; swapping or extending the fixture affects no production code or file format.
