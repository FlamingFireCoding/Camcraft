## #1608 - Center the fixture stock in trochoid_law and its sibling circular law suites

**What was decided:** Center the `40 x 40 x 8` `StockShape::Box` on `pos (0, 0, 0)` (with `rot` unchanged) and assert `model::stock_world_aabb_for` equals `[-20, 20]^2 x [0, 8]` inside `job()` and dedicated pinning tests in `crates/app/tests/trochoid_law.rs`, `crates/app/tests/circular_pick_law.rs`, and `crates/app/tests/circular_entity_law.rs`. In `trochoid_law::a_whole_model_trochoid_through_the_pocket_wall_gouges_and_never_posts`, assert `by_op = rotated` on `assert_post_refused` so the aligned (`rotated = false`) case allows the program simulator's refusal clause to lead now that the stock covers `[-20, 20]^2`.

**Why:** `SetStock` places a `StockShape::Box` centered on `Location.pos`, so the previous `(-20, -20, 0)` and `(20, -20, 0)` offsets placed the aligned stock on `[-40, 0]^2` and the rotated stock on `[0, 40] x [-40, 0]`, covering only one quarter of the `[-20, 20]^2` part. Both sibling suites `circular_pick_law.rs` and `circular_entity_law.rs` used the same offsets.

**Considered and dropped:** Leaving the post-refusal assertion `by_op = true` on `rotated = false` by bypassing or reordering program simulation; dropped because the program simulator seeing the gouge on the full `[-20, 20]^2` stock and leading `post_setup`'s error list is the expected verification-gate behavior (and the op-level `gouges(&warns) > 0` assertion still checks the operation directly). Also considered fixing only `trochoid_law.rs`; dropped because the issue asked to check `circular_pick_law.rs` and `circular_entity_law.rs` and both had the identical quadrant offset.

**Cost to change later:** Low; test-only fixture change across three integration test files in `crates/app/tests/`.
