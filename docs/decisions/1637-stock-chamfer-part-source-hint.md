## #1637 - Stock-source 2D Chamfer refusal names the cause and part-source workaround

**What was decided.** Add `Warning::StockChamferGouge { at: usize }` to
`camcraft_sim::verify::Warning` (blocking post like `Warning::Gouge`,
mapped to `"gouge"` in MCP `verify`, located on `at` in `warned_moves`,
and folded on its own line in UI warning folding). Its `Display` reads
`"move {at} cuts past the finish surface (stock-source chamfer breaks into a part; chamfer with the part as source)"`.
In `operation::keep_law::note_stock_chamfer_gouges`, rewrite `Warning::Gouge`
emitted against kept parts into `Warning::StockChamferGouge` when
`!part_solid` and the op is `OpParams::Chamfer2D`, called from both
aligned and tilted generation arms and `operation::kept_rejudge`.

**Why.** `GenerateOperation`, `posting::post_setup`, `mcp::job::verify`,
and the UI op warning fold all surface `tp.warnings`, not `tp.path.notes`.
Giving the refusal its own `Warning` variant preserves `kind: "gouge"`,
`blocks_post: true`, move-index focus, and existing `"move N cuts past the
finish surface"` assertions while surfacing both the cause and the exact
`"chamfer with the part as source"` workaround across every channel.

**Considered and dropped.**
- Attaching a message to `tp.path.notes`: `mcp::job::verify`, `v_verify`,
  and `post_setup` do not print `path.notes`, so MCP agents would still
  see a generic gouge.
- Adding a free-form `String` field to `Warning::Gouge`: `Warning`
  derives `Copy`, and rewriting only stock-source 2D Chamfer keeps all
  existing gouge sites zero-allocation.
- Refusing at generation with `Err(GenError)`: existing suites (#1631)
  expect the path to generate with blocking warnings rather than fail
  generation, and the PR #1634 review specifically asked for clearer
  op-warning wording.

**Cost to change later.** Remove `Warning::StockChamferGouge` and
`note_stock_chamfer_gouges` if another warning representation carries
per-law hints.
