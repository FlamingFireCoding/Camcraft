# 1732 - Slot manual A/B section and viewport marks

- Chose: group the four A/B rows under a "Manual" group on the Geometry tab; Slot width and the #1731 extension rows stay outside it (they are not the hand-typed ends).
- Chose: marks read the page draft (`op_ui.params`), not the committed operation, so they follow typing before Apply; ephemeral, no command. Shown only with no picked selection and the Geometry tab open; drawn at the draft top Z, nothing when it does not resolve (never a guessed height).
- Chose: stacked on #1745 (it edits the same rows in `mill2d_more_page.rs`).
- Dropped: a separate "Manual" tab (a section is what the maintainer asked for); drawing the slot's width outline (not asked).
