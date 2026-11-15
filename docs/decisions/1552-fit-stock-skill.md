## #1552 - Document the fit_stock MCP tool

**Decided:** Add the shipped `fit_stock` tool beside `set_stock` in the MCP skill. Document the schema, defaults, millimetres, side order, parts-only scope, preserved rotation, single undo step, response and named refusals.

**Why:** Agents need the same stock-fit contract the MCP implementation already exposes. The row follows `stock_fit_tools.rs`, its parameter parsers and `stock_fit.rs`.

**Considered and dropped:** No runtime changes or new dependencies are needed. A separate guide would split the tool reference. Keep the process version and #681 merge announcement with the merging session, as the issue requires.

**Review decision:** Remove the xtask documentation-only pin and keep this PR docs-only, as the pre-review suggests. The pin did not compare the tool schema, did not run for later skill-only changes, and made the PR ineligible for the PR-test runner. Moving or expanding the pin was considered and dropped because it would add test or gate scope to this documentation fix. Add the exact `Process change: yes` line to the PR body.

**Validation:** Temporary review-fix checks pinned the PR body marker and the absence of the introduced xtask test. Each check failed with its correction reverted and passed after restoration. The quick gate passed with a docs-only plan (no tests selected), and xtask clippy passed locally.

**Cost to change later:** Edit the row when the tool contract changes. A future schema-aware documentation check would need its own test location and gate design.
