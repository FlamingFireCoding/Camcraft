//! #361 / #1040: the tool editor's modular piece-assembly section -
//! attaching pieces onto a landed spindle or lathe body (auto-mated or
//! naming target/piece CSYS), removing a piece via the Remove button in
//! one undoable step, and refusing missing/ambiguous CSYS in the status.
//! Provenance: our own page test over our own modular fixtures; no vendor
//! file consulted.

use super::*;
use camcraft_app::lathe_holder::LathePlacement;
use camcraft_app::tool_body;

const INSERT: &str = "TN-1600-0R8";
const MILL: &str = "EM-0600-4FL";

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/models").join(name)
}

fn editor_on(tool: &str) -> Harness<'static, ShellUi> {
    let mut harness = shell_harness();
    harness.run();
    rack(&mut harness.state_mut().state, &[tool]);
    harness.run();
    crate::actions::open_tool_editor(&mut harness.state_mut().state, tool);
    harness.run();
    harness.run();
    harness
}

fn pieces_len(state: &State, tool: &str) -> usize {
    model::find_by_name(&state.world, tool)
        .and_then(|e| tool_body::body_of(&state.world, &e))
        .map_or(0, |b| b.pieces.len())
}

#[test]
fn editor_attaches_and_removes_modular_lathe_pieces_in_one_undo_step() {
    let mut harness = editor_on(INSERT);
    let head = fixture("modular_lathe_head.step");
    let clamp = fixture("modular_clamp_unit.step");
    {
        let st = &mut harness.state_mut().state;
        st.import_tool_body_from(&head);
        st.tool_body_ui.lathe.as_mut().expect("lathe draft").place = LathePlacement {
            radial: [1.0, 0.0, 0.0],
            along: [0.0, 0.0, 1.0],
            tip: [0.0, 0.0, 0.0],
        };
    }
    harness.run();
    harness.get_by_label("Apply body").click();
    harness.run();
    harness.run();
    assert_eq!(pieces_len(&harness.state().state, INSERT), 1);
    assert!(harness.query_by_label("Assembly (1 piece(s))").is_some());
    assert!(harness.query_by_label("Attach piece…").is_some());

    // Attach clamping unit via unambiguous auto-mate (MCS <-> CSW).
    harness.state_mut().state.attach_tool_body_piece_from(&clamp);
    harness.run();
    harness.run();
    assert_eq!(pieces_len(&harness.state().state, INSERT), 2);
    assert!(harness.query_by_label("Assembly (2 piece(s))").is_some());
    assert!(harness.query_by_label_contains("0: modular_lathe_head.step").is_some());
    assert!(harness.query_by_label_contains("1: modular_clamp_unit.step").is_some());

    // Remove piece 1 via the UI Remove button, then undo to restore it.
    let mut remove_btns: Vec<_> = harness.get_all_by_label("Remove").collect();
    assert_eq!(remove_btns.len(), 2);
    remove_btns.swap_remove(1).click();
    harness.run();
    harness.run();
    assert_eq!(pieces_len(&harness.state().state, INSERT), 1);

    harness.state_mut().state.pending.push_back(CommandKind::Undo);
    harness.run();
    harness.run();
    assert_eq!(pieces_len(&harness.state().state, INSERT), 2);
}

#[test]
fn editor_mates_spindle_pieces_with_explicit_csys_and_refuses_bad_names() {
    let mut harness = editor_on(MILL);
    let head = fixture("tool_csys_box_iso13399.step");
    let unit = fixture("modular_clamp_unit.step");
    harness.state_mut().state.import_tool_body_from(&head);
    harness.run();
    harness.get_by_label("Apply body").click();
    harness.run();
    harness.run();
    assert_eq!(pieces_len(&harness.state().state, MILL), 1);

    // Explicit CSYS mate attaches piece 1 cleanly.
    {
        let st = &mut harness.state_mut().state;
        st.tool_body_ui.assembly.target_csys = "MCS".into();
        st.tool_body_ui.assembly.piece_csys = "CSW".into();
        st.attach_tool_body_piece_from(&unit);
    }
    harness.run();
    harness.run();
    assert_eq!(pieces_len(&harness.state().state, MILL), 2);

    // Unknown CSYS name refuses by name without changing the assembly.
    {
        let st = &mut harness.state_mut().state;
        st.tool_body_ui.assembly.target_csys = "NO_SUCH".into();
        st.tool_body_ui.assembly.piece_csys = "CSW".into();
        st.attach_tool_body_piece_from(&unit);
    }
    harness.run();
    harness.run();
    let st = &harness.state().state;
    assert_eq!(pieces_len(st, MILL), 2);
    assert!(st.status.contains("piece not attached") && st.status.contains("NO_SUCH"), "{}", st.status);
}
