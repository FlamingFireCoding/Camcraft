//! Gate map: covers ui::egui_app::heights_page, ui::operations.
//! Regression pins for height relationship notes in the production Heights page.
//! Provenance: the asymmetric fixture and local tool-axis display law from #2416.
use super::*;

fn retract_page(retract: f64) -> Harness<'static, State> {
    let mut state = asymmetric_tilted_pocket();
    state.op_ui.params = with_retract(state.op_ui.params, Height::fixed(retract));
    heights_tab(state)
}

#[test]
fn tilted_retract_below_clearance_does_not_report_setup_z_order() {
    let mut harness = retract_page(23.5);
    harness.run();
    assert!(harness.query_by_label("(= 21.000 along tool axis)").is_some());
    assert!(harness.query_by_label("(= 20.500 along tool axis)").is_some());
    assert!(harness.query_by_label_contains("above clearance -").is_none(),
        "local retract 20.5 is below local clearance 21");
}

#[test]
fn tilted_retract_above_clearance_names_the_local_hop_height() {
    let mut harness = retract_page(25.5);
    harness.run();
    assert!(harness.query_by_label("(= 21.000 along tool axis)").is_some());
    assert!(harness.query_by_label("(= 22.500 along tool axis)").is_some());
    assert!(harness.query_by_label("above clearance - the hop travels at 21.000 (min of the two)").is_some(),
        "the note must use the same folded local clearance as its row");
}

#[test]
fn retract_note_follows_an_unapplied_tilt_edit() {
    let mut state = asymmetric_tilted_pocket();
    state.op_ui.params = with_retract(state.op_ui.params, Height::fixed(37.0));
    if let OpParams::Pocket { tilt: Some(tilt), .. } = &mut state.op_ui.params {
        tilt.a_deg = 0.0;
        tilt.b_deg = 180.0;
    }
    let mut harness = heights_tab(state);
    harness.run();
    assert!(harness.query_by_label("(= 33.000 along tool axis)").is_some());
    assert!(harness.query_by_label("(= 34.000 along tool axis)").is_some());
    assert!(harness.query_by_label("above clearance - the hop travels at 33.000 (min of the two)").is_some(),
        "the note must read the draft frame rather than the saved A=-90 frame");
}

#[test]
fn tilted_corner_note_uses_the_same_local_frame_as_its_top_row() {
    let mut state = asymmetric_tilted_pocket();
    state.op_ui.params = OpParams::Chamfer2D {
        width: 1.0, angle_deg: 45.0,
        clearance_z: Height::of(R::TopOfStock, 10.0), retract_z: None,
        top_z: Height::of(R::TopOfStock, 0.0),
        feed: 800.0, plunge_feed: 200.0, climb: true,
        side: camcraft_app::machining2d::Side::Outside,
        tolerance: 0.01, tilt: state.op_ui.params.tilt(),
    };
    let mut harness = heights_tab(state);
    harness.run();
    assert!(harness.query_by_label("(= 11.000 along tool axis)").is_some());
    assert!(harness.query_by_label("the corner stands at 10.000 - derived: width x tan(angle) below the top plane").is_some(),
        "the corner is one unit below the local top, not below setup Z");
}
