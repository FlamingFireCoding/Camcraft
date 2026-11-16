## #744 - Editable spun/extruded fixture profiles with DXF and STEP source support

**Chosen:** Add shared ASCII DXF profile import/export and mesh generation in
`camcraft-geometry` (`dxf`, `dxf_profile`, `profile_mesh`, `profile_derive`),
command/persistence/MCP support in `camcraft-app` (`step_source`,
`fixture_profile`, `commands::profile_fixture`, `mcp::fixture_profile_tools`,
`fixture_library`, `project`), and UI dialog/row editing in `camcraft-ui`
(`state_fixture_profile`, `fixture_profile_dialog`, `fixture_profile_rows`).
Fixtures support revolved `Spun` half-profiles around `+Z` (compatible with `#743`
tool `SpunProfile` via `from_tool_spun`/`to_tool_spun`) and closed 2D `Extruded`
profiles along `X`/`Y`/`Z` with positive depth from arguments or DXF
`TEXT`/`MTEXT`/`DEPTH*` annotations. Models and fixtures imported from STEP keep
their raw STEP text in ECS `StepSource`, `.camproj`, and `.camfixtures`;
`SetFixtureProfile` refuses overwriting STEP solids, while
`DeriveFixtureProfile` records an optional reference profile without altering
STEP facets or `StepSource`. Left `egui_app/fixture_menu.rs` untouched to avoid
conflicting with open PR #1687.

**Why:** Rebuilding `Facets` on every profile edit feeds `fixture::meshes_for`,
`fixture::boxes_for`, rapid-strike detection (`fixture_struck`), and post
verification through the existing collision path with no duplicate intersection
code. Keeping raw STEP intact avoids degrading vendor STEP solids into
approximated 2D outlines while still supporting optional reference DXF export.

**Dropped:** Converting imported STEP solids into editable profiles (destroys
exact 3D vendor geometry on re-save), storing profile metadata without updating
`Facets`, or adding external DXF parsing crates (kept dependency-free per repo
policy).

**Cost to change later:** Isolated to the `dxf_profile`/`fixture_profile`
modules, five undoable commands/MCP handlers, and lenient `.camproj`/library
deserializers; multi-region or island profiles can extend `FixtureProfile` in
place while older files fall back cleanly.
