## #2076 - Document `set_stock_material` and `cutting_data` keys in `camcraft-mcp` skill

- **What was decided:** Add the `set_stock_material` row in Section 4 (`Stock & Tool Management`) of `.claude/skills/camcraft-mcp/SKILL.md` right after `set_work_material`. Document `cutting_data` rows carrying `"material": "unknown"` and optional `stepdown` / `stepover` (mm) for `add_tool` and `edit_tool` definitions inside the `add_tool` row. Omit any `.claude/PROCESS_VERSION` edit and include `Process change: yes` in the PR body instead.
- **Why:** Matches the `set_stock_material` MCP handler and `cutting_data` parsing added for `#2015` (PR `#2077`), keeps all workpiece-material rows together in Section 4, avoids duplicating `edit_tool` documentation while `#1343` owns the dedicated `edit_tool` row, and follows the single-writer `PROCESS_VERSION` rule in `AGENTS.md` (since `PROCESS_VERSION` 63).
- **Considered and dropped:** Adding a standalone `edit_tool` row (dropped because open `#1343` owns that row), adding a Rust markdown-matching test to `crates/` or `xtask/` (dropped to keep this PR docs-only like `#1584` and `#1910`), and bumping `.claude/PROCESS_VERSION` on the branch (forbidden by `AGENTS.md` to avoid per-PR merge conflicts).
- **Cost to change later:** Low — a single markdown edit in `SKILL.md`, and `#1343` can move or cross-reference the `cutting_data` note when adding the dedicated `edit_tool` row.

### Review fix - pending implementation dependency

- **What was decided:** Mark `set_stock_material` and the new `cutting_data` inputs (`"material": "unknown"`, `stepdown`, `stepover`) as pending PR #2077. Tell agents not to use them until that PR is merged, and state the dependency in the PR body. After #2077 merges, check these rows against main.
- **Why:** Main does not yet expose `set_stock_material` or accept the new inputs. Keeping the planned shapes with an explicit restriction allows this documentation to merge first without directing agents to unsupported calls.
- **Considered and dropped:** Waiting for #2077 to merge before fixing this review would leave the current skill advertising unsupported inputs. Removing the planned rows would discard the documentation this PR supplies.
- **Cost to change later:** Low: check the rows against main after #2077 merges, then remove their pending restrictions.
