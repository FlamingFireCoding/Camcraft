## #2217 - camcraft-gcode crate for RS-274/ISO 6983 3-axis milling

**What was decided.**
- Created `crates/gcode` (`camcraft-gcode`) as a pure-Rust library beside `crates/post`, depending only on `camcraft-cl`, `camcraft-geometry`, and `thiserror`. Registered `"crates/gcode"` in root `Cargo.toml` and added `"gcode"` after `"cl"` in `xtask/src/plan.rs` (`BATCH_CRATES`, `ALL_CRATES`, `CLIPPY`) plus updated `xtask/src/tests.rs` to count `plan::ALL_CRATES.len()`.
- Accepted caller-supplied `ParseOptions` combining `Dialect` (`default_arc_center`, `default_unit: None` by default so missing `G20`/`G21` refuses unless overridden, `cycle_dwell_unit`, `g4_dwell_unit`, `arc_radius_tolerance_mm`, `chip_break_retract_mm`, `initial_position`, with presets `rs274`, `fanuc`, `haas`, `tormach`, `grblhal`, `tinyg`, `carvera`) and `ReferencePositions` (`g28`, `g30`, `g53_origin`, per-WCS `wcs_origins`).
- Implemented lexer, block modal-group conflict checker, arc solver (`I`/`J`/`K` in `G90.1`/`G91.1` and signed `R` in `G17`/`G18`/`G19`), canned-cycle expander (`G73`, `G81`, `G82`, `G83`, `G84`, `G85`, `G86`, `G89` with `G98`/`G99` and `L` repeats into explicit `cl::Move` chains and `cl::Cycle` records), tool-change section splitting (`T`/`M6`), and 1-based source line mapping (`ParsedSection::move_lines` and `LineMap`).
- Refused unsupported features and malformed blocks by name with the 1-based line number: unknown words, missing unit mode (`G20`/`G21`), `G41`/`G42`/`D`, `G43.4`, `G93`/`G95`, macro calls/variables (`G65`/`G66`/`G67`, `M98`/`M99`, `#`, `[`, `]`, `=`, `IF`/`GOTO`/`WHILE`/`SUB`), probing (`G31`/`G38.2`), `R` full circles, missing `G28`/`G30`/`G53` reference positions, and rotary words (`A`/`B`/`C`).
- Recorded new internal crate edges:
  - `camcraft-gcode -> camcraft-cl`: imports `cl::{ArcPlane, Cycle, CycleKind, MoveKind, PathBuilder, Toolpath, lattice}` (`>= 5` items). Caller cannot pass these in because `camcraft-gcode` constructs `cl::Toolpath` sections and `cl::Cycle` metadata directly as its parsed output product. Rebuild impact: changing `camcraft-cl` rebuilds `camcraft-gcode`.
  - `camcraft-gcode -> camcraft-geometry`: imports `geo2d::Point2` and `geometry::{Point3, units::{from_display, Unit, MM_PER_INCH}}` (`5` items). Caller cannot pass raw floats instead because `Toolpath` endpoints, arc centers, and exact micron-lattice inch conversion (`from_display` / `lattice`) bind to `camcraft-geometry` types. Rebuild impact: changing `camcraft-geometry` rebuilds `camcraft-gcode`.

**Why.**
- Keeping `camcraft-gcode` free of `boa_engine`, `flecs_ecs`, and `camcraft-post` lets callers, simulator round-trip tests, and verification tooling parse posted NC programs with minimal dependencies and fast incremental builds.
- Splitting full 360° `I/J/K` circles at the opposite point into two 180° `push_arc_in` calls preserves them because `cl::PathBuilder` treats zero in-plane chord arcs as straight moves; aligning `SectionBuilder::push_arc` on `moves_len()` before/after automatically attributes both halves and any vertical Z-extremum splits to the same 1-based source line.
- Refusing unconfigured `G28`/`G30`/`G53` reference positions instead of guessing `(0, 0, 0)` avoids silent coordinate shifts when machine home is off work zero.

**What was considered and dropped.**
- *Depending on `camcraft-post` for `Dialect` presets*: rejected because it would pull `boa_engine` into `camcraft-gcode`; instead `camcraft-gcode::Dialect` carries the small subset of controller rules (arc center default, `P` dwell units) and later callers in `camcraft-post` or `camcraft-app` can map a post machine/dialect onto it.
- *Evaluating fanuc macro variables (`#`) or `M98` subprograms in v1*: rejected so the 3-axis parser stays deterministic and never mis-executes control flow; left cleanly separated for a future macro preprocessor if needed.
- *Silent fallback to `G21` mm when unit mode is absent*: rejected so unitless programs cannot scale inches as millimeters silently.

**Continuation on current main (qsmits, 2026-11-23).**
- Kept commit `711314efd` (Mike Kinney) by a merge of it into main `60376ccbb`. The three conflicts were lists only: the root `members` line and the `BATCH_CRATES` / `ALL_CRATES` / `CLIPPY` lines in `xtask/src/plan.rs` now hold both `"gcode"` (after `"cl"`) and main's `"gpu"`; the release step count in `xtask/src/tests.rs` keeps main's known-fail `checks` term and the held `plan::ALL_CRATES.len()` term.
- Main's later xtask tests list the crates by hand. `heavy_plan_tests::reach_follows_the_crate_dependencies` now expects `gcode` in the reach of `geometry` (the edge `camcraft-gcode -> camcraft-geometry` puts it there). `step_tests::batch_prebuild_leaves_suite_steps_fresh_with_truck_dev_dep` writes a stub `crates/gcode` for its fixture workspace.
- Both test files name their modules in a `Gate map: covers camcraft_gcode::{..}` line, so quick selects them for a change of any `crates/gcode/src` module. Added `tests/degenerate_input.rs`: malformed programs (empty, huge numbers, zero radius, `Q0` pecks, huge `L` repeats, bad comments) give a value or a refusal, with no panic and no hang (5 s limit for each case).
- Rebuild impact of the two edges, restated: a change in `camcraft-geometry` or `camcraft-cl` rebuilds `camcraft-gcode`. No crate depends on `camcraft-gcode` yet, so a change in it rebuilds nothing else. The batch, release and clippy plans each get one more crate.

**What it would cost to change later.**
- Adding a new dialect field or mapping from `camcraft-post` / `.cps` headers is additive on `Dialect` with `..Dialect::rs274()`.
- Supporting cutter compensation or macro expansion later only requires extending `block_codes` and `lexer` behind explicit `ParseOptions` opt-in without altering existing 3-axis section output.
