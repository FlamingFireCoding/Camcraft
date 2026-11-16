## #2015 - Tool cutting data by work material: the Unknown row, stepdown and stepover, the stock's material

**The `Unknown` row is `material: None`, not a `WorkMaterial::Unknown` variant.**
Decided: `CuttingRow::material` became `Option<WorkMaterial>`; `None` is the
`Unknown` row and the files write it as the word `unknown`. Why: `WorkMaterial`
indexes the handbook table (`WorkMaterial::row()` is `TABLE[self as usize]`,
the turning table likewise); a variant with no handbook row would make every
document-material path a possible out-of-bounds index, and "no material
stated" already reads `None` everywhere. `row_for(rows, None)` now answers the
`Unknown` row, which is exactly "the row for no stated material". Dropped: the
variant the issue names (word kept, type differs). Cost to change: one type,
~20 literals.

**Selection order and what moves.** The row in force is the one naming the
material, else `Unknown`. The material for an operation is its setup stock's,
else the document's (`stock_material::in_force`). The tool's running rpm/feed
(what the post's S word reads) follow the document material only: a tool is
one entity shared by every setup, so a per-setup spindle speed would need
per-operation spindle words - out of scope, noted as a follow-up. An
operation's milling feed still moves only for a row that NAMES the material
(#890's rule): with the `Unknown` fallback applied to op feeds, picking a
material would have overwritten every hand-set op feed with the tool's own
feed. Stepdown/stepover come from the row in force, each missing one from the
`Unknown` row. Considered: falling back for op feeds too (dropped, above).

**Behaviour change, inside a command only.** `SetWorkMaterial` to a material
the tool has no row for now runs the tool's `Unknown` row instead of leaving
the previous material's numbers in place (the `tool_cutting_rows` test that
pinned "numbers stay" was updated). For a project with no material rows this
is a no-op: the `Unknown` row equals the running numbers.

**The `Unknown` row follows hand edits.** A `ChangeTool` that moves the running
rpm/feed without touching the table, while `Unknown` is the row in force, moves
the `Unknown` row with it - otherwise a later material change would put back
numbers the user had replaced. A row's seed (`SetWorkMaterial`) uses
`ChangeTool::seeding`, which does not. The editor shows the derived row; handed
back untouched while the running numbers changed, it stays derived and follows
them.

**Canonical form: the `Unknown` row is stored only when it says something.**
Every tool has an `Unknown` row (`unknown_of`), but a row that only repeats
the tool's own rpm/feed is derived from them, not stored - in memory and on
disk (`cutting_rows::canonical`, applied by `AddTool`, `ChangeTool`, the
reader and the tool editor's draft). A command that moves the running numbers
away from a derived row (`SetWorkMaterial` seeding a material row) stores it
first, so the default is kept. Why: storing it always made every tool differ
from the definition it was created from (16 existing round-trip and undo
tests compared them), changed every saved tool, and changed MCP `get_job`
for every tool; the derived form keeps all of that byte-identical and still
reads v62 projects and v1/v2 `.camtools` files with their own rpm/feed as the
`Unknown` row. Considered: always storing it (dropped, above). The project
still steps to v63 (stock material, steps, a stated `Unknown` row must not be
dropped by an old reader). A `.camtools` file is written as version 3 only
when it holds a stated `Unknown` row or a step; otherwise it keeps 1 (or 2
for left-hand taps), as #1453 did.

**Kinds without the row.** A waterjet (rows are thickness recipes) and an
electrode (rpm 0, which a non-torch row refuses) get no `Unknown` row: the
derivation adds it only when the row would pass the table's own refusal, so a
sane tool never turns insane.

**Steps are seeded on these kinds only:** pocket, adaptive, offset, slot, 3-D
pocket, multi-axis rough (stepdown and stepover), face (stepover; stepdown
only when staged) and 2-D contour (stepdown). A row's stepover wider than the
cutter is refused by tool name.

**Stock material lives on the stock entity** (`StockMaterial` component, like
#1094's `StockSourceOnly`): cloned with a setup's datum, restored by the undo
of a stock delete or setup delete, saved as the stock entity's
`stock_material` word (an unknown word refuses the file by name).

**MCP:** `set_stock_material` added. The camcraft-mcp skill row is left to a
follow-up issue (a skill edit is a process-file change with a
`PROCESS_VERSION` bump).
