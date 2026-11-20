//! #2397: the tool shape editor window and the new-tool flow in Properties.
//! The saved tool becomes the selection when Add tool lands, so the page
//! shows its normal controls - the Edit tool shape button - without a
//! second browser selection.
//! Provenance: our own structural tests over the shipping tool page.
//! Gate map: covers ui::egui_app::tool_settings, ui::actions.

use super::*;
use egui_kittest::kittest::NodeT;

/// Drive the new-tool door on the Tool library page: New tool… lands the
/// draft in Properties. No browser selection is used after the add.
fn begin_new_tool_from_the_library_door(h: &mut Harness<'static, ShellUi>) {
    h.get_by_label("View").click();
    h.run_steps(2);
    h.get_by_label("Tool library…").click();
    h.run_steps(3);
    h.get_by_label("New tool…").click();
    h.run_steps(3);
    assert!(h.state().state.new_tool_form, "the door opens the blank draft");
}

/// #2397 review: after New tool… → Add tool, the page must leave the
/// "New tool" form and show the saved tool's normal controls, including
/// Edit tool shape, without another browser selection.
#[test]
fn add_tool_selects_the_saved_tool_and_opens_its_shape() {
    let mut h = shell_harness();
    begin_new_tool_from_the_library_door(&mut h);
    let name = h.state().state.tool_ui.name.clone();
    assert!(!name.is_empty(), "the draft is numbered already");
    h.get_by_label("Add tool").click_accesskit();
    h.run_steps(3);

    // The saved tool is the target, and the form is over: no second
    // selection was made.
    assert_eq!(
        h.state().state.ui_target,
        UiTarget::Tool(name.clone()),
        "the Add tool click retargets the saved tool"
    );
    assert!(!h.state().state.new_tool_form, "the new-tool form ends with the add");
    assert!(
        model::find_by_name(&h.state().state.world, &name).is_some(),
        "the tool landed in the document"
    );

    // The normal controls show: the named heading, and the shape door.
    assert!(
        h.query_all_by_label(&format!("Tool: {name}")).next().is_some(),
        "the page shows the saved tool's heading"
    );
    h.get_by_label("Edit tool shape").click_accesskit();
    h.run_steps(3);
    assert_eq!(h.state().state.tool_shape_editor.as_deref(), Some(name.as_str()));
    let titled = format!("Tool shape: {name}");
    assert!(
        h.query_all_by_role(egui::accesskit::Role::Window)
            .any(|n| n.accesskit_node().label().as_deref() == Some(titled.as_str())),
        "the shape window opens on the tool added without a browser selection"
    );
}
