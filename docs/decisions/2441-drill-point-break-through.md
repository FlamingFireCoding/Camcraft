## #2441 - through drills break through to full diameter by default, sim cuts the drill point cone, and MCP lists break_through

**What was decided.**

1. *Through-hole tip depth (`BreakThroughFrom`).* `Drill` adds `BreakThroughFrom` (`#[default] FullDiameter` vs `Tip`) alongside `break_through`. When `break_through_from` is `FullDiameter` on a `ToolKind::Drill` (outside `Spot`, `Tap`, `FloatingTap`, and `Ream` cycles), `through_tip_run` adds `drill_point_length(d, tip_angle) = (d / 2) / tan(tip_angle / 2)` (`180 deg -> 0.0`; non-finite or outside `(0, 180]` refused by name) to `through_clear` so the drill's cylindrical margin exits the bottom of the hole (`G73 ... Z-2.803` for a Ø6 118° drill with 1.0 mm break-through, instead of stopping its tip at `Z-1.000`). Lathe axial drills stay on `Tip` (`THROUGH_CLEAR`), because `axial_doors` limits spindle-axis holes against the chuck end.
2. *Drill-point simulation.* Added `SdfTool::Drill { r, cot, flank }`, `RevKind::Drill { r, cot, flank }`, and `Stock::simulate_drill` / `Stock::simulate_drill_display` (carried/held/field/rest variants across `sim` and `app`). A drill now removes only its conical point `[0, r * cot]` plus cylindrical flank above `r * cot`, instead of plunging a flat bottom disc across the full diameter; stopping a Ø6 118° drill tip flush at `floor_z = 0` now leaves the ~1.5 mm conical lip inside the bore and reports it as rest.
3. *Project v68 migration.* Bumped `PROJECT_VERSION` from 67 to 68. Pre-v68 `.camproj` files without an explicit `break_through_from` migrate to `Tip` (`settle_drill_break_through`), keeping existing saved projects and snapshot goldens (`examples/first-part.camproj`) byte-identical while newly created operations and MCP/script patches default to `FullDiameter`.
4. *MCP schema & UI.* Exposed `"break_through"` (`type: "number"`, `default: null`) on `"drill"`, `"bore"`, and `"thread"` in `list_operation_types` alongside `"break_through_from"` (`"full_diameter"` | `"tip"`), and added the `Break-through from` selector (`Full diameter` / `Tip`) to `holes_page.rs`.

**Why.**

Twist drills taper to a point (118° standard, ~1.803 mm on a 6 mm drill). Stopping the drill tip only 1.0 mm below the hole floor stopped the cylindrical cutting margin ~0.803 mm above the exit edge, leaving an uncut ring around through holes. Modeling the drill as a flat-end cylinder in column and field stock simulation hid that uncut lip completely. Separately, `break_through` was skipped by `kind_defaults()` because its default is `None`, so `list_operation_types` omitted it on hole operations.

**Considered and dropped.**

- *Changing `break_through` directly for all loaded pre-v68 files to `FullDiameter`.* Existing saved projects and UI snapshot goldens (`examples/first-part.camproj`) already expect their saved depths; migrating absent v<68 fields to `Tip` preserves saved behavior without forcing old files stale, while every new drill gets `FullDiameter`.
- *Adjusting `break_through` silently only inside `generate` without a user-facing parameter.* Machinists intentionally choose whether `Break-through` measures past the drill point (`Full diameter`) or the tip (`Tip`) depending on spot/pilot holes and spoilboard clearance; `BreakThroughFrom` makes the choice explicit in parameters, UI, and MCP.

**Cost to change later.**

Low and isolated. `BreakThroughFrom` is a two-variant enum with a single helper (`through_tip_run`) and one v68 loader check (`settle_drill_break_through`). `SdfTool::Drill` / `RevKind::Drill` sit cleanly alongside `SdfTool::Cone` in `crates/sim` and are dispatched only for `ToolKind::Drill` with `tip_angle < 180 deg`.
