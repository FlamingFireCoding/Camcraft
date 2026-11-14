//! The tool editor's modular piece-assembly section (#361 / #1040):
//! lists pieces already on the tool, lets the user attach an adapter or
//! clamping unit (auto-mated by ISO 13399 CSYS or naming target/piece CSYS),
//! and removes individual pieces as one undoable command.
//! Provenance: our own pane section over `app::tool_body_assembly`; no vendor material consulted.

use egui::{TextEdit, Ui};

use camcraft_app::tool_body_assembly;

use crate::state::State;

/// Show modular piece assembly controls below a landed non-spun tool body.
pub(super) fn section(ui: &mut Ui, state: &mut State) {
    let Some(body) = state.editor_tool_body() else {
        return;
    };
    if body.spun.is_some() {
        return;
    }
    let pieces = tool_body_assembly::pieces_of(&body);
    ui.separator();
    ui.horizontal(|ui| {
        ui.strong(format!("Assembly ({} piece(s))", pieces.len()));
        let attach = crate::browser_files::button(
            ui,
            state.in_browser(crate::browser_files::NEEDS_PICKER),
            "Attach piece…",
            "mate another STEP/STL/OBJ adapter or clamping unit onto this tool body (one undo step)",
        );
        if attach {
            state.attach_tool_body_piece_dialog();
        }
    });
    ui.horizontal(|ui| {
        ui.label("Target CSYS").on_hover_text("blank = auto-mate MCS <-> CSW");
        ui.add(
            TextEdit::singleline(&mut state.tool_body_ui.assembly.target_csys)
                .desired_width(64.0)
                .hint_text("auto"),
        );
        ui.label("Piece CSYS").on_hover_text("blank = auto-mate MCS <-> CSW");
        ui.add(
            TextEdit::singleline(&mut state.tool_body_ui.assembly.piece_csys)
                .desired_width(64.0)
                .hint_text("auto"),
        );
    });
    for (i, piece) in pieces.iter().enumerate() {
        ui.push_id(i, |ui| {
            ui.horizontal(|ui| {
                let csys = if piece.csys.is_empty() {
                    String::new()
                } else {
                    let names: Vec<&str> = piece.csys.iter().map(|c| c.name.as_str()).collect();
                    format!(" [{}]", names.join(", "))
                };
                ui.label(format!("{i}: {}{csys}", piece.source));
                let rm = ui
                    .add_enabled(pieces.len() > 1, egui::Button::new("Remove"))
                    .on_hover_text("remove this piece from the assembly (undoable)");
                if rm.clicked() {
                    state.queue_remove_tool_body_piece(i);
                }
            });
        });
    }
}
