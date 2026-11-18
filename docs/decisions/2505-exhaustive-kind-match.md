## #2505 - The tool kind rules name each kind, with no catch-all

- **Decided:** `Tool::kind_rule` in `crates/tools/src/tool_sanity.rs` has no `_ => None` arm. Its last arm names each kind that has no rule, or whose guarded rules held. A new `ToolKind` does not compile until this match names it.
  - **Why:** with the catch-all, a new kind passed the check with no kind rule, and no test saw it.
- **Decided:** the test list `KINDS` is guarded by `kind_slot`, an exhaustive match in the test file. A new kind does not compile until it gets a slot. Then `the_sweep_list_holds_each_kind_once` fails until `KINDS` holds it.
  - **Dropped:** a `ToolKind::ALL` constant in `crates/tools/src/tools.rs`. That file has 691 lines, over the 400-line limit. A hand-written constant also does not grow by itself when a kind is added.
- **Not changed:** the oracle `old_is_sane` keeps its `_ => true`. It is the old law, verbatim, and the sweep compares against it. The verdicts for the 18 kinds do not change.
