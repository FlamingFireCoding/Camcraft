## #1926 - "Set WCS from features" moves to the WCS page

**Decided.** The group "Set WCS from features" (the "Pick for WCS" button,
the role box for Origin / X / Y / Z, the picked list and Apply) is drawn on
the WCS page, in its own group directly above "Work offset (G54-G59)". The
Setup page no longer draws it. The Setup page keeps "Active setup", "Stock
source", "Part orientation" and "This setup". This supersedes the part of
`1648-wcs-page-work-offset.md` that kept the row on the Setup page.

The commands do not change. Apply queues the same `SetWcs`, so undo and redo
are as before. No new command exists, so no MCP tool is owed (the MCP tool for
this action, `wcs_tools.rs`, exists).

**Why.** The maintainer said, in chat on 2026-11-15, after he read the
screenshots of PR #1717: "The issue of moving WCS setup allowing for picking,
axises and origin on setup page, moving it to the WCS pane instead.. is that
in there? cuz screenshots i look at does not have that functionality moved".
He then marked the group titled "Work offset" on main: it holds the "Set WCS
from features" row and the "Work offset" dropdown, and both go to the WCS page.

**Pick gate.** A viewport pick lands in the draft while `wcs_pick` is
collecting. The per-frame sweep (`axis_pick::disarm_hidden`) pauses a draft
whose row was not drawn. The row marks itself drawn (`setup_pick_rows`), so
the gate follows the row, not the Setup page. The WCS page draws the row for
the active setup, so the picks work there. A page without the row pauses the
draft, as before.

**Considered and dropped.**
- Put the group first on the WCS page. The maintainer asked for it near the
  Work offset group, so it sits there.
- Keep the row on both pages. This does not meet "Done means".

**Cost to change.** One call in `pages.rs`, one group constant in
`wcs_pick_row.rs`, one group less in `setup_settings_page.rs`.
