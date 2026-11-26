//! The viewport overlay inks are theme tokens (#1120): each built-in
//! column keeps the colour the draw site wrote as a literal before, so
//! the move changes no pixel, and a theme file can set each ink by name.
//! Provenance: our own tests over our own token table and theme format.
//!
//! A module of the `theme_files` suite (#3187: no new test binary).

use camcraft_ui::egui_app::theme::Tokens;
use camcraft_ui::egui_app::theme_files as tf;
use egui::Color32;

/// The handle and pick inks of a token table.
fn inks(t: &Tokens) -> [Color32; 3] {
    [t.handle_shade, t.handle_text, t.pick_rim]
}

#[test]
fn the_built_in_columns_keep_the_handle_and_pick_colours() {
    // The literals of `move_handles.rs`, `turn_handles.rs` and
    // `align_pick_viz.rs` before #1120, in both columns: the handles and
    // the picks draw over the 3D view, whose ink does not follow the
    // chrome's column.
    let before = [
        Color32::from_black_alpha(170),
        Color32::WHITE,
        Color32::from_rgb(245, 240, 250),
    ];
    assert_eq!(inks(&Tokens::DARK), before, "dark column");
    assert_eq!(inks(&Tokens::LIGHT), before, "light column");
}

#[test]
fn a_theme_file_sets_each_handle_and_pick_ink_by_name() {
    let text = r##"{
        "format": "camcraft-theme/1",
        "descriptor": { "name": "inks" },
        "base": "dark",
        "colors": { "handle_shade": "#10203040", "handle_text": "#FFEE00", "pick_rim": "#00FF88" }
    }"##;
    let theme =
        tf::parse_text(text, "inks.json").unwrap_or_else(|e| panic!("the theme reads: {e}"));
    assert_eq!(
        inks(&theme.tokens),
        [
            Color32::from_rgba_unmultiplied(0x10, 0x20, 0x30, 0x40),
            Color32::from_rgb(0xFF, 0xEE, 0x00),
            Color32::from_rgb(0x00, 0xFF, 0x88),
        ]
    );
    assert!(
        theme.notes.is_empty(),
        "each key is known: {:?}",
        theme.notes
    );
    for name in ["handle_shade", "handle_text", "pick_rim"] {
        assert!(
            tf::COLOR_NAMES.contains(&name),
            "{name} is a colour key of the theme format"
        );
    }
}
