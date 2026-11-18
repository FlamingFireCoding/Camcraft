## #2439 - Recheck freshness during unnamed generation

**Decided.** Keep every operation in program order for an unnamed
`generate` call. Check its current status immediately before generating
it. Skip it only if it is still current at that point. Named operations
continue to regenerate regardless of status. Each generation keeps its
own command and undo step. A refusal still names the operation and
leaves earlier successful generations undoable.

**Why.** The old loop selected its targets using the statuses from the
start of the call. Regenerating an edited upstream operation can make
later operations stale. Checking freshness when each operation is reached
includes those later operations in the same call.

**Dropped.** Repeating the whole batch until nothing is stale adds an
unnecessary loop because these dependencies point to earlier operations.
Routing the call through the GUI's `RegenerateAll` command would change
first-generation, refusal, and undo behavior. That command already uses
rounds to account for changing predecessor paths.

**Validation.** The three-operation MCP pin fails before the fix and
passes after it. Restoring upstream's `api.rs` makes the pin fail again;
restoring the fix makes it pass. A second pin preserves named generation.
The quick gate passes all 17 steps. App clippy passes for all targets with
warnings denied. No NC or CL goldens changed.

**Cost to change.** Small. The policy stays in `v_generate`, with its MCP
description and integration tests beside it.
