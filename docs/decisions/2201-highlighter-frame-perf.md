## #2201 - Cache highlight geometry resolution, batch pick strokes into a single mesh, and cache per-frame view state

**What was decided:**

- Add `feature::index` (`resolve_chain_arc`, `patch_rings_arc`, `patches_arc`) with a thread-local weak-referenced lookup index (`Weak<Features>`, cap 32) built lazily per `Arc<Features>`. Avoid heap allocations in `feature::digest_of` and hoist lattice calculation out of `patches` and `patch_rings`.
- Add `SHARED_FEATURES_CACHE` (`Weak<[Triangle]>`, cap 32) in `app::model::geometry::features_of` so repeated reads on unchanged meshes skip re-hashing triangles.
- Add non-cloning `model::toolpath_nonempty` and `model::toolpath_warning_count` and use them in `operation::stamp::status_of` and `browser_tree::op_row`.
- Add `History::revision_token` so view caches catch both `doc_version` bumps and direct `History::execute` calls in tests.
- Add `ui::pick_highlight` (`HighlightCache` on `State`) to resolve selected, hovered, and row-hovered outlines into world polylines once per selection, visibility, or entity change.
- Add `ui::pick_highlight_mesh` using `egui::epaint::Tessellator` to draw each highlight layer (halo + ink) as a single `egui::Shape::Mesh` cached per viewport camera state.
- Add `ui::frame_cache::FrameCache` and `ui::panel_cache::PanelCache` on `State` to cache transformed entity vertices, cutting regions and outlines, toolpath overlay lines, non-transient `world::Scene` + digest, Geometry tab chips, 3-2-1 datum checks, selection account, unresolved notes, operation status banners, and browser operation rows.

**Why:**

- On models with many chains or facets, `features_of` re-hashed every triangle and `resolve_chain` re-walked and split every chain on every frame.
- `pick_ink::paint` and `shell::viewport_panel` emitted two individual `Shape::LineSegment` shapes per projected segment on every frame instead of one batched mesh.
- `world::collect`, `view_pass::sync_scene`, `browser_tree::op_row`, `op_page::status`, and `geo_page::body` rebuilt and cloned toolpaths, vertices, warnings, and selection geometry on every frame even when the document did not change.

**What was considered and dropped:**

- Caching `&Features` by raw reference address inside `geometry::feature`: dropped because stack-moved `Features` values in unit tests can reuse stack addresses. `Weak<Features>` on `Arc<Features>` keys strictly on live allocations (`strong_count() > 0`).
- Adding internal `OnceLock` cache fields to `pub struct Features`: dropped because geometry tests construct `Features { ... }` with struct literals.
- Grouping highlights with `egui::Shape::Vec`: dropped because `epaint` still tessellates each child shape individually and test shape counters observe multiple shapes. `Tessellator::tessellate_path` produces a single `egui::Mesh` with halo behind ink.
- Keying whole-scene caches only on `state.doc_version`: dropped because direct `History::execute`, `SelectEntity`, and draft UI edits can mutate state without incrementing `doc_version`. Keying on `History::revision_token()`, `selected_entity_names`, UI drafts, and `EntityGuard` pointer/pose checks keeps caches exact.

**What it would cost to change later:**

- Low. The geometry index sits behind additive `*_arc` helpers with existing `resolve_chain` and `patch_rings` signatures intact. `FrameCache`, `PanelCache`, and `HighlightCache` are internal `State` caches that can be cleared or bypassed independently without affecting document persistence or toolpath generation.
