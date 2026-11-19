//! #1864: `egui_kittest::Harness` shell test for DXF sketch layer picking
//! and `mill2d_page` per-chain overrides driving `GenerateOperation` to `OpStatus::Ok`.
//!
//! Provenance: our own page harness test; no vendor material consulted.

use camcraft_app::commands::{DxfOptions, ImportDxf};
use camcraft_app::operation::{self, GeoSel, HeightReference, OpStatus};
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;

use super::*;

const TOOL: &str = "EM-0400-2FL";

fn plate_dxf() -> &'static str {
    "0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n4\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n\
     0\nLWPOLYLINE\n8\ncut\n90\n4\n70\n1\n10\n0\n20\n0\n10\n120\n20\n0\n10\n120\n20\n80\n10\n0\n20\n80\n\
     0\nCIRCLE\n8\nholes\n10\n25\n20\n55\n40\n8\n\
     0\nLINE\n8\nopen\n10\n20\n20\n10\n11\n100\n21\n10\n\
     0\nENDSEC\n0\nEOF\n"
}

fn contour_params() -> OpParams {
    let mut p = operation::seed_params(camcraft_app::operation::OpType::Contour.ordinal());
    if let OpParams::Contour { clearance_z, retract_z, top_z, bottom_z, .. } = &mut p {
        *clearance_z = Height::of(HeightReference::TopOfStock, 10.0);
        *retract_z = Height::of(HeightReference::TopOfStock, 5.0);
        *top_z = Height::of(HeightReference::TopOfStock, 0.0);
        *bottom_z = Height::of(HeightReference::TopOfStock, -2.0);
    }
    p
}

fn sketch_shell() -> ShellUi {
    let mut sh = ShellUi::blank();
    let state = &mut sh.state;
    state.pending.push_back(CommandKind::SetStock {
        shape: StockShape::Box { dx: 300.0, dy: 240.0, dz: 10.0 },
        pos: [0.0, 0.0, 0.0],
        yaw_deg: 0.0,
    });
    let tool = catalog().into_iter().find(|(n, _)| *n == TOOL).map(|(_, t)| t).expect("catalog");
    state.pending.push_back(CommandKind::AddTool { name: TOOL.into(), tool });
    state.drain_actions();

    let cmd = ImportDxf::from_bytes(&state.world, None, "plate.dxf", plate_dxf().as_bytes(), &DxfOptions::default())
        .expect("import dxf");
    state.history.execute(&state.world, cmd);

    state.pending.push_back(CommandKind::CreateOperation {
        name: "Contour1".into(),
        tool: TOOL.into(),
        src: SrcRef::Entity("plate".into()),
        params: contour_params(),
    });
    state.pending.push_back(CommandKind::Select(Some("Contour1".into())));
    state.drain_actions();
    state.sync_ops_ui();
    sh
}

#[test]
fn geo_page_layer_picker_and_mill2d_per_chain_overrides_work_and_generate() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(2400.0, 1200.0))
        .build_ui_state(frame, sketch_shell());
    harness.run_steps(2);

    // Click layer button "holes (1)" in the Geometry tab.
    harness.get_by_label("holes (1)").click();
    harness.run_steps(3);
    assert_eq!(
        harness.state().state.op_ui.geo,
        vec![GeoSel { entity: "plate".into(), feature: FeatureSel::SketchLayer("holes".into()) }],
        "clicking layer button should select SketchLayer"
    );
    let chips = harness.state().state.geo_chips();
    assert_eq!(chips.len(), 1);
    assert!(chips[0].label.contains("1 closed, 0 open"), "layer chip label: {}", chips[0].label);

    // Toggle "Reverse" under Per-chain overrides on the active Geometry tab.
    harness.get_by_label("Reverse").click();
    harness.run_steps(2);
    let has_reverse = match &harness.state().state.op_ui.params {
        OpParams::Contour { chain_overrides, .. } => chain_overrides.len() == 1 && chain_overrides[0].reverse,
        _ => false,
    };
    assert!(has_reverse, "clicking Reverse checkbox should set per-chain reverse override");

    // Apply draft and generate: operation should become OpStatus::Ok.
    let target = harness.state().state.op_ui.name.clone();
    let edited = harness.state().state.op_ui.params.clone();
    let st = &mut harness.state_mut().state;
    st.pending.push_back(CommandKind::EditOperationParams { name: target.clone(), params: edited });
    st.pending.push_back(CommandKind::GenerateOperation(target.clone()));
    st.drain_actions();

    let op_e = model::find_by_name(&harness.state().state.world, &target).expect("op entity");
    let op_status = operation::status_of(&harness.state().state.world, &op_e);
    assert_eq!(
        op_status,
        OpStatus::Ok,
        "expected OpStatus::Ok, got {:?}, status={}",
        op_status,
        harness.state().state.status
    );
    let tp = model::toolpath_of(&harness.state().state.world, &op_e).expect("generated toolpath");
    assert!(!tp.path.moves.is_empty());
}
