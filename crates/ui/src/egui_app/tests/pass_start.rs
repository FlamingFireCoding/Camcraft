//! #3316 / #3354 / #3355: the Passes tabs of 2D Offset, 2D Pocket and
//! 2D Adaptive carry the shared "Pass starts at" row (Auto / Inside /
//! Outside, Auto by default). Provenance:
//! our own page law; asserted structurally (labels, the combo's value;
//! no pixels).

use super::*;
use camcraft_app::machining2d::PassStart;
use camcraft_app::operation::{KindInfo, OpType};
use egui::accesskit::Role;

use super::super::pass_start_row::{
    FACE_NO_PASS_ORDER, PASS_START_LABEL, TROCH_PASS_START_REFUSED, pass_start_names,
};

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

/// #3360: Bore's Passes tab has the same row, Auto by default, and the
/// combo shows the draft's value.
#[test]
fn the_bore_passes_tab_offers_pass_start_and_shows_the_draft() {
    let mut harness = passes_of(OpType::Bore);
    assert_eq!(harness.query_all_by_label(PASS_START_LABEL).count(), 1, "the Pass starts at row");
    let outside_before = combos_showing(&harness, "Outside");
    if let OpParams::Bore { pass_start, .. } = &mut harness.state_mut().state.op_ui.params {
        assert_eq!(*pass_start, PassStart::Auto, "a new Bore starts Auto");
        *pass_start = PassStart::Outside;
    } else {
        panic!("the draft is not a Bore");
    }
    harness.run();
    assert_eq!(combos_showing(&harness, "Outside"), outside_before + 1, "the combo shows the draft's value");
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
fn the_slot_passes_tab_locks_auto_and_names_the_continuous_pass() {
    let harness = passes_of(OpType::Slot);
    assert_eq!(harness.query_all_by_label(PASS_START_LABEL).count(), 1, "the Pass starts at row");
    let locked = harness
        .query_all_by(|n| n.role() == Role::ComboBox && n.value().as_deref() == Some("Auto") && n.is_disabled())
        .count();
    assert_eq!(locked, 1, "Slot shows Auto and cannot pick an order");
    let reason = "Slot has no pass order. Each depth has one continuous pass. A tool-width cut follows the centerline. A wider cut connects rows.";
    assert_eq!(
        harness
            .query_all_by(|n| [n.label(), n.value()].into_iter().flatten().any(|text| text.contains(reason)))
            .count(),
        1,
        "the reason is visible without hovering"
    );
}

#[test]
fn the_trochoidal_slot_passes_tab_locks_auto_and_explains_the_loop_order() {
    let harness = passes_of(OpType::TrochoidalSlot);
    assert_eq!(harness.query_all_by_label(PASS_START_LABEL).count(), 1, "the Pass starts at row");
    let locked = harness
        .query_all_by(|n| {
            n.role() == Role::ComboBox && n.value().as_deref() == Some("Auto") && n.is_disabled()
        })
        .count();
    assert_eq!(locked, 1, "Trochoidal Slot shows Auto and cannot pick an order");
    let reason = harness
        .query_all_by(|n| {
            [n.label(), n.value()].into_iter().flatten().any(|text| {
                text.contains("entry bore")
                    && text.contains("advances along the slot")
                    && text.contains("clearance from the previous cut")
            })
        })
        .count();
    assert_eq!(reason, 1, "the reason is visible without hovering");
}

/// #3355: 2D Adaptive has the same row; under the Trochoidal method a
/// forced order is said refused by name, never dropped in silence.
#[test]
fn the_adaptive_twin_offers_pass_start_and_says_the_trochoidal_refusal() {
    let mut harness = passes_of(OpType::Adaptive2D);
    assert_eq!(harness.query_all_by_label(PASS_START_LABEL).count(), 1, "the Pass starts at row");
    let inside_before = combos_showing(&harness, "Inside");
    if let OpParams::Adaptive2D { pass_start, .. } = &mut harness.state_mut().state.op_ui.params {
        assert_eq!(*pass_start, PassStart::Auto, "a new 2D Adaptive starts Auto");
        *pass_start = PassStart::Inside;
    } else {
        panic!("the draft is not a 2D Adaptive");
    }
    harness.run();
    assert_eq!(combos_showing(&harness, "Inside"), inside_before + 1, "the combo shows the draft's value");
    assert_eq!(harness.query_all_by_label(TROCH_PASS_START_REFUSED).count(), 0, "Spiral takes the order");
    if let OpParams::Adaptive2D { method, .. } = &mut harness.state_mut().state.op_ui.params {
        *method = camcraft_app::machining2d::AdaptiveMethod::Trochoidal;
    }
    harness.run();
    assert_eq!(harness.query_all_by_label(TROCH_PASS_START_REFUSED).count(), 1, "Trochoidal says the refusal");
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

#[test]
fn circular_passes_tab_offers_all_three_pass_starts() {
    let mut harness = passes_of(OpType::Circular);
    assert_eq!(harness.query_all_by_label(PASS_START_LABEL).count(), 1, "the Circular Passes tab has the shared row");
    let mut current = "Auto";
    for (label, choice) in [("Inside", Some("inside")), ("Outside", Some("outside")), ("Auto", None)] {
        harness.get_by(|node| node.role() == Role::ComboBox && node.value().as_deref() == Some(current)).click();
        harness.run_steps(2);
        harness.get_by_label(label).click();
        harness.run_steps(2);
        let wire = camcraft_app::project::params_to_json(&harness.state().state.op_ui.params);
        assert_eq!(wire["circular"].get("pass_start").and_then(serde_json::Value::as_str), choice);
        assert_eq!(combos_showing(&harness, label), 1, "the combo displays the new choice");
        current = label;
    }
}

/// #3361: 3D Pocket's Passes tab carries the shared row, Auto by default.
/// Under the default strategy (Auto, which cuts rows) and under Adaptive
/// the line under it says, in view, that the value changes no move; the
/// ring strategies take it without that line. The wire writes no key for
/// Auto and the word for Inside / Outside.
#[test]
fn the_pocket3d_passes_tab_offers_pass_start_and_names_the_row_strategies() {
    use super::super::pass_start_row::POCKET3D_NO_RING_ORDER;
    use camcraft_app::machining2d::PocketStrategy;
    let shown = |h: &Harness<'static, ShellUi>| {
        h.query_all_by(|n| [n.label(), n.value()].into_iter().flatten().any(|l| l.contains(POCKET3D_NO_RING_ORDER)))
            .count()
    };
    let mut harness = passes_of(OpType::Pocket3D);
    assert_eq!(harness.query_all_by_label(PASS_START_LABEL).count(), 1, "the Pass starts at row");
    assert_eq!(shown(&harness), 1, "the default strategy cuts rows: the line says the value is not used");
    let wire = camcraft_app::project::params_to_json(&harness.state().state.op_ui.params);
    assert_eq!(wire["pocket3d"].get("pass_start"), None, "Auto writes no key");
    let outside_before = combos_showing(&harness, "Outside");
    if let OpParams::Pocket3D { pass_start, strategy, .. } = &mut harness.state_mut().state.op_ui.params {
        assert_eq!(*pass_start, PassStart::Auto, "a new 3D Pocket starts Auto");
        *pass_start = PassStart::Outside;
        *strategy = PocketStrategy::Waterline;
    } else {
        panic!("the draft is not a 3D Pocket");
    }
    harness.run();
    assert_eq!(combos_showing(&harness, "Outside"), outside_before + 1, "the combo shows the draft's value");
    assert_eq!(shown(&harness), 0, "Waterline uses the value: no line");
    let wire = camcraft_app::project::params_to_json(&harness.state().state.op_ui.params);
    assert_eq!(wire["pocket3d"].get("pass_start").and_then(serde_json::Value::as_str), Some("outside"));
    if let OpParams::Pocket3D { strategy, .. } = &mut harness.state_mut().state.op_ui.params {
        *strategy = PocketStrategy::Adaptive;
    }
    harness.run();
    assert_eq!(shown(&harness), 1, "the Adaptive spiral has no ring order: the line is back");
}
