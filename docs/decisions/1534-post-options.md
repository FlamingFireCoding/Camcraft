## #1534 - Expose MCP get_post_options and set_post_options tools

**Decided:** Add `get_post_options` and `set_post_options` in `crates/app/src/mcp/post_options_tools.rs` over `SetPostOptions` and `SetSetupPostOptions`, registered in `crates/app/src/mcp/tools.rs`, and document both in `.claude/skills/camcraft-mcp/SKILL.md`. Omit `setup` (or pass `null`) to target the document's `PostOptions`; pass `setup` to target that declared setup's own options (`follow: true` returns it to following the document, `follow: false` snapshots the resolved options as its own). Expose all seven `PostOptions` fields (`skip_stops`, `per_op`, `threading`, `end_units`, `smoothing`, `between_setups`, `explain`), and omit `setup` from implicit active-setup routing.

**Why:** Issue #1534 asks for explicit document-vs-setup targeting like `load_machine` so multi-setup sessions never suffer #1526-style bleed from an omitted `setup` silently editing an active setup. Routing mutations through `SetPostOptions` and `SetSetupPostOptions` keeps undo/redo and no-op refusal semantics identical to the GUI. Exposing all seven `PostOptions` fields on `schema::PostOptions` leaves no unreadable post option behind.

**Considered and dropped:**
- Routing omitted `setup` to the active setup (`setup_or_active`): dropped because #1534 explicitly forbids implicit routing to prevent #1526-style bleed.
- Embedding post options inside `get_job` instead of adding `get_post_options`: dropped so agents can inspect and verify document or per-setup `follows_document`/`own`/`options` without parsing the full job dump (and `set_post_options` returns the same shape).
- Treating `null` option fields as omitted: dropped because explicit `null` on a boolean or enum option is likely a caller mistake and should be refused by field name rather than silently ignored.

**Cost to change later:** Low. The wire schema and command dispatch live in one isolated module (`crates/app/src/mcp/post_options_tools.rs`).
