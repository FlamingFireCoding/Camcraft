//! #3892 (U1 of #3663): the named layout checks of the tool library
//! window. `page_layout.rs` is full, so the window's tests live here
//! (#1953: one named layout test for each page and window).
//! Gate map: covers ui::egui_app::tool_library_dialog, ui::actions, ui::state.
//! Provenance: our own layout law over the egui_kittest AccessKit tree; no
//! HSMWorks material involved.

use super::page_layout_windows::assert_window_layout;
use super::*;
use egui_kittest::kittest::NodeT;

type H = Harness<'static, ShellUi>;

fn library_window(h: &H) -> egui::Rect {
    h.query_all_by_role(egui::accesskit::Role::Window)
        .find(|n| n.accesskit_node().label().as_deref() == Some("Tool library"))
        .map(|n| n.rect())
        .unwrap_or_else(|| panic!("the Tool library window is not open"))
}

/// The tool library is its own window: the file buttons first, then the
/// source row, then the Document list. It floats beside the dock and
/// leaves the Properties pane free, so both can be used at once.
#[test]
fn layout_window_tool_library() {
    let _g = snapshot_guard();
    let mut h = shell_harness();
    h.run_steps(2);
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Tools").click();
    h.run_steps(3);
    assert_window_layout(
        &h,
        "Tool library",
        &["New tool…", "Import tool holders", "Document", "Recipes"],
    );
    let win = library_window(&h);
    let Some(props) = super::layout::leaf_rect(&h.state().dock, &Tab::Properties) else {
        panic!("the Properties leaf is not laid out");
    };
    assert!(
        !win.intersects(props),
        "the window {win:?} stands beside the Properties pane {props:?}, not over it"
    );
}
