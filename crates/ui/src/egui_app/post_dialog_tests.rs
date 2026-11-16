//! #258: desktop Post dialog (`CommandKind::PostProgramDialog`) and
//! Setup -> Post grouped declared options UI tests.
//!
//! Provenance: our own page and window law; no vendor material consulted.

use camcraft_app::posting;
use camcraft_app::schema::PostRef;
use camcraft_app::setup_post_options;
use egui::accesskit::Role;
use egui::{Event, Key, Modifiers, Vec2};
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

use crate::state::{CommandKind, State};

fn frame(ui: &mut egui::Ui, st: &mut State) {
    super::panes::shortcuts(ui, st);
    st.drain_actions();
    super::post_dialog::window(ui.ctx(), st);
}

#[test]
fn desktop_post_command_opens_dialog_and_applies_declared_script_options_undoably() {
    let mut state = State::new();
    assert!(!state.browser_build);
    assert!(state.post_dialog.is_none(), "fresh State keeps Post dialog closed for snapshots");

    state.pending.push_back(CommandKind::PostProgramDialog);
    state.drain_actions();
    assert!(state.post_dialog.is_some(), "PostProgramDialog opens in-app Post dialog on desktop");

    let mut h = Harness::builder()
        .with_size(Vec2::new(1100.0, 900.0))
        .build_ui_state(frame, state);
    h.run();

    // Key sections & action buttons are visible inside the Post dialog.
    assert!(
        h.query_by_role_and_label(Role::TextInput, "Program number")
            .is_some(),
        "missing Program number"
    );
    assert!(
        h.query_by_role_and_label(Role::TextInput, "Program comment")
            .is_some(),
        "missing Program comment"
    );
    assert!(
        h.query_by(|n| n.role() == Role::CheckBox && n.label().as_deref() == Some("Write tool list"))
            .is_some(),
        "missing Fanuc writeToolList option"
    );
    assert!(h.query_by_role_and_label(Role::Button, "Post…").is_some());
    assert!(h.query_by_role_and_label(Role::Button, "Apply").is_some());
    assert!(h.query_by_role_and_label(Role::Button, "Cancel").is_some());

    // Switching to Operations tab renders the operations view inside the Post dialog, then return to Settings.
    h.get_by_label("Operations").click();
    h.run();
    assert!(h.query_by_label_contains("No operations").is_some());
    h.get_by_label("Settings").click();
    h.run();

    // Toggle "Write tool list" checkbox and click Apply.
    let cb = h.get_by(|n| n.role() == Role::CheckBox && n.label().as_deref() == Some("Write tool list"));
    let rect = cb.rect();
    cb.click();
    h.run_steps(2);
    assert_eq!(
        h.state()
            .post_dialog
            .as_ref()
            .and_then(|d| d.values.get("writeToolList")),
        Some(&camcraft_app::post::options::OptionValue::Bool(true)),
        "clicking Write tool list at {rect:?} must toggle draft value"
    );
    h.get_by_role_and_label(Role::Button, "Apply").click();
    h.run_steps(2);

    let key = setup_post_options::post_key(&PostRef::Builtin);
    let doc = setup_post_options::script_document(&h.state().world);
    assert!(
        doc.for_post(&key).contains_key("writeToolList"),
        "Apply must commit declared option override via undoable command (status={:?}, feedback={:?}, pending={}, dlg_open={}, dlg_vals={:?}, dlg_err={:?})",
        h.state().status,
        h.state().feedback,
        h.state().pending.len(),
        h.state().post_dialog.is_some(),
        h.state().post_dialog.as_ref().map(|d| d.values.get("writeToolList")),
        h.state().post_dialog.as_ref().and_then(|d| d.error.as_ref()),
    );

    // Undo restores default.
    h.input_mut().events.push(Event::ModifiersChanged(Modifiers::COMMAND));
    h.input_mut().events.push(Event::Key {
        key: Key::Z,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::COMMAND,
    });
    h.run_steps(2);
    let doc_after = setup_post_options::script_document(&h.state().world);
    assert!(
        !doc_after.for_post(&key).contains_key("writeToolList"),
        "Ctrl+Z reverts declared option edit"
    );
    let _ = posting::post_options(&h.state().world);
}

#[test]
fn setup_post_page_also_renders_declared_script_options() {
    let state = State::new();
    let mut h = Harness::builder()
        .with_size(Vec2::new(500.0, 1100.0))
        .build_ui_state(
            |ui, st: &mut State| {
                st.drain_actions();
                super::setup_pages::page(ui, st, "", crate::state::SetupPage::Post);
            },
            state,
        );
    h.run();
    h.get_by_label_contains("Post script options").click();
    h.run_steps(2);
    assert!(
        h.query_by(|n| n.role() == Role::CheckBox && n.label().as_deref() == Some("Write tool list"))
            .is_some(),
        "Setup -> Post must expose declared script options"
    );
}
