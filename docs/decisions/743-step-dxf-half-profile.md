## #743 - STEP-to-DXF half-profile conversion and DXF half-profile writer/importer

**What was decided.**

- **Clean-room ASCII DXF writer (`crates/geometry/src/dxf/{writer.rs,units.rs,mod.rs}`).**
  Added `dxf::write`, `dxf::write_bytes`, and `Unit::code()`, pairing with the
  existing reader (`#1860`) to emit `HEADER` (`$INSUNITS`), `TABLES` (`LAYER`),
  and `ENTITIES` (`LINE`, `ARC`, `CIRCLE`). Invalid coordinates, zero-length
  `LINE` entities, non-positive or `>= 2*pi` `ARC` sweeps, unsupported
  `$INSUNITS` codes, duplicate or newline-containing layer names, and limit
  overflows are refused by entity or section name without panicking.
- **Conservative stepped-band reduction (`crates/tools/src/spun_envelope.rs`).**
  Added `SpunProfile::conservative_from_bands(&[HolderBand])` because a full
  `MAX_BANDS` (`32`) `HolderProfile` produces `2 * 32 + 2 = 66` vertices,
  exceeding `MAX_POINTS` (`64`). Adjacent bands merge greedily by minimum
  added swept volume taking `max(r)` until equal or tiny segments clear
  `MIN_SEGMENT` and band count fits `MAX_STEPPED_BANDS = 31` (`<= 64`
  points), strictly preserving outer radius across non-revolved features such
  as drive slots and flats.
- **STEP / `ToolBody` <-> DXF half-profile pipeline (`crates/app/src/tool_profile_dxf.rs`, `crates/tools/src/tool_profile_dxf.rs`).**
  Exports `SpunProfile` (lines and signed circular arcs) and spindle
  `ToolBody` envelopes to millimetre ASCII DXF (`$INSUNITS = 4`) with an
  explicit `CENTRELINE` `LINE` (`(0, 0) -> (0, z_top)`) and an open `PROFILE`
  chain in `(x = r, y = z)`. Converting a STEP file (`step_to_spun_profile`,
  `step_to_dxf`) routes exclusively through `ToolBodySpec::from_step_auto`,
  refusing missing, degenerate, or ambiguous ISO 13399 coordinate systems and
  non-STEP meshes by name rather than guessing `+Z`. Importing DXF
  (`dxf_to_spun`, `dxf_to_spun_with_unit`) accepts one `LINE` on layer
  `CENTRELINE`/`CENTERLINE` in any 2D orientation plus one continuous open
  chain wholly on one side of the centreline, refusing missing or multiple
  centrelines, `CIRCLE`, `SPLINE` (via reader), crossing geometry, closed
  loops, branching, disconnected segments, unitless DXF (when no fallback unit
  is supplied), axial reversals (`RunsBack`), and below-tip geometry by name.
- **DXF importer ownership and dependency (PR #2100 review fix).**
  The byte-to-`SpunProfile` importer belongs to
  `camcraft_tools::tool_profile_dxf`; the app preserves its existing public
  import paths through re-exports. Parsing, chain ordering, projection, and
  validation use only domain values. Their tests live in `crates/tools/tests/`.
  `SetToolBody` retains the ECS world and command work in app.
  This adds `camcraft-geometry` to the tools crate's dependencies. The importer
  uses `dxf`, `dxf::Document`, `dxf::Primitive`, `dxf::Unit`, `dxf::Units`,
  `dxf::parse`, and `geo2d::Point2`. Its public contract accepts DXF bytes and
  returns a validated tool profile, so it needs the format's parser, units,
  primitive representation, and coordinates together. Passing parsing and
  projection in from app would leave this domain work there; a recorded file
  cannot replace conversion of user-supplied bytes. A geometry change can now
  rebuild tools and its downstream consumers, including app, motion, simulation,
  post, and UI.
- **Commands, MCP, and UI (`crates/app/src/{commands/tool_body_dxf.rs,mcp/holder_dxf_tools.rs}`, `crates/ui/src/{state_tool_profile.rs,egui_app/tool_profile_page.rs,egui_app/tool_body_page.rs}`).**
  Added undoable `SetToolBody` constructors (`from_dxf_bytes`, `from_dxf`,
  `from_step_profile`, `from_placed_profile`) over `tool_body::build_spun`.
  Exposed MCP tools `import_tool_holder_dxf`, `export_tool_holder_dxf`, and
  `convert_tool_step_to_profile`. Updated the Profile tab with an explicit
  notice that non-revolved features are enveloped conservatively (never cut
  away) and only the half-profile is editable, plus `Import DXF…`,
  `Export DXF…`, and `Import STEP…`, and added Body tab `Convert to profile`
  (refusing drafts whose placement is still the unaligned numeric fallback).

**Why.**

Holder collision checking relies on `SpunProfile` never undercutting the
actual 3D tool assembly; merging bands only upward in radius guarantees that
slots, flats, and polygon corners stay enveloped inside the revolved
silhouette. Refusing STEP conversion when ISO 13399 CSYS is absent or
ambiguous prevents silently spinning a holder about the wrong axis. Separating
the new modules (`tool_body_dxf.rs`, `holder_dxf_tools.rs`, and tools'
`tool_profile_dxf.rs`) and appending UI helpers avoids overlapping
edits with open PRs `#2092`, `#2054`, `#2077`, and `#2023`, and keeps every
file at or under 400 lines.

**Considered and dropped.**

- Guessing `+Z` when a STEP file lacks ISO 13399 coordinate systems: unsafe
  for vendor STEP assemblies whose local frame is arbitrary.
- Sampling or truncating STEP bands without conservative `max(r)` merging:
  would cut away flats or drive slots and risk undetected collisions.
- Supporting `SPLINE` or `ELLIPSE` entities in DXF half-profile import or
  export: left out of scope; ASCII `LINE` and `ARC` on explicit `CENTRELINE`
  and `PROFILE` layers represent `SpunProfile` segments losslessly.
- Writing `ToolBody { ... }` struct literals directly in `tool_body_dxf.rs`:
  dropped so open PR `#2092`'s `library` field causes no unnecessary conflict.
- Passing geometry-independent adapters or parsing closures from app to avoid
  the tools-to-geometry dependency: this would duplicate the DXF representation
  or retain domain conversion in app. The direct dependency keeps the complete
  byte-to-profile operation and its tests in the owning domain crate.

**Cost to change later.**

Low. Conversions sit behind dedicated modules and `SetToolBody` constructors,
and saved `.camproj` files continue storing `SpunProfile` without format
migration.
