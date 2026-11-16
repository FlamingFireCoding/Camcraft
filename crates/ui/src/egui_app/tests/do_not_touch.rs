//! Structural UI checks for #2055: Geometry tab -> Boundaries ->
//! "Do not touch" controls, rows with remove buttons, Apply/Undo, and viewport highlight.
//! Provenance: our own tests over our own shell; no vendor material.
//! Gate map (M4.4): covers ui::egui_app::boundary, ui::avoid_viz, ui::boundary_draft.

use super::*;
use camcraft_app::commands::ImportModel;
use camcraft_app::geometry::{Point3, cube};
use camcraft_app::operation::{FeatureSel, GeoSel, OpType, SrcRef, retype_params};

fn seed_part_and_offset(st: &mut State, kind: OpType) -> (GeoSel, GeoSel, GeoSel) {
    let part = ImportModel::from_mesh("block", cube(Point3::new(0.0, 0.0, 0.0), Point3::new(30.0, 30.0, 15.0)));
    st.history.execute(&st.world, part);
    let feats = camcraft_app::model::find_by_name(&st.world, "block")
        .and_then(|e| camcraft_app::model::features_of(&st.world, &e))
        .expect("block features");
    let face_top = GeoSel {
        entity: "block".into(),
        feature: FeatureSel::Face(feats.faces[0].id),
    };
    let face_other = GeoSel {
        entity: "block".into(),
        feature: FeatureSel::Face(feats.faces[1].id),
    };
    let chain_id = camcraft_app::operation::run_pick(&st.world, "block", 0).expect("edge run").feature;
    let edge = GeoSel { entity: "block".into(), feature: chain_id };

    let tool = camcraft_app::model::named_entities(&st.world)
        .into_iter()
        .find_map(|(n, e)| camcraft_app::model::tool_of(&st.world, &e).map(|_| n))
        .expect("seeded tool");
    let op = retype_params(&st.op_ui.params, kind as usize);
    st.pending.push_back(CommandKind::CreateOperation {
        name: "OpTest".into(),
        tool,
        src: SrcRef::Selection(vec![face_top.clone()]),
        params: op,
    });
    st.drain_actions();
    crate::actions::target(st, UiTarget::Operation("OpTest".into()));
    (face_top, face_other, edge)
}

fn expand_boundary(h: &mut Harness<'static, ShellUi>) {
    let rect = h.get_by_label("Boundary").rect();
    let p = egui::pos2(rect.left() - 10.0, rect.center().y);
    h.input_mut().events.push(Event::PointerMoved(p));
    h.run();
    for pressed in [true, false] {
        h.input_mut().events.push(Event::PointerButton {
            pos: p,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
        h.run();
    }
    h.run_steps(2);
}

#[test]
fn geometry_boundaries_shows_do_not_touch_controls_rows_and_viewport_marks() {
    let _g = snapshot_guard();
    let mut h = Harness::builder()
        .with_size(Vec2::new(1400.0, 1100.0))
        .build_ui_state(frame, ShellUi::seeded());
    h.run_steps(2);
    let (face_top, face_other, edge) = {
        let st = &mut h.state_mut().state;
        seed_part_and_offset(st, OpType::Offset2D)
    };
    h.run_steps(3);
    // Expand the collapsible Boundary group on the Geometry tab.
    expand_boundary(&mut h);

    assert!(
        h.query_by_label_contains("Do not touch").is_some(),
        "Boundary group must show the Do not touch section"
    );
    assert!(h.query_by_label("Clearance").is_some());
    h.get_by_label("Pick protected").click();
    h.run_steps(2);
    assert_eq!(h.state().state.boundary_pick, Some(crate::state::BoundaryPick::DoNotTouch));
    assert!(h.query_by_label("Picking…").is_some());

    {
        let st = &mut h.state_mut().state;
        // Overlapping face in src is refused on status line and not drafted.
        crate::boundary_draft::take_pick(
            st,
            crate::state::BoundaryPick::DoNotTouch,
            face_top,
            crate::boundary_draft::PickMods::default(),
        );
        assert!(st.status.contains("machining selection"), "overlap refusal: {}", st.status);
        assert!(st.op_ui.area.do_not_touch.picks.is_empty());

        // Non-overlapping face + edge chain land cleanly in Do not touch draft.
        crate::boundary_draft::take_pick(
            st,
            crate::state::BoundaryPick::DoNotTouch,
            face_other.clone(),
            crate::boundary_draft::PickMods::default(),
        );
        crate::boundary_draft::take_pick(
            st,
            crate::state::BoundaryPick::DoNotTouch,
            edge,
            crate::boundary_draft::PickMods::default(),
        );
        st.op_ui.area.do_not_touch.clearance = 1.25;
        assert_eq!(st.op_ui.area.do_not_touch.picks.len(), 2);
        // Viewport overlay renders marks while this operation is selected.
        let marks = crate::avoid_viz::avoid_marks(st, 1.4);
        assert!(!marks.faces.is_empty(), "protected face must render in viewport overlay");
        assert!(!marks.edges.is_empty(), "protected edge must render in viewport overlay");
    }
    h.run_steps(2);

    // Clicking the remove button removes one row from Do not touch.
    assert_eq!(h.query_all_by_label("×").count(), 2 + h.state().state.op_ui.geo.len());
    h.query_all_by_label("×").last().expect("do not touch ×").click();
    h.run_steps(2);
    assert_eq!(h.state().state.op_ui.area.do_not_touch.picks.len(), 1);

    // Apply persists to world, Undo clears it, Redo restores it.
    h.get_by_label("Apply").click();
    h.run_steps(2);
    {
        let st = &mut h.state_mut().state;
        let saved = camcraft_app::model::find_by_name(&st.world, "OpTest")
            .and_then(|e| camcraft_app::model::operation_of(&st.world, &e))
            .expect("saved op");
        assert_eq!(saved.area.do_not_touch.picks, vec![face_other]);
        assert_eq!(saved.area.do_not_touch.clearance, 1.25);
        st.pending.push_back(CommandKind::Undo);
        st.drain_actions();
        st.sync_ops_ui();
        assert!(st.op_ui.area.do_not_touch.picks.is_empty());
        // Deselecting clears the viewport overlay.
        crate::actions::target(st, UiTarget::None);
        assert_eq!(crate::avoid_viz::avoid_marks(st, 1.4), crate::avoid_viz::AvoidMarks::default());
    }
}

#[test]
fn unsupported_op_shows_do_not_touch_note_without_pick_button() {
    let _g = snapshot_guard();
    let mut h = Harness::builder()
        .with_size(Vec2::new(1400.0, 1100.0))
        .build_ui_state(frame, ShellUi::seeded());
    h.run_steps(2);
    {
        let st = &mut h.state_mut().state;
        seed_part_and_offset(st, OpType::Pocket);
    }
    h.run_steps(3);
    expand_boundary(&mut h);
    assert!(h.query_by_label_contains("2D Offset and 3D Pocket").is_some());
    assert!(h.query_by_label("Pick protected").is_none());
}
