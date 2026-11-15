## #1871 - Plasma Profile through MCP

**What was decided.** Keep the existing plasma commands and safety gates.
Add a headless MCP suite for the 1300PL workflow, edit undo/redo, named
tool/rpm/post refusals, and a bad cut-order cache. Add the shipped
`Tormach PathPilot Plasma` name to the `post` argument description and
append a dedicated Plasma Profile workflow to the MCP guide.

**Why.** The machine, torch, operation and post already work through the
shared command layer. MCP clients need to discover the post and see both
the successful workflow and the refusals at the protocol boundary. The
schema pin fails before the description change; the workflow coverage
protects behavior that already shipped. Restoring the old description
makes the pin fail again, and restoring the fix makes it pass.

**What was dropped.** No new MCP tool or production safety guard is
needed. No golden refresh is needed: the suite checks emitted NC words
and refused output files without changing the post. The bad-order case
reorders a generated saved cache because the normal generator already
orders holes before outlines; it then reopens the project through MCP.
This corruption is confined to the test fixture and exercises the
existing post-gate backstop without adding a production way to reorder
torch spans. The documentation calls out the 2D plasma verdict and the
absent 3D stock simulation so that clients do not overstate verification.

**Overlaps.** PR #1738 changes WCS registration and guide rows; #1817
adds the Pocket rest-machining guide row. They touch the same files but
different functions or sections. This change appends its own workflow;
whichever merges second refreshes.

**Cost to change later.** Update the one MCP test suite and workflow
section when the tool arguments or plasma verdict change. No file-format
or dependency changes are introduced.
