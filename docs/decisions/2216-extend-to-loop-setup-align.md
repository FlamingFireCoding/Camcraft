## #2216 - Read setup part alignment in boundary_picks::extend_to_loop

**What was decided.**
- Added `operation::extend_to_loop_in(world, setup, picks)` in `crates/app/src/operation/boundary_picks.rs` (re-exported in `crates/app/src/operation/mod.rs`), reading `setup_align::pose_in(world, &e, setup)` instead of `model::transform_of(world, &e)` — the same transform `OpFrame::pose_in_setup` / `selection.rs` uses for picked features.
- Updated `operation::extend_to_loop(world, picks)` to delegate to `extend_to_loop_in(world, crate::setup::active(world).as_deref(), picks)`, matching `setup_align::view_pose` (the frame the viewport draws and picks in, and the same convention as `pick::circle_centre`).
- Updated `boundary_draft::extend` in `crates/ui/src/boundary_draft.rs` and `edge_pick::take_loop` in `crates/ui/src/edge_pick.rs` to call `extend_to_loop_in` with the open operation's `setup`, so extending an operation's boundary or Shift-clicking an edge loop honours that operation's setup alignment even when another setup is active.

**Why.**
- `extend_to_loop` previously read face normals through `model::transform_of`, ignoring the setup's `part_alignment`. Under a Z rotation or a 180-degree X/Y flip (`|-n.z| == |n.z|`), `flat = n.z.abs()` happens to pick the same face, whereas under a 90-degree flip onto a side or any free angle, `extend_to_loop` chose the model-frame horizontal face rather than the setup-frame horizontal face.

**What was considered and dropped.**
- *Adding `setup: Option<&str>` directly to `extend_to_loop` and editing `boundary_draft::take_pick`:* open PR `#2149` (stacked on `#2077` and carrying `conflicts`) modifies `boundary_draft::take_pick` and `boundary_draft::refresh_preview` in `crates/ui/src/boundary_draft.rs`. Keeping `extend_to_loop`'s signature (reading `setup::active(world)` for viewport clicks in `take_pick`, matching `view_pose`) and adding `extend_to_loop_in` for `boundary_draft::extend` (a different function) and `edge_pick::take_loop` avoids overlapping `#2149`'s edited functions while still giving every operation-scoped caller the operation's own setup.
- *Dropping `.abs()` on `n.z.abs()`:* `extend_to_loop` picks "the most horizontal planar face", including downward-facing horizontal faces (`n.z = -1`). Without `.abs()`, vertical walls (`n.z = 0`) would beat horizontal bottom faces.

**What it would cost to change later.**
- Low. A single line in `boundary_draft::take_pick` can pass `setup_of(state).as_deref()` to `extend_to_loop_in` after `#2149` merges, or `extend_to_loop` and `extend_to_loop_in` can be folded into one function.
