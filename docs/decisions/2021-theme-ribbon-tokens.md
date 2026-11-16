# 2021: ribbon rows and glyph ink are theme tokens

Issue: #2021. Replaces the closed PR #1999 (hard-coded values).

## Chosen

- `Tokens` gets six fields: `ribbon_ink` (colour) and five geometry values:
  `ribbon_tab_h`, `ribbon_cell_top_gap`, `ribbon_caption_gap`,
  `ribbon_caption_row_h`, `ribbon_bottom_gap`. Theme files set them by
  these names (`colors.ribbon_ink`, `geometry.ribbon_*`).
- `theme::ribbon_h` is the sum of the rows, the gaps and `cell_h` of the
  active table. The shell has no row number and no ink literal.
- Built-in tables: 24 / 4 / 8 / 18 / 4 (ribbon 88 px). `DARK` ink is white.
  `LIGHT` ink is `#1F242A`.
- `horizon-day.json`: ink `#1F242A`. The ribbon ground is near white, so white
  glyphs would vanish. This is the one place where the maintainer's "white"
  does not apply.
- `teal-night.json`: ink white.
- Both theme files set `glyph_px` 22, so the label under the glyph ends 10.3 px
  below the cell box. Both files use `ribbon_caption_gap` 12 (the table says
  "or larger"), so the caption does not touch the label.
- `contrast::readable` refuses a theme whose `ribbon_ink` reads under 4.5:1 on
  the ribbon ground. A file that spells an unreadable ink is a visible refusal.
- Family hues (`family_tint`) are removed. A filled cell keeps the on-fill ink.

## Dropped

- A fixed `glyph_ink` function (PR #1999): the rule moves into the tables.
- Clamp windows: rows 12 to 48 px, gaps 0 to 24 px, so a file cannot make
  the ribbon impossible.
