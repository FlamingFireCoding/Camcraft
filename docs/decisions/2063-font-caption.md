## #2063 - Read `Tokens::font_caption` in `painted::caption` and align built-in defaults

Issue: #2063. Stacked on PR #2027 (#2021).

### Chosen

- **Read and clamp `pal.font_caption` in `painted::caption`**: `crates/ui/src/egui_app/painted.rs` now lays out ribbon group captions with `clamp::range(pal.font_caption, clamp::FONT)` instead of `MIN_LABEL_PX` (`10.0`). This matches the existing draw-site clamp law (`6.0..=32.0` px; non-finite values fall back to `6.0`) so custom theme files change caption size without overflowing or breaking layout on corrupt values.
- **Set built-in `Tokens::DARK.font_caption` and `Tokens::LIGHT.font_caption` to `10.0`**: `crates/ui/src/egui_app/theme/tables.rs` restores the pre-`33cd8b56` caption size (`10.0` px) instead of the orphaned `12.5` px left behind when `painted::caption` was removed and restored. At `10.0` px (`line_h(10.0) + 4.0 = 17.0 <= 18.0`), group captions keep at least 2 px clearance above and below inside `ribbon_caption_row_h: 18.0` and stay smaller than button labels (`font_label: 11.0`).
- **Track `t.font_caption` in `ribbon_rows` height checks**: `tab_text_and_caption_text_fit_their_rows_in_each_theme` checks `line_h(t.font_caption) + 4.0` instead of literal `line_h(10.0) + 4.0`, and `builtin_caption_font_fits_caption_row` pins `Tokens::DARK` and `Tokens::LIGHT`.

### Considered and dropped

- **Keeping `font_caption: 12.5` and growing `ribbon_caption_row_h` in `DARK`/`LIGHT`**: rejected because `12.5` px captions exceed `font_label` (`11.0` px), inverting the ribbon hierarchy, and would force increasing `ribbon_caption_row_h` (to at least `21.0` px) and shipped `horizon-day.json` / `teal-night.json` ribbon heights away from PR #2027's `18.0` / `88.0` px design.
- **Using raw `pal.font_caption` without `clamp::range(..., clamp::FONT)`**: rejected because `theme.rs` documents that every draw site enforces `clamp::FONT` against arbitrary JSON inputs.

### Cost to change later

- Trivial: a two-line edit in `crates/ui/src/egui_app/theme/tables.rs` (plus updating `ribbon_caption_row_h` if `font_caption` exceeds `10.7` px at `18.0` px row height). Open PR #2038 (`assets/themes/theme.json.example`, currently `"font_caption": 12.5`) should align to `10.0` after merging.
