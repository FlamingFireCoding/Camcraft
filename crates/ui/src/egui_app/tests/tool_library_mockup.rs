//! #3897 (U6 of #3663): the gaps between the tool library window and the
//! #3663 mock-up that U6 closes - the preview's Dimensions and Holder
//! toggles, a small cached drawing and a gauge length in each Document
//! row, the categories on the left of the list, and a window wide enough
//! for the Recipes table and the preview side by side. All of it is view
//! state: no toggle or pick queues a command.
//! Gate map: covers ui::egui_app::tool_library_dialog, ui::state.
//! Provenance: our own structural tests; no HSMWorks material involved.

use camcraft_app::units::{self, Unit};
use egui::accesskit::Role;

use super::tool_library_editor::{H, click_window_button, library_window, open_in_window};
use super::*;
use crate::egui_app::tool_library_dialog::{self as dialog, categories, list, preview, recipes_tab, thumbs};

fn painted_texts(h: &H) -> Vec<String> {
    super::structural::painted(h).into_iter().map(|(t, _)| t).collect()
}

fn face(h: &H) -> crate::state::EditorFace {
    match h.state().state.tool_library.as_ref().and_then(|v| v.editor.as_ref()) {
        Some(e) => e.face,
        None => panic!("no tool is open in the tool library window"),
    }
}

/// The library holder the tests put the seeded tool in.
pub(super) const HOLDER: &str = "BT40 ER32";

/// Put the seeded tool in `HOLDER` (the Apply holder command); its name.
pub(super) fn seeded_in_holder(h: &mut H) -> String {
    let Some(name) = h.state().state.tool_names().into_iter().next() else {
        panic!("the seeded document has a tool");
    };
    let cmd = CommandKind::SetToolHolder { name: name.clone(), entry: HOLDER.into(), gauge: None, projection: None };
    h.state_mut().state.pending.push_back(cmd);
    h.run_steps(2);
    let st = &h.state().state;
    let landed = model::find_by_name(&st.world, &name)
        .and_then(|e| camcraft_app::tool_body::body_of(&st.world, &e))
        .and_then(|b| b.library)
        .map(|r| r.entry);
    assert_eq!(landed.as_deref(), Some(HOLDER), "the holder landed: {}", st.status);
    name
}

/// A shell as wide as a common desktop screen.
pub(super) fn wide_harness() -> H {
    Harness::builder().with_size(egui::Vec2::new(1920.0, 1080.0)).build_ui_state(frame, ShellUi::seeded())
}

#[test]
fn the_preview_toggles_hide_the_dimension_lines_and_the_holder() {
    let mut h = shell_harness();
    h.run_steps(2);
    let name = seeded_in_holder(&mut h);
    open_in_window(&mut h, &name);
    click_window_button(&mut h, "Side");
    let has = |h: &H, start: &str| painted_texts(h).iter().any(|t| t.starts_with(start));
    assert!(face(&h).dims && face(&h).holder, "both toggles start on");
    assert!(has(&h, "dimensions in") && has(&h, "holder Ø"), "the side view dimensions the holder: {:?}", painted_texts(&h));
    let done = h.state().state.history.done_len();
    library_window(&h).get_by_label(preview::HOLDER).click();
    h.run_steps(2);
    assert!(!face(&h).holder, "the Holder toggle is off");
    assert!(has(&h, "dimensions in") && !has(&h, "holder Ø"), "no holder, no holder dimension");
    library_window(&h).get_by_label(preview::DIMS).click();
    h.run_steps(2);
    assert!(!has(&h, "dimensions in") && !has(&h, "stick-out"), "no dimension lines");
    // The 3D view takes the model without the holder.
    click_window_button(&mut h, "3D");
    let m = dialog::model::last().unwrap_or_else(|| panic!("the preview built a model"));
    assert!(m.bare.len() < m.mesh.len(), "the bare model leaves the holder out");
    let st = &h.state().state;
    assert_eq!(st.history.done_len(), done, "a toggle lands no command");
    assert!(st.pending.is_empty(), "and queues none");
}

