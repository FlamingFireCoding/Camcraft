//! #3892 (U1 of #3663): the tool library window. It replaces the old
//! "Tool library" dock tab: a non-modal in-app `egui::Window` that can stay
//! open beside the viewport while Properties shows its own subject. Each
//! door (the ribbon's Tools cell, the View menu, the gate route) opens it
//! through `actions::target` with the empty tool name.
//! Provenance: our own window, after the tool shape window pattern
//! (`pages::tool_shape_editor_window`) and the egui `Window` docs; no
//! HSMWorks material involved.

mod list;

use egui::{Id, Window};

use crate::state::State;

/// The window's title, and its AccessKit name.
pub(crate) const TITLE: &str = "Tool library";

/// Draw the window when it is open (`state.tool_library` is `Some`).
/// Its ✕ closes it; closing is view state, nothing is queued.
pub(crate) fn tool_library_window(ctx: &egui::Context, state: &mut State) {
    if state.tool_library.is_none() {
        return;
    }
    let screen = ctx.viewport_rect();
    let mut open = true;
    Window::new(TITLE)
        .id(Id::new("tool-library"))
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        // Over the viewport, left of the Properties pane, so the two can
        // be used at the same time.
        .default_pos([screen.left() + screen.width() * 0.2, screen.top() + 0.15 * screen.height()])
        .default_size([620.0, (screen.height() * 0.7).max(320.0)])
        .min_width(420.0)
        .show(ctx, |ui| list::draw(ui, state));
    if !open {
        state.close_tool_library();
    }
}
