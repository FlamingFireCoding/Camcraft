//! The file editor's "Axis chain" group through the real shell (issue
//! #2756): a blank mill gets an A on the table, the A moves up and
//! down the chain, a refused letter changes nothing, and the saved file
//! reads back with the A. None of it touches the command queue.
//! Provenance: our own tests, specced from issue #2756.
//! Structural only: no golden PNG.

use super::machines_edit::{draft_at, pick_kind, press, prompt, type_into, user_dir};
use super::machines_pane::{assert_no_commands, harness_with_user_dir, open_machines};
use super::*;
use crate::egui_app::machine_file_editor::Prompt;

const TABLE: &str = "/kinematics/default/parts/0/parts";

fn name_at(h: &Harness<'static, ShellUi>, slot: usize) -> Option<String> {
    draft_at(h, &format!("{TABLE}/{slot}/name")).and_then(|v| v.as_str().map(str::to_owned))
}

fn names(h: &Harness<'static, ShellUi>) -> Vec<String> {
    (0..5).filter_map(|i| name_at(h, i)).filter(|n| n != "head").collect()
}

fn status(h: &Harness<'static, ShellUi>) -> Option<String> {
    h.state().state.machines_pane.editor.as_ref().and_then(|e| e.status.as_ref().map(|s| s.1.clone()))
}

fn blank_mill(tag: &str) -> (Harness<'static, ShellUi>, std::path::PathBuf) {
    let dir = user_dir(tag);
    let mut h = harness_with_user_dir(Some(dir.clone()));
    h.run();
    open_machines(&mut h);
    pick_kind(&mut h, "All", "mill");
    press(&mut h, "New blank");
    (h, dir)
}

#[test]
fn a_blank_mill_gets_an_a_on_the_table_moves_it_and_saves_it() {
    let (mut h, dir) = blank_mill("chain");
    h.get_by_label("Z - linear, moves the work");
    type_into(&mut h, "New axis", "a");
    press(&mut h, "-X");
    press(&mut h, "Add axis");
    assert_eq!(names(&h), ["X", "Y", "Z", "A"], "A goes on the table, before the head");
    let axis = draft_at(&h, &format!("{TABLE}/3/axis"));
    assert_eq!(axis, Some(serde_json::json!([-1.0, 0.0, 0.0])));
    h.get_by_label("A - rotary, moves the work");
    h.get_by_label("Validation: passes");

    press(&mut h, "Move A up");
    assert_eq!(names(&h), ["X", "Y", "A", "Z"]);
    press(&mut h, "Move A down");
    assert_eq!(names(&h), ["X", "Y", "Z", "A"], "up then down is no change");

    // The travel field of the moved chain edits the right part.
    type_into(&mut h, "Z travel min", "-100");
    assert_eq!(draft_at(&h, &format!("{TABLE}/2/travel/min")).and_then(|v| v.as_f64()), Some(-100.0));
    type_into(&mut h, "Z travel max", "0");
    h.get_by_label("Validation: passes");

    press(&mut h, "Save machine");
    assert_eq!(prompt(&h), Some(Prompt::SaveAs));
    press(&mut h, "Write file");
    let text = std::fs::read_to_string(dir.join("new-mill.mch")).expect("the machine was written");
    let m = camcraft_app::machine::parse_mch(&text).expect("the saved machine reads back");
    assert!(m.axes.iter().any(|a| a.name == "A"), "{text}");
    assert_no_commands(&h);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_refused_letter_adds_nothing_and_a_removed_axis_leaves_the_rest() {
    let (mut h, dir) = blank_mill("chain_refused");
    type_into(&mut h, "New axis", "Q");
    press(&mut h, "Add axis");
    assert_eq!(status(&h).as_deref(), Some("axis Q: a linear axis is named X, Y, Z, U, V or W"));
    assert_eq!(names(&h), ["X", "Y", "Z"], "nothing added");

    type_into(&mut h, "New axis", "X");
    press(&mut h, "Add axis");
    assert_eq!(status(&h).as_deref(), Some("axis X: the machine already has an axis X"));

    press(&mut h, "Remove Y");
    assert_eq!(names(&h), ["X", "Z"]);
    assert!(h.query_by_label("Y - linear, moves the work").is_none());
    assert_no_commands(&h);
    let _ = std::fs::remove_dir_all(&dir);
}
