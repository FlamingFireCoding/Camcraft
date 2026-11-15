## #1990 - Remove size, colour, and fixed-point checks from UI behaviour tests

**Decided (unattended, Antigravity (len)):**

1. **Tag the Properties pane and G-code cursor row via AccessKit instead of adding custom test getters.**
   `pages::properties` wraps its body in `ui.scope` with `Role::Pane` and label `"Properties"`, and `gcode_view::gcode_listing` calls `set_selected(at_cursor)` on each code label. `pane_texts` queries the `"Properties"` pane subtree instead of filtering widgets by `rect().left()`, and `gcode_viewer` checks `is_selected() == Some(true)` instead of matching `selection.bg_fill` colours. Tagging `pages::properties` directly avoids touching `shell.rs` (where open PR `#2029` changes dock layout) and leaves `properties_body` untouched to minimize merge conflicts with open pages PRs.
   *Considered and dropped:* Exposing shell state getters for filtered properties or selected G-code text, which would test internal state rather than rendered UI output.

2. **Expose icon texture names through `.alt_text(tex.name())` in `tool_icons::glyph`.**
   Each bundled texture already carries an `"icon-..."` name. Setting `.alt_text(tex.name())` lets `tool_icon_wiring::glyph_in_row` verify that the row container holds the matching `Role::Image` node alongside checking `painted_textures`, removing the Y-center geometry check without modifying `icons.rs`.
   *Considered and dropped:* Keeping Y-coordinate row matching or parsing icon pixel data.

3. **Move layout checks to `page_layout.rs` under a non-colliding test name and keep `#1996` as the stack base.**
   The size, alignment, and non-wrapping assertions from `fixtures_page::the_fixtures_page_offers_mark_for_exactly_one_selected_model` and `gcode_viewer` (`a_line_wider_than_the_viewer_stays_on_one_row` and `ticking_trace_keeps_the_window_its_size`) move into `layout_page_setup_fixtures_mark` and `layout_window_gcode_wrap_and_trace` in `crates/ui/src/egui_app/tests/page_layout.rs`, keeping `camcraft-ui` `#[test]` count above baseline (`1004 >= 1000`). The second test uses `layout_window_gcode_wrap_and_trace` rather than `layout_window_gcode` because open `merge-ready` PR `#2068` already defines `layout_window_gcode` on top of `#1996`. Fixed `pos2(1.0, 1.0)` pointer parks in `edge_click_modifiers.rs` and `geo_row_hover.rs` move to `anchors::chrome_point`.
   *Considered and dropped:* Stacking on PR `#2068` (which itself stacks on `#1996`, breaking the one-level stack rule) or deleting the layout checks without moving them to `page_layout.rs`.

**Cost to change later:** Low. The three UI AccessKit additions are small localized builders (`gcode_view.rs`, `tool_icons.rs`, `pages.rs`), and the moved layout tests can merge with `#2068`'s `layout_window_gcode` once `#1996` and `#2068` land.
