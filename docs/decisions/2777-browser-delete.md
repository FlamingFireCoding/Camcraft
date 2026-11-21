## #2777 - Delete models and tools from the browser menu

**What was decided:** Add Delete to the model and tool context menus. Route
the clicked model to `DeleteByName` and the clicked tool to `DeleteTool`.
Reuse the existing menu renderer. Add a tool menu target with its name and
row so the menu stays attached to the clicked item.

**Why:** These commands already provide undo and redo. A menu action must
delete its own target even when another item is selected. The shared menu
renderer keeps the existing pointer handling and command dispatch.

The existing `DeleteTool` command permits deletion of a referenced tool.
The operation then shows `BLOCKED` and names the missing tool. Undo restores
the tool and clears that block. A regression test pins this behavior. This
change preserves that command behavior rather than adding a refusal policy
outside the menu scope.

**Considered and dropped:** Direct world changes would bypass command
history. Selection-based deletion could remove a different item. A second
menu renderer would duplicate interaction code. A new command and MCP tool
are unnecessary because the existing commands perform these actions.

**Cost to change later:** Menu labels and command dispatch remain small
changes in the shared menu. A different tool deletion policy would require
a separate command change and tests for its affected callers.
