## #2118 - Review the guide after PR #1995

**Chosen.** Update the existing guide pages for every user-visible PR
merged after #1995 (through upstream/main `ce59ef446`). The AI Chat pane
is a section of the MCP page, not a new page.

**Why.** The pane has no network transport yet (#1945 is open), so a page
of provider setup and worked CAM examples (#1949) would describe controls
that do not exist. The section states the shipped controls and the limit.
A new page would also change the navigation test's page list for little
text.

**Dropped.** #1949 stays open and blocked: provider setup, key storage and
worked examples wait for #1942, #1944, #1945 and #1947. Library-only
changes (DXF reader, AC kinematics solver, MCP schema mapper, shape-mask
verifier) have no user control and need no guide text. Unmerged PRs are not
documented.

**Cost to change later.** The AI Chat section must be rewritten when the
transport ships; its "no live transport endpoint" line is pinned by
`GuideAfter1995` so the test fails with that rewrite.
