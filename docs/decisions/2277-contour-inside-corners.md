## #2277 - Open-chain 2D Contour joining and inside-corner offsetting

**What was decided**

- Added `camcraft_geometry::boundary_chains::join_open_runs` to deduplicate direction-free XY-coincident runs and weld touching open runs end to end (`group_runs`) while passing single-point and non-finite runs through untouched so generation's own refusal names them.
- Called `join_open_runs` on `chains` in `crates/app/src/operation/selection.rs` (`selection_geometry_impl`) for `OpParams::Contour`, `OpParams::PlasmaProfile`, and `OpParams::WireContour`.
- Replaced `offset_run` in `crates/motion/src/machining2d/contour_run_side.rs` with a shared `pub fn offset_run_side` (re-exported from `crates/motion/src/machining2d/mod.rs`) and delegated both `contour_run_side::offset_run` and `crates/app/src/operation/contour_open.rs` (`offset_run`) to it:
  - Filters `Region::sweep` loops to a single outer CCW loop, ignoring inner CW holes when bottleneck notches bridge their mouth.
  - Uses `corner_sector_left` across `on_interior` and `point_is_left` so convex round-join arcs wrapping past 90 deg classify by turn sign.
  - Breaks ties within `CAP_TIE` (`0.006 mm`) in favor of non-cap interior/join segments when Clipper2's `5 um` arc sag pulls a swallowed end cap chord slightly inside `d`.
  - Inserts exact `normal_at` end points only when `nearest_on_run(p, run).0 >= d - TOL`, preventing swallowed short inside-corner legs from gouging adjacent walls.

**Why**

- `stitch_open_runs` only removes and commits closed loops, leaving manually picked open contour edges as independent runs whose individual offsets overran inside corners into neighboring walls (reproducing `Z 0.000 near (40.000, 10.995)` on the U-notched block).
- Even on a joined open chain, `offset_run` rejected bottleneck CW holes, misclassified >90 deg convex join arcs via `cross`, unconditionally inserted swallowed `normal_at` endpoints on short legs, and dropped cap intersections under `5 um` chord sag.
- `crates/app/src/operation/contour_open.rs` duplicated `offset_run` with the same defects.

**What was considered and dropped**

- Extending `stitch_open_runs` (`crates/app/src/operation/stitch.rs`) to commit open chains alongside closed rings: dropped because `stitched_rings` tracks closed-loop ring chips and `selection_geometry_in` treats `loops` and `chains` as distinct outputs; joining `chains` in `selection_geometry_impl` keeps open-boundary vs closed-ring routing intact.
- Auto-trimming colliding moves inside `clear_of_part`: dropped under the Prime Directive because masking collisions would hide real wall-ending picks; `clear_of_part` remains an unconditional safety check.
- Fixing `offset_run` separately in `contour_open.rs`: dropped to eliminate duplicate offsetting logic.

**What it would cost to change later**

- Low and localized to `boundary_chains::join_open_runs`, the single call site in `selection_geometry_impl`, `contour_run_side::offset_run_side`, and the delegation in `contour_open::offset_run`.
