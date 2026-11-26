## #4182 - The stock export judges the mesh as the file holds it; duplicates settle by edge parity

**Decided.** `stock_export::file_bytes` builds the bytes that `write_stl` and the UI's stock export write. It rounds every corner to the `f32` value binary STL holds (`repair::weld_at_f32`). When any facet collapses there, it welds the rounded corners bit-exact (`weld` at 0) and checks that result with `unpaired_edges`; a surface with any edge not on two facets is refused by name and nothing is written. A mesh with nothing to collapse writes byte for byte as before. In `repair::drop_degenerate`, triangles that share one vertex set (either winding) are settled by the parity of their edges' uses outside the set: when the rest of the mesh uses each of the three edges an even number of times, no copy stays; otherwise the first copy stays, as before.

**Why.** The export checked the `f64` mesh, then wrote `f32` corners. Corners closer than one `f32` step (7.6 um at 64 mm, 15.3 um at 150 mm) are one corner in the file, so a mesh that passed the check was written with repeated corners and edges on one or three facets. Welding at the written values is the only way the check sees the file. `weld` kept one copy of a back-to-back pair, the shape a closed sliver leaves when its two faces weld together: each of its three edges was left on an odd count, so the weld opened the mesh it was cleaning (a thin tetrahedron welded at 5e-6: 1 facet kept, 3 open edges).

The issue proposed "k copies keep k mod 2". That drops both copies of a flipped duplicate laid on a closed cube, where `weld_drops_collapsed_and_duplicate_triangles` pins that one copy stays (the cube's own facet is one of the two). Edge parity covers both cases with one rule: the copy that stays is the one that leaves every edge of the set on an even count.

**Considered and dropped.**
- k mod 2 as the issue wrote it: breaks the pinned duplicate-on-a-cube case above.
- Welding at a fixed tolerance in the export (e.g. 1e-5): the `f32` step grows with the coordinate, so no one tolerance matches what the file holds at every size; rounding to `f32` is exact.
- Writing the file and reading it back to check: the same result with a second pass over the bytes; `file_bytes` checks the facets the bytes are made from.

**Cost to change later.** `file_bytes` and `weld_at_f32` are two functions; going back is calling `stl_bytes` in `write_stl` and the UI. The parity rule is `duplicate_sets` in `repair`. Pins: `a_back_to_back_pair_left_by_a_weld_goes_and_the_mesh_stays_closed` and `corners_closer_than_an_f32_step_weld_at_their_written_values` in `crates/geometry/tests/repair.rs`, and `the_exported_file_is_closed_as_written_and_a_clean_mesh_writes_unchanged` in `crates/app/tests/stock_export.rs`.
