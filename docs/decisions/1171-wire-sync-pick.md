## #1171 - Wire sync pairs picked in the viewport

**Decided.** The pick is armed by a button on the Wire Contour page
(Geometry tab, pairing by sync points). While it is armed, a vertex click
states one sync point in the page draft. The first click of a pair is the
upper point. The second click is the lower point and must lie below the
upper one. A flat or higher second click is refused by name.

**Why.** A vertex click already routes to the operation's geometry list.
An explicit armed state keeps the two uses apart. The draft rides Apply
(`EditOperationParams`), so undo works and no new command is needed. For
that reason no `MCP:` follow-up is filed: an agent states the pairs with
the existing parameter tool.

**Considered and dropped.**
- Always-on sync picking while a 4-axis op is open: it would steal the
  clicks that select the two profiles.
- A new command per pick: the draft already has Apply and undo.
- Choosing upper and lower by profile membership: the page cannot know the
  profiles before generation, so the Z order of the two vertices decides.

**Cost to change.** Small. The pick lives in `crates/ui/src/wire_sync_pick.rs`.
