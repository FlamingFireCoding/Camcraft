## #2003 - Keep the tool editor width stable

**Decided:** Align oversized unit-bearing numeric controls toward the
start of their allocated space in `fields::num_sized`. Keep their existing
width budgets, units, precision, and editing behavior. Pin the tool editor
width across 100 frames with pointer movement using UI value tests rather
than a pixel snapshot.
Four value pins cover existing tools, new tools, long metric feeds, and
long inch feeds; long values exercise overflow in both unit-formatting paths.

**Why:** The feed `DragValue` can exceed its 96 px width budget when its
number and unit suffix are drawn. `ui.add_sized` centers an oversized
control, moving its left edge outside the column. That overflow feeds the
window's next-frame width and grew it by 1.8125 px per frame in the
reproduction. An explicit allocation with the same layout direction and
justification, but both alignments at the start, lets excess width extend
forward without moving the column's origin.

**Considered and dropped:** Widening the field column did not stop the
growth. A larger fixed control budget would fail again for longer feed
values. A whole-window cap would clip the symptom. Removing the footer
separator or title bar did not address the cause. Blanket changes to the
other dialogs would expand the scope without evidence of the same defect.
The source audit found explicit default sizes or widths in the machine
editor, G-code viewer, post refusal, and Debug info windows. The fixture
library and unsaved prompt already cap their content widths. The G-code
viewer also bounds its trace columns explicitly. This audit does not claim
runtime coverage for those other dialogs.

**Cost to change later:** Adjust the numeric-control allocation helper
and its UI value tests if the field layout changes. The tool editor's
column widths remain unchanged.
