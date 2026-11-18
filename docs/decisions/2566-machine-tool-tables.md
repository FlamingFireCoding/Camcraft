## #2566 - Import and export PathPilot (.csv) and Hurco WinMax (.TCM) tool tables

**Decided:** Add a shared machine tool-table codec layer in `camcraft-tools`
(`crates/tools/src/tool_table*.rs`) with a minimal pure-Rust XML reader/escaper
(`tool_table_xml.rs`, no new crate dependencies) and a document-unit bridge in
`crates/app/src/tool_table.rs`. Extend `LibraryTool` with `number: Option<u32>`
and extend `ImportTools` (`crates/app/src/commands/tool_import/mod.rs`) to preserve
controller `T<n>` numbers, refuse tool-number clashes by name, allow number
swaps and reassignments within Update/Replace imports, and apply everything in one
undo step. Keep `.camtools` JSON (`units: "mm"`, no controller numbers)
unchanged. Default controller table units to the document's active units in GUI
(`state_tool_library.rs`), MCP (`import_tool_library`, `export_tool_library`), and
job scripts (`cam.importTools`, `cam.exportTools`), with optional `format` and
`units` overrides in MCP/scripts.

For Tormach PathPilot `.csv`: require the row-1 `Mill Tool Table` marker, bound
numbers to `1..=1000`, parse optional `Description` tokens (`R:`, `loc:`,
`<n>FL`, tool material, `[part]`), keep `Description` as tool name, ignore
machine `Z offset` placeholders, treat negative `Max RPM` values (such as
`-2.0` on probes) as unstated, skip empty slots, refuse bad rows or non-mill
export kinds by name, and export with zeroed offsets.

For Hurco WinMax `.TCM`: read/write UTF-8 XML with BOM and CRLF under
`<Hurco_Document Version="1.1">`, strictly map documented `#30001` `ToolType`
codes (`0,2,7,11,12,13,14,15,16,18,20,21,22,26`) and refuse unsupported codes
(`1,3,4,5,6,8,9,10,17,19,23` or unknown) by name, map `AbsoluteGaugeLength` onto
holder `gauge_len`, and omit Hurco control state (`UsedTime`, `Flags`,
`GeometryFlags`, probe offset indexes) on export.

**Why:** Keeping parsers/writers in `camcraft-tools` decouples table codecs
from Flecs ECS while sharing tool geometry and sanity rules. Refusing
unsupported Hurco `ToolType` codes and non-mill PathPilot exports by name
prevents silent cutter substitution. Ignoring PathPilot `Z offset` on import
and zeroing exported offsets avoids confusing machine gauge offsets with tool
stick-out or writing stale offsets back to a machine. Omitting WinMax control
state avoids resetting tool life or probe calibration data on import.

**Considered and dropped:**
- Adding an external XML dependency in `Cargo.toml` (dropped for one narrow,
  self-contained XML format).
- Mapping unsupported Hurco cutters such as keyseat or dovetail tools to
  `ToolKind::EndMill` (dropped because wrong cutter geometry is unsafe).
- Adding controller tool numbers to `.camtools` JSON or changing its `units: "mm"`
  contract (dropped to keep native `.camtools` files backwards-compatible).
- Emitting guessed zeroed `UsedTime`/`Flags`/`GeometryFlags` tags in `.TCM`
  exports (dropped so partial exports never clobber controller runtime state).

**Limits and later cost:** Acceptance of exported partial `.TCM` files on a
live Hurco WinMax 9.01 control and live PathPilot Hub UI import still need
machine verification. All format-specific field mappings sit inside
`crates/tools/src/tool_table*.rs`, so adding fields or relaxing version headers
from real machine captures is isolated to that module.
