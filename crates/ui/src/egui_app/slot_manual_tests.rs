//! #1732: the Slot page's "Manual" section and the A/B marks in the
//! viewport. Structural (AccessKit labels and placement) plus the mark
//! geometry read straight off the page draft; never pixels.
//! Provenance: our own page law; no vendor material consulted.

use camcraft_app::geo2d::Point2;
use camcraft_app::model;
use camcraft_app::operation::OpParams;
use egui_kittest::kittest::Queryable;

use super::dialog_tests::page;
use super::slot_pick_page_tests::opened;
use crate::slot_manual_viz::{SlotMarkKind, draft_points, slot_marks};
use crate::state::State;

fn draft_a(state: &mut State, x: f64, y: f64) {
    if let OpParams::Slot { a, .. } = &mut state.op_ui.params {
        *a = Point2::new(x, y);
    }
}

fn committed_a(state: &State) -> Point2 {
    match model::find_by_name(&state.world, "Op").and_then(|e| model::operation_of(&state.world, &e)).map(|o| o.params) {
        Some(OpParams::Slot { a, .. }) => a,
        other => panic!("a slot op expected, got {other:?}"),
    }
}

#[test]
fn the_hand_typed_ends_sit_in_a_manual_section() {
    let mut harness = page(opened());
    harness.run();
    let top = |l: &str| harness.get_by_label(l).rect().min.y;
    let manual = top("Manual");
    for f in ["A X", "A Y", "B X", "B Y"] {
        assert!(top(f) > manual, "{f} stands under the Manual header");
    }
    assert!(top("Selection") < manual, "Manual is its own section after Selection");
    assert!(top("Slot width") > top("B Y"), "the width stays outside the point rows");
}

#[test]
fn the_marks_follow_a_draft_edit_before_it_is_committed() {
    let mut state = opened();
    let before = slot_marks(&state, 1.5);
    assert!(before.iter().any(|m| m.kind == SlotMarkKind::Line), "the A-B line draws");
    assert!(before.iter().any(|m| m.kind == SlotMarkKind::PointA));
    assert!(before.iter().any(|m| m.kind == SlotMarkKind::PointB));
    let (pa, _) = draft_points(&state).expect("points");
    assert_eq!((pa[0], pa[1]), (-20.0, 0.0));
    draft_a(&mut state, -10.0, 5.0);
    let (pa, _) = draft_points(&state).expect("points");
    assert_eq!((pa[0], pa[1]), (-10.0, 5.0), "the preview reads the draft");
    assert_ne!(slot_marks(&state, 1.5), before, "the screen marks moved");
    assert_eq!((committed_a(&state).x, committed_a(&state).y), (-20.0, 0.0), "nothing committed yet");
}

#[test]
fn no_marks_with_a_selection_or_off_the_geometry_tab() {
    let mut state = opened();
    state.op_tab = super::op_tabs::SEC_HEIGHTS;
    assert!(slot_marks(&state, 1.5).is_empty(), "only the Geometry tab shows them");
    state.op_tab = super::op_tabs::SEC_GEO;
    assert!(!slot_marks(&state, 1.5).is_empty());
    state.op_ui.geo.push(camcraft_app::operation::GeoSel {
        entity: "part".into(),
        feature: camcraft_app::operation::FeatureSel::Chain(0),
    });
    assert!(slot_marks(&state, 1.5).is_empty(), "a picked selection owns A and B");
}
