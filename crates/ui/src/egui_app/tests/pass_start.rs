//! #3316: 2D Offset's Passes tab carries the shared "Pass starts at"
//! row (Auto / Inside / Outside, Auto by default), and 2D Pocket too
//! (#3354); 2D Adaptive, which
//! shares the pane, does not have it yet (its own sub-issue). Provenance:
//! our own page law; asserted structurally (labels, the combo's value;
//! no pixels).

use super::*;
use camcraft_app::machining2d::PassStart;
use camcraft_app::operation::{KindInfo, OpType};
use egui::accesskit::Role;

use super::super::pass_start_row::{FACE_NO_PASS_ORDER, PASS_START_LABEL, pass_start_names};

fn kind(ty: OpType) -> usize {
    KindInfo::ALL.iter().position(|k| k.ty == ty).unwrap_or(usize::MAX)
}

/// The Passes tab of the draft retyped to `ty`, drawn.
fn passes_of(ty: OpType) -> Harness<'static, ShellUi> {
    let mut harness = shell_harness();
    let cur = harness.state().state.op_ui.params.clone();
    harness.state_mut().state.op_ui.params = crate::operations::retype(&cur, kind(ty));
    let passes = super::super::op_tabs::SECTIONS.iter().position(|s| *s == "Passes");
    harness.state_mut().state.op_tab = passes.unwrap_or(0);
    harness.run();
    harness
}

fn combos_showing(h: &Harness<'static, ShellUi>, text: &str) -> usize {
    h.query_all_by(|n| n.role() == Role::ComboBox && n.value().as_deref() == Some(text)).count()
}

#[test]
fn the_offset_passes_tab_offers_pass_start_and_shows_the_draft() {
    assert_eq!(pass_start_names(), ["Auto", "Inside", "Outside"], "one name per value, in the combo's order");
    let mut harness = passes_of(OpType::Offset2D);
    assert_eq!(harness.query_all_by_label(PASS_START_LABEL).count(), 1, "the Pass starts at row");
    let outside_before = combos_showing(&harness, "Outside");
    if let OpParams::Offset2D { pass_start, .. } = &mut harness.state_mut().state.op_ui.params {
        assert_eq!(*pass_start, PassStart::Auto, "a new 2D Offset starts Auto");
        *pass_start = PassStart::Outside;
    } else {
        panic!("the draft is not a 2D Offset");
    }
    harness.run();
    assert_eq!(combos_showing(&harness, "Outside"), outside_before + 1, "the combo shows the draft's value");
}

/// #3354: 2D Pocket's Passes tab carries the same row, Auto by default.
#[test]
fn the_pocket_passes_tab_offers_pass_start_and_shows_the_draft() {
    let mut harness = passes_of(OpType::Pocket);
    assert_eq!(harness.query_all_by_label(PASS_START_LABEL).count(), 1, "the Pass starts at row");
    let inside_before = combos_showing(&harness, "Inside");
    if let OpParams::Pocket { pass_start, .. } = &mut harness.state_mut().state.op_ui.params {
        assert_eq!(*pass_start, PassStart::Auto, "a new 2D Pocket starts Auto");
        *pass_start = PassStart::Inside;
    } else {
        panic!("the draft is not a 2D Pocket");
    }
    harness.run();
    assert_eq!(combos_showing(&harness, "Inside"), inside_before + 1, "the combo shows the draft's value");
}

/// #3356: Face has no pass order (its rows run edge to edge). Its row is
/// there, disabled, and says why by name.
#[test]
fn the_face_passes_tab_says_it_has_no_pass_order() {
    let harness = passes_of(OpType::Face);
    let rows: Vec<_> = harness.query_all_by_label(PASS_START_LABEL).collect();
    assert_eq!(rows.len(), 1, "the Pass starts at row on Face");
    let locked = harness
        .query_all_by(|n| n.role() == Role::ComboBox && n.value().as_deref() == Some("Auto") && n.is_disabled())
        .count();
    assert_eq!(locked, 1, "the combo shows Auto and Face cannot pick an order");
    assert_eq!(
        harness
            .query_all_by(|n| [n.label(), n.value()].into_iter().flatten().any(|l| l.contains(FACE_NO_PASS_ORDER)))
            .count(),
        1,
        "the row names why Face has no order, in view (not only on hover)"
    );
}

#[test]
fn the_adaptive_twin_has_no_pass_start_row_yet() {
    let harness = passes_of(OpType::Adaptive2D);
    assert_eq!(harness.query_all_by_label(PASS_START_LABEL).count(), 0, "2D Adaptive gets the row in its own issue");
}

/// #3316: a pass-start warning on the toolpath is a warning line of the
/// op page's status, word for word.
#[test]
fn a_pass_start_warning_is_a_status_line() {
    let side = camcraft_app::machining2d::StartSide::from(PassStart::Outside);
    let note = side.wall_first_note().unwrap_or_default();
    let lines = super::super::approach_page::entry_status_lines(&[note.clone(), "a quiet note".to_owned()]);
    assert_eq!(lines, vec![format!("warning: {note}")]);
    assert!(note.starts_with(camcraft_app::machining2d::PASS_START_NOTE), "{note}");
}
