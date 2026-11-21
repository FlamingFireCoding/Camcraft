//! A synthetic Agie inspection request through the saved Wire Contour editor.
//! Gate map (M4.4): covers ui::egui_app::op_page, ui::state_per_op, app::operation::agie_inspection.

use super::super::dialog_tests::page;
use crate::state::{CommandKind, State};
use camcraft_app::commands::{DxfOptions, GenerateOperation, ImportDxf};
use camcraft_app::operation::OpParams;
use camcraft_app::{machine, model, project, sketch, stock::StockShape, wire_tools};
use egui_kittest::kittest::{NodeT, Queryable};

const BUTTON: &str = "Export Agie inspection setup";
const DXF: &str = "0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n4\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n\
  0\nLWPOLYLINE\n8\nnotch\n90\n6\n70\n1\n10\n-7\n20\n-5\n10\n7\n20\n-5\n10\n7\n20\n5\n10\n2\n20\n5\n42\n-1\n10\n-2\n20\n5\n10\n-7\n20\n5\n\
  0\nLINE\n8\nnotch\n10\n7\n20\n-5\n11\n-7\n21\n-5\n\
  0\nCIRCLE\n8\nframe\n10\n0\n20\n0\n40\n16\n0\nCIRCLE\n8\nother\n10\n12\n20\n0\n40\n2\n0\nENDSEC\n0\nEOF\n";

fn inspection_page() -> (State, String) {
    let mut st = State::new();
    let machine = machine::parse_mch(include_str!(
        "../../../../assets/machines/agiecut-classic-2s.mch"
    ))
    .expect("machine");
    st.pending.push_back(CommandKind::EditMachineApply {
        path: "wire.mch".into(),
        machine: Box::new(machine),
    });
    st.pending.push_back(CommandKind::SetStock {
        shape: StockShape::Box {
            dx: 1.0,
            dy: 1.0,
            dz: 10.0,
        },
        pos: [0.0, 0.0, -10.0],
        yaw_deg: 0.0,
    });
    st.pending.push_back(CommandKind::AddTool {
        name: "Wire".into(),
        tool: wire_tools::wire_tool(0.25),
    });
    st.drain_actions();
    st.history.execute(
        &st.world,
        ImportDxf::from_bytes(
            &st.world,
            None,
            "shape.dxf",
            DXF.as_bytes(),
            &DxfOptions::default(),
        )
        .expect("profile"),
    );
    let id = sketch::find(&st.world, "shape")
        .expect("sketch")
        .1
        .data
        .chains()
        .by_layer("notch")[0]
        .id();
    st.pending.push_back(CommandKind::CreateAgieInspection {
        name: "Opening".into(),
        tool: "Wire".into(),
        drawing: "shape".into(),
        chain: id,
        start: [0.0, -1.0],
        setup: camcraft_app::setup::active(&st.world),
    });
    st.drain_actions();
    assert!(
        model::find_by_name(&st.world, "Opening").is_some(),
        "{}",
        st.status
    );
    st.sync_ops_ui();
    (st, id.to_string())
}

#[test]
fn unassigned_inspection_opening_survives_undo_redo_and_project_reopen() {
    let (mut st, id) = inspection_page();
    assert!(st.history.undo(&st.world));
    assert!(model::find_by_name(&st.world, "Opening").is_none());
    assert!(st.history.redo(&st.world));
    let doc = serde_json::to_string(&project::collect(&st.world)).unwrap();
    let world = camcraft_app::world::new_world();
    project::open(&world, &doc).unwrap();
    let entity = model::find_by_name(&world, "Opening").unwrap();
    let op = model::operation_of(&world, &entity).unwrap();
    let OpParams::WireContour(w) = op.params else {
        panic!("wire contour")
    };
    assert!(w.condition.is_empty() && !w.capture);
    assert_eq!(w.start, Some([0.0, -1.0]));
    let camcraft_app::operation::SrcRef::Selection(picks) = op.src else {
        panic!("selection")
    };
    let camcraft_app::operation::FeatureSel::SketchChain(selected) = picks[0].feature else {
        panic!("one chain")
    };
    assert_eq!(id, selected.to_string());
    assert!(GenerateOperation::new(&world, "Opening").is_err());
}

#[test]
fn saved_selected_opening_exports_zero_steps_through_the_desktop_action() {
    let (st, id) = inspection_page();
    let mut h = page(st);
    h.run();
    h.get_by_label(BUTTON).click();
    h.run_steps(2);
    let st = h.state_mut();
    let Some(CommandKind::ExportAgieInspection(name)) = st.pending.pop_front() else {
        panic!("inspection action");
    };
    assert_eq!(name, "Opening");
    let parent = std::env::temp_dir().join(format!("camcraft-agie-view-{}", std::process::id()));
    std::fs::create_dir(&parent).expect("new directory");
    let dest = parent.join("CHECK");
    st.export_agie_inspection_to(&name, &dest);
    assert!(
        st.status.contains("0 working steps; inspection only"),
        "{}",
        st.status
    );
    assert!(st.status.contains(&id) && st.status.contains("6 nominal spans"));
    let iso = std::fs::read_to_string(dest.join("CHECK.ISO")).expect("ISO");
    assert_eq!(iso.matches(" G02 ").count(), 1);
    let sbl = std::fs::read_to_string(dest.join("CHECK.SBL")).expect("script");
    assert!(sbl.contains("ASDELETE") && !sbl.contains("ASCREATE"));
    assert!(GenerateOperation::new(&st.world, &name).is_err());
    st.export_agie_inspection_to("missing", &parent.join("REFUSED"));
    assert!(st.status.contains("refused"));
    assert!(!parent.join("REFUSED").exists());
    std::fs::remove_dir_all(parent).expect("remove test bundle");
}
