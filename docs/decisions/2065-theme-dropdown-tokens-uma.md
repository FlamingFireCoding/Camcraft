## #2065 - Shared theme metrics for dropdown menus

**What was decided.** Add `menu_width`, `recent_width`, `menu_gap`, and
`menu_pad_x` to the theme vocabulary. Keep the existing defaults of 200,
300, 2, and 8 points. Clamp them to 100–480, 120–640, 0–24, and 0–32 points,
respectively, both when reading files and when deriving drawing geometry.
Use one dropdown geometry helper for ribbon menus, recent files, Theme,
tree menus, Fixtures, and View rows. Fit the requested width to the available
bounds and shift its left edge inside them. A window narrower than 40 points
still owns the width limit; imposing a larger menu minimum would overflow it.
Machine menus retain their expansion for long labels before fitting to the
window. Derive View checkbox size and label spacing from the clamped row
height and horizontal inset. Move the built-in token columns and file overlay
into child modules so the edited theme modules remain near 400 lines.

**Why.** Fixed dimensions prevent a file theme from controlling the whole
menu and let popups extend past narrow windows. Sharing the geometry keeps
the painted panels, interactive rows, spacing, and dismissal gap consistent.
Keeping the defaults preserves the existing appearance without an override.

**Considered and dropped.** Separate metrics for each drawer would repeat
the same policy and permit drift. Fixed popup minimum widths would defeat
window fitting. Removing machine-label expansion would truncate labels even
where the window has enough room. Separate checkbox and clipping tokens are
unnecessary when their geometry can follow row height and horizontal inset.

**Stacking decision.** Stack on PR #1687 because it also changes
`theme_drop::show` and `fixture_menu::fixture_menu`. Preserve its disabled
browser commands and refusal text. That head predates newer upstream theme
fields and section-icon rendering. Refresh this branch against the main of
the merge batch after #1687 merges, preserving those newer fields and wiring
their icon placement to the shared metrics. This branch does not establish
behavior for that newer icon path.

**Cost to change later.** Adjust the four token ranges or the shared helper
to change the menu policy. Additional metrics must join the parser vocabulary,
both built-in columns, the exhaustive token wiring, and the example theme.
No project schema or NC output format changes are needed.