#[test]
fn a_document_row_has_a_cached_drawing_and_its_gauge_length() {
    let mut h = shell_harness();
    h.run_steps(2);
    let name = seeded_in_holder(&mut h);
    h.get_by_role_and_label(Role::Button, "Tools").click();
    h.run_steps(2);
    h.state_mut().state.close_tool_in_library();
    h.run_steps(3);
    let st = &h.state().state;
    let r = model::find_by_name(&st.world, &name)
        .and_then(|e| camcraft_app::tool_body::body_of(&st.world, &e))
        .and_then(|b| b.library)
        .unwrap_or_else(|| panic!("{name} is in a library holder"));
    let gauge = units::len(r.gauge_len + r.projection, st.units(), 2);
    let win = library_window(&h);
    assert!(win.query_all_by_label(list::GAUGE).next().is_some(), "the list has a Gauge length column");
    assert!(win.query_all_by_label(&gauge).next().is_some(), "the row shows the gauge length {gauge}");
    let b = thumbs::builds();
    h.run_steps(3);
    assert_eq!(thumbs::builds(), b, "a still frame builds no row drawing");
    let mut t = st_tool(&h, &name);
    t.diameter += 1.0;
    h.state_mut().state.pending.push_back(CommandKind::ChangeTool { name: name.clone(), tool: t });
    h.run_steps(2);
    let b1 = thumbs::builds();
    assert_eq!(b1, b + 1, "a changed tool builds its drawing once");
    h.run_steps(3);
    assert_eq!(thumbs::builds(), b1, "and then holds");
}

fn st_tool(h: &H, name: &str) -> camcraft_app::tools::Tool {
    super::tool_library_editor::tool(h, name)
}

#[test]
fn the_categories_filter_the_list() {
    let mut h = shell_harness();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Tools").click();
    h.run_steps(2);
    h.state_mut().state.close_tool_in_library();
    if let Some(v) = h.state_mut().state.tool_library.as_mut() {
        v.source = crate::state::ToolSource::Catalog;
    }
    h.run_steps(3);
    let offered: Vec<(String, camcraft_app::tools::Tool)> = {
        let st = &h.state().state;
        camcraft_app::tools::catalog()
            .into_iter()
            .filter(|(_, t)| crate::machine_modes::tool_kind_offered(&st.world, t.kind))
            .map(|(n, t)| (n.to_owned(), t))
            .collect()
    };
    let pick = |want: crate::state::ToolCategory| offered.iter().find(|(_, t)| categories::category_of(t.kind) == want).map(|(n, _)| n.clone());
    let (Some(drill), Some(mill)) = (pick(crate::state::ToolCategory::Drills), pick(crate::state::ToolCategory::FlatEndMills)) else {
        panic!("the catalog offers a drill and an end mill");
    };
    let row = |h: &H, n: &str| library_window(h).query_all_by_label_contains(n).next().is_some();
    assert!(row(&h, &drill) && row(&h, &mill), "all tools show");
    click_window_button(&mut h, categories::label(crate::state::ToolCategory::Drills));
    assert!(row(&h, &drill) && !row(&h, &mill), "Drills shows the drills only");
    click_window_button(&mut h, categories::ALL);
    assert!(row(&h, &mill), "All shows them again");
    assert!(h.state().state.pending.is_empty(), "a category pick queues no command");
}

#[test]
fn the_window_shows_the_recipes_table_and_the_preview_side_by_side() {
    let mut h = wide_harness();
    h.run_steps(2);
    let Some(name) = h.state().state.tool_names().into_iter().next() else {
        panic!("the seeded document has a tool");
    };
    open_in_window(&mut h, &name);
    click_window_button(&mut h, recipes_tab::TAB);
    let win = library_window(&h).rect();
    assert!(win.width() >= dialog::WIDE_W - 1.0, "the window {win:?} opens {} wide", dialog::WIDE_W);
    let fz = if h.state().state.units() == Unit::Inches { "fz (in)" } else { "fz (mm)" };
    let node = library_window(&h);
    let rect = |l: &str| node.query_all_by_label(l).next().unwrap_or_else(|| panic!("no {l:?} in the window")).rect();
    let (head, view) = (rect(fz), rect("3D"));
    assert!(head.right() <= view.left(), "the table's {fz} column {head:?} stands left of the preview {view:?}");
    assert!(win.expand(1.0).contains_rect(head) && win.expand(1.0).contains_rect(view), "both stay in the window {win:?}");
    // A narrow screen: the window stays left of the Properties pane.
    let narrow = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(1280.0, 720.0));
    let (w, left) = dialog::width(narrow);
    assert!(left >= 8.0 && left + w <= 1280.0 * 0.77, "{w} from {left} keeps left of Properties");
}
