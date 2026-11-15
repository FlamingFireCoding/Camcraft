## #1910 - Document the current Bore detection rule

**Decided:** Update only the `detect_holes` skill row to describe concentric
rings, the 4096-ring budget, an unusable floor band, and the refusal that
stores nothing when every detected hole is skipped.

**Why:** PR #1839 (#1566) removed the single-ring width limit. The row must
match `bore_reach.rs` and `bore_core.rs` so agents can use wide Bore holes.

**Dropped:** Changing runtime behavior, which already implements this rule,
and adding a Rust test that matches documentation text. A temporary text pin
checks the correction while keeping this PR documentation-only.

**Cost to change later:** Update this row when the kernel's Bore rules change.
