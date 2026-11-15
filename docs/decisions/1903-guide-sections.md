## #1903 - Group the user guide by job stage

**Decided:** Use eight weighted Zola sections: Getting started, Set up the
job, Operations, Check and post, Multi-axis, Automate, Reference and CAM
fundamentals. Move existing pages into those sections and update internal
links. Keep each moved page's old URL as an alias. Make the operations
overview its section index, so `/guide/operations/` stays the same.

**Why:** The order follows a job and makes reference topics easier to find.
Aliases preserve existing links; Zola's redirect also preserves fragments.
Section introductions link to the existing import, stock, work-offset,
simulation, verification and scripting headings instead of repeating them.
The guide index gives one linked start path.

**Navigation:** Read sections and pages from Zola's content tree. Use
native expandable groups and open the current group. This keeps navigation
usable without JavaScript and avoids a separate hard-coded page list.

**Dropped:** Headings over the flat list would not create reusable sections.
Copied pages would give readers and search results duplicate instructions.
Splitting the existing subject pages would expand the change beyond guide
organization. No machining behavior, safety guard or G-code changes.

**Cost to change later:** Moving a page again requires updating its source
links, retaining all published aliases and extending the navigation tests.
Changing the section order requires changing weights and the expected order
in those tests. Existing page headings remain stable for bookmarked links.
