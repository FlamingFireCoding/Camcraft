## #3644 - Tool components: the document component table and the assembly commands

**What was decided.**
- Storage: two schema components in `crates/app/src/schema.rs`.
  `DocumentComponents` is a world singleton with one row for each
  component id. A row (`DocumentComponent`) keeps the #3642
  `ToolComponent` and the #3643 `ComponentGeometry` together, so the
  catalog fact and its ports, cutting point and body cannot drift apart.
  `ToolAssembly` sits on a tool entity. It is the tool's stack: for each
  node, a component id and its `Mount` (`None` for the root).
- Ids are the component's own trimmed id (`ToolComponent::id`). The
  document makes no ids. A link names a row by the exact trimmed id.
  `library_ref` and id creation in a user library stay with #3645.
- `crates/app/src/tool_assembly.rs` reads and writes the table and the
  stacks, lists the users of a component (`users_of`), and derives a
  tool (`derive`). `derive` calls `camcraft_tools::tool_assembly::resolve`.
  A solid cutting item gives its own `Tool` record. An insert gives the
  current record with the cutting diameter and the corner radius of the
  resolver; the other fields (kind, flutes, cutting data) stay, because
  #3646 owns the full record of an indexable cutter.
- The body is `tool_body::build_spun(spun)` with the resolver's judged
  bands as its profile, and the source `tool assembly`. When the
  resolver can judge a united body but cannot draw it as one outline,
  the stack is refused by name. A body with no facets for display was
  dropped: each reader of `ToolBody` expects facets.
- Four undoable commands in `crates/app/src/commands/tool_assembly.rs`:
  - `UpsertDocumentComponent` adds a row. When the id is held already,
    it is an `EditDocumentComponent`, with the same law.
  - `EditDocumentComponent` replaces a row and derives again each tool
    whose stack names the id. Each tool goes through every `ChangeTool`
    law (`ChangeTool::assembled`), judged against the body that lands
    with it. One refusal refuses the whole edit, with the component id
    and the tool name. The table, each `ToolDef` and each `ToolBody`
    land in one undo step. An edit equal to the row is refused (no empty
    undo step).
  - `DeleteDocumentComponent` refuses a used component and names the
    tools.
  - `SetToolAssembly` sets a stack and lands the derived records in one
    undo step. `SetToolAssembly::clear` removes the stack and keeps the
    records.
- Staleness is not written. The tool record and `tool_body::digest` are
  in each operation's stamp, so the operations of each changed tool go
  Stale by themselves.
- `DeleteTool` now keeps the stack and restores it on undo. It moved to
  `commands/tool_delete.rs` to keep `commands/tool.rs` under 400 lines.
- The project loader clears the table, so no row of the previous
  document leaks into the next one. The table and the stacks are not in
  the project file yet (#3645). The derived records are: the `ToolDef`
  and the drawn body save as today.
- The turning insert of the issue's "Done means" is replaced by a
  milling insert in two indexable cutters. The resolver has only the
  spindle side (#3643). `derive` refuses a lathe-held tool by name.
- No MCP tool in this PR. #3647 owns the component MCP tools.

**Why.** The issue asks that a shared component edit updates every tool
in one undo step and never leaves an invalid assembly. Each derived
record goes through the laws that a hand edit takes, so an assembly
cannot pass a check that the same numbers typed by hand fail.

**Considered and dropped.**
- A separate table entity with one child entity for each component.
  Dropped: one singleton with a snapshot restore is simpler, and the
  table is small.
- A refusal of `ChangeTool` and `SetToolBody` on a tool that has a stack.
  Dropped for this slice: the next component edit derives the records
  again. #3648 (the UI) decides how a hand edit and a stack live
  together.
- An upsert that refuses a held id. Dropped: the name says insert or
  update, and the update takes the edit's whole law.

**Cost to change later.** Low. Only the commands and the tests read the
table. #3645 adds the file format over the same rows.
