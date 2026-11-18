## #2403 - Shared user-friendly pick names cached per entity revision

**What was decided:**

1. **Deterministic local-space name table in geometry (`crates/geometry/src/geometry/feature/names.rs`, `patch_shape.rs`):** `build_feature_names` assigns one deterministic user-friendly label per pickable item (`FeatureNames`, keyed by `(NameKind, u64)`) from local-space geometry alone:
   - Planar faces (`FaceKind::Planar`): `"Face 1..M"`, sorting standalone planar faces before smooth-patch strip faces, ordered by quantized local centroid (`1e-3 mm` lattice, tie-broken by id).
   - Welded smooth patches: `classify_patch` uses triangle-normal covariance eigendecomposition + Kasa 2D circle fit (plus `>= 0.9 * PI` angular wrap and radial normal sign) to separate concave cylindrical `"Bore N"`, convex cylindrical `"Cylinder N"`, and general `"Curved face N"`, ordered by quantized centroid and id.
   - Pickable edge chains, sub-runs, and remaining singleton feature edges (`singleton_chain_id_with_lattice` in `feature.rs`): `"Edge 1..K"` ordered by quantized local midpoint and id.
   - Pickable feature corners: `"Vertex 1..V"` ordered by quantized local position and id.
2. **Revision + facet-allocation cache in ECS (`crates/app/src/schema.rs`, `crates/app/src/pick/names.rs`):** `FeatureNamesMemo { rev, tris, table }` caches the `Arc<FeatureNames>` on each model or stock entity keyed by `Revision` and `Weak<[Triangle]>`. `feature_names_of`, `feature_name`, `feature_name_or_kind`, `label_in_table`, and `geo_label` serve every app, UI, and MCP caller without per-frame recomputation.
3. **Internal IDs stay on the wire; user text never shows `0x`:** `GeoSel` and `FeatureSel` keep their 64-bit geometry hashes in commands, `.camproj` files, undo history, and MCP `"id"` fields. `Display for FeatureSel` (`crates/app/src/operation/types/sel.rs`) prints only plain nouns (`"face"`, `"edge"`, `"curved face"`, `"vertex"`). All UI pages, status lines, chips (`short_id` removed from `crates/ui/src/state.rs`), refusals, fixture mate errors (`fixture_locate.rs`, `fixture_locate_land.rs`), and MCP views (`list_features`, `detect_pockets` `"floor_name"`, `get_operation_geometry`, `set_operation_geometry`, `set_operation_area`, `set_wcs_from_features`, `get_selection`) resolve through `pick::geo_label` / `pick::feature_name_or_kind`. Missing features fall back to the kind noun (`"Face of 'part' · missing"`) with no hexadecimal fallback.

**Why:** Previously `Display for FeatureSel` printed `"welded face 0x..."` / `"face 0x..."` and `state.rs::short_id` printed `"abcd…ef"` on missing chips, leaking raw 64-bit hashes into status lines, refusals, Setup/WCS panes, and Geometry chips while `locate_pick` numbered faces independently (`#N`). Centralizing naming in one revision-cached table ensures every UI surface and MCP tool refers to the same feature by the same stable human-readable name.

**Considered and dropped:**
- Storing friendly names inside `GeoSel` or `.camproj` files: dropped because it changes serialized schemas, command equality, and undo signatures when internal hashes already provide stable identity.
- Computing names in world-transformed space or per frame inside `ui`: dropped so moving/orienting a part does not renumber its features and egui frame draws avoid per-frame classification cost.
- Keeping `short_id` hex digests for deleted/missing picks: dropped because missing chips still exposed raw internal ids to users; falling back to the plain kind noun plus `"missing"` keeps the chip informative without hex noise.

**Cost to change later:** Low. `FeatureNamesMemo` is derived and never serialized; changing label prefixes or classification heuristics requires editing only `patch_shape.rs` / `names.rs` and bumping tests asserting exact names.
