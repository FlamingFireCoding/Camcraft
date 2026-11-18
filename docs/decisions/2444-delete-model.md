## #2444 - Delete an imported model over MCP

**Chosen:** Add `delete_model {name}` to the existing model tool table.
Execute the existing `DeleteEntity` through session history as one undo step.
Reuse the model picker validation to reject unknown names, non-models and fixtures.
Refuse models referenced by operation sources, selections or area boundaries.
Refuse models referenced by any `FromModel` stock, including setup-owned stock.
Name the dependent operation or stock and leave the document and history unchanged.

**Why:** Agents need to remove duplicate imports without undoing later work.
The existing command already restores model geometry, placement and import fidelity.
Fixtures require `remove_fixture` first so deletion cannot silently drop collision furniture.

**Dropped:** A general `delete_entity` tool would expose unrelated entity kinds and require a broader reference policy. A new command would duplicate the existing undo implementation.
**Future cost:** Supporting more entity kinds requires a separately reviewed refusal policy and matching MCP pins; no project format or dependency changed.
