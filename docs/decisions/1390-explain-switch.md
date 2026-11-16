## #1390 - Explain G-code switch

**Decided:** Add the Explain G-code checkbox to the shared post section used
by the document and setup post pages. Queue `CommandKind::SetPostOptions`
and use its existing routing to `SetPostOptions` or `SetSetupPostOptions`.
Keep the longer-program consequence visible beside the control.

**Why:** The existing command history already provides undo and redo and
keeps setup option edits scoped to their intended owner. The engine option
from #1386 and the MCP tools and skill entries from #1534 already expose
Explain, so reuse that MCP surface and extend its named-setup test. Pin the
missing UI control structurally by role, name, checked state, undo/redo,
and setup isolation. An original one-block CPS fixture proves that the
switch changes comments while preserving executable text.

**Considered and dropped:** A new command or MCP tool would duplicate the
existing surface. Engine changes would extend this issue beyond its UI
scope. Hiding the checkbox for individual posts would duplicate capability
rules; the engine keeps its named refusal when a post cannot explain its
output.

**Cost to change later:** Wording and placement are local UI changes. A
different ownership rule would change the shared command routing and its
tests; it would not require another Explain engine option or MCP tool.
