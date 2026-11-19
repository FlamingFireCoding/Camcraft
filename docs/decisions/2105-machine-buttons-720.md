## #2105 - the machine page button row fits a 720 px window

**What was decided.** The "Refresh" button moves from its own prop_grid
row onto the "Library" combo row. The row is built right-to-left, so the
combo cannot push the button out of the cell. The page content gets about
28 px more; the overflow measured 8 px.

**Why.** The machine page in a 1280x720 window pushed the bottom of the
button row 8 px below the properties leaf. The user had to scroll to reach
"New", "Edit" and "Clear". Removing one stacked row reclaims the height
without changing any behaviour. The right-to-left order keeps the button
visible when the combo label is long ("Tormach 770MX Microarc 4").

**Considered and dropped.**
- A smaller row spacing for the whole page: drops, it moves a shared
  constant and changes every page.
- Putting "Refresh" next to "New/Edit/Clear" in one three-button row:
  drops, the four buttons do not fit 330 px of panel width.
- Making the panel scroll-by-default: drops, the pin test says the
  buttons must be visible without scrolling.

**Cost to change later.** Low. The pin test
`the_machine_page_buttons_fit_without_scrolling_at_720_px` and the
`layout_page_setup_machine` anchors encode the row; any new machine-file
button must join that row or a re-flow must keep the row height.
