## #763 - OpFrame Phase 2: route feature picks through setup alignment

Chosen: add `OpFrame::pose_in_setup` (delegating to `setup_align::pose_in`), route `selection_geometry_in` (`selection.rs`), `stitch_open_runs` (`stitch.rs`), `stitched_rings` (`picks.rs`, reading `setup::active`), and `current_stamp`'s `SrcRef::Selection` arm (`stamp.rs`) through the setup-aligned pose and `OpFrame::furniture_facets` (for stock envelope). Lift only the `SrcRef::Selection(_)` refusal in `setup_align::generation_problem`, leaving `Tilt x picks`, unverified op kinds, non-Mill machines, and area controls (`!op.area.is_default()`) refused by name.

Why: `heights_sel`, `material`, `area::pocket_ladder`, `keep_law`, and the UI/picking helpers (`setup_align::view_pose`) already resolved parts in the setup frame `S` and furniture in `S`; only the 2D selection walk, edge stitching, and selection stamp still read raw document transforms. Routing them through `OpFrame` keeps one owner for frame transforms, lets the four verified kinds (`Face`, `Contour`, `Pocket`, `Adaptive2D`) machine picked features in aligned setups, and preserves byte-identical behavior in unaligned setups (`alignment` is `None`).

Dropped: allowing `Tilt x picks` or `area` picks in aligned setups in this slice (those stay for Phases 3 and 4 once tilted selection projection and `boundary_ctx` picks compose cleanly through `OpFrame::pose_in_local` / `to_local`), and routing unverified multi-axis `drive_chains` (`MultiAxisContour`) without its own alignment verification.

Cost to change later: low. Extending Phase 3 (`Tilt x picks`) or Phase 4 (`area` picks) builds directly on `OpFrame::pose_in_setup` and `stitch_open_runs`'s `setup` argument without changing any Phase 2 signature.
