//! #3100: the first traverse of a 3+2 plane on a machine whose table
//! tilts the work. The post sends Z, then X/Y at that height, then the
//! move with the rotary words (`fanuc.cps` `xyFirst`, `onRapid`). So the
//! X/Y traverse runs with each work-turning rotary where the previous
//! program left it, and the posed law (#3084) reads the work through
//! every such pose.
//!
//! - One rotary A on the table (about X, through the WCS origin): an A 30
//!   face plane over a 60 x 60 x 20 block. At the plane's own pose the
//!   block's top stands 15 over the plane origin; a full turn about A
//!   lifts the top edge (x, 30, 20) to radius sqrt(1300), 18.735 over it.
//!   A clearance 16 over the origin is held by name; 20 passes.
//! - A and B on the table (a trunnion): a B 30 face plane reads the block
//!   on its sphere about the WCS origin; 30 over the origin passes.
//!
//! Provenance: our own machine files and fixtures; the shop rule "never
//! traverse through uncut stock or steel" (machine-tool handbooks) and
//! plain rotation geometry. No vendor material.
//!
//! Gate map (#3100): covers app::operation (first_traverse_posed,
//! generate), sim::verify (first_traverse_posed).

use camcraft_app::commands::{AddTool, CreateOperation, GenerateOperation, History, LoadMachine, SetStock, SetWcsOrigin};
use camcraft_app::geometry::{Point3, IDENTITY_Q};
use camcraft_app::machine::parse_mch;
use camcraft_app::operation::{Height, OpParams, OpStatus, SrcRef, Tilt};
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;
use camcraft_app::verify::Warning;
use camcraft_app::{model, posting};
use flecs_ecs::prelude::World;

/// The plane origin: the block's top centre (Z 20).
const ORIGIN_Z: f64 = 20.0;

/// Which rotaries turn the work. The head never tilts.
#[derive(Clone, Copy)]
enum Rig {
    /// A about X on the table.
    TableA,
    /// A about X on the table, B about Y on A.
    Trunnion,
}

fn machine_text(rig: Rig) -> String {
    let rot = |id: &str, name: &str, axis: &str, inner: &str| {
        format!(r#"{{ "id": "{id}", "name": "{name}", "type": "rotary", "axis": {axis}, "travel": {{ "min": -120.0, "max": 120.0 }}{inner} }}"#)
    };
    let head = r#"{ "id": "hed", "name": "head", "type": "head", "spindle": { "min_speed": 50.0, "max_speed": 15000.0 } }"#;
    let (table, b) = match rig {
        Rig::TableA => (rot("ra", "A", "[1.0, 0.0, 0.0]", ""), ""),
        Rig::Trunnion => (rot("ra", "A", "[1.0, 0.0, 0.0]", &format!(r#", "parts": [ {} ]"#, rot("rb", "B", "[0.0, 1.0, 0.0]", ""))), r#", "B": { "coordinate": 4 }"#),
    };
    format!(
        r#"{{
  "general": {{ "model": "TEST TABLE TILT", "vendor": "Camcraft", "kind": "mill" }},
  "kinematics": {{ "default": {{
    "units": {{ "length": "mm", "angle": "degrees" }},
    "parts": [
      {{ "id": "base", "name": "table", "type": "table", "parts": [
        {{ "id": "crl", "name": "X", "type": "linear", "travel": {{ "min": -200.0, "max": 200.0 }}, "parts": [
          {{ "id": "sld", "name": "Y", "type": "linear", "travel": {{ "min": -200.0, "max": 200.0 }} }} ] }},
        {{ "id": "col", "name": "Z", "type": "linear", "travel": {{ "min": 0.0, "max": 300.0 }}, "safe_retract": 0.0, "parts": [ {head} ] }},
        {table}
      ] }}
    ] }} }},
  "controller": {{ "synced_configuration": {{ "parts": {{
    "X": {{ "coordinate": 0 }}, "Y": {{ "coordinate": 1 }}, "Z": {{ "coordinate": 2 }},
    "A": {{ "coordinate": 3 }}{b}
  }} }} }},
  "tooling": {{ "number_of_tools": 24 }}
}}"#
    )
}

/// A face plane at A `a_deg` / B `b_deg` through the top centre of a
/// 60 x 60 x 20 block (x, y +-30, z 0..20), its clearance `over` the
/// plane origin; the WCS origin on the rotary lines.
fn face_on(rig: Rig, a_deg: f64, b_deg: f64, over: f64) -> (World, OpStatus, Vec<Warning>) {
    let world = camcraft_app::world::new_world();
    let mut h = History::default();
    let m = parse_mch(&machine_text(rig)).unwrap_or_else(|e| panic!("test machine: {e}"));
    h.execute(&world, LoadMachine::new(&world, "test-table-tilt.mch".into(), Some(m)).expect("machine"));
    h.execute(&world, SetWcsOrigin::new(&world, Location { pos: Point3::ZERO, rot: IDENTITY_Q }).expect("G54 on the lines"));
    let at = Location { pos: Point3::ZERO, rot: IDENTITY_Q };
    h.execute(&world, SetStock::new(&world, StockShape::Box { dx: 60.0, dy: 60.0, dz: 20.0 }, at).expect("stock"));
    let t = catalog().into_iter().find(|(n, _)| *n == "EM-0400-2FL").map(|(_, t)| t).expect("catalog");
    h.execute(&world, AddTool::new(&world, "EM-0400-2FL", t).expect("tool"));
    let p = OpParams::Face {
        holes: Default::default(),
        tilt: Some(Tilt { basis_version: 1, a_deg, b_deg, origin: [0.0, 0.0, ORIGIN_Z] }),
        blend_turns: false,
        stepover: 3.0,
        clearance_z: Height::fixed(ORIGIN_Z + over),
        cut_z: Height::fixed(19.5),
        top_z: None,
        stepdown: None,
        feed: 1200.0,
        plunge_feed: 300.0,
        climb: true,
    };
    h.execute(&world, CreateOperation::new(&world, "Face1", "EM-0400-2FL", SrcRef::Stock, p).expect("create"));
    h.execute(&world, GenerateOperation::new(&world, "Face1").expect("generate"));
    let e = model::find_by_name(&world, "Face1").expect("op");
    let status = camcraft_app::operation::status_of(&world, &e);
    let warns = model::toolpath_of(&world, &e).expect("cached path").warnings.clone();
    (world, status, warns)
}

fn low(warns: &[Warning]) -> Option<(f64, f64, bool)> {
    warns.iter().find_map(|w| match *w {
        Warning::FirstTraverseLow { height_mm, top_mm, fixture, .. } => Some((height_mm, top_mm, fixture)),
        _ => None,
    })
}

#[test]
fn a_3_2_plane_under_the_work_a_turning_table_can_leave_turned_up_is_refused_by_name() {
    // The top edge (x, +-30, 20) on its circle about A: radius sqrt(1300);
    // read along the A 30 axis from the plane origin (0, 0, 20), the line
    // stands 20 cos 30 under it.
    let want = 1300f64.sqrt() - ORIGIN_Z * 30f64.to_radians().cos();
    assert!((want - 18.735).abs() < 1e-3, "{want}");
    let (world, status, warns) = face_on(Rig::TableA, 30.0, 0.0, 16.0);
    assert_eq!(status, OpStatus::Ok);
    let (height, top, fixture) = low(&warns).unwrap_or_else(|| panic!("clearance 16: no first-traverse verdict in {warns:?}"));
    assert!((height - 16.0).abs() < 1e-6, "the post traverses at the clearance: {height}");
    assert!((top - want).abs() < 1e-6 && !fixture, "the stock edge turned up: {top}, want {want}");
    let n = warns.iter().filter(|w| matches!(w, Warning::FirstTraverseLow { .. })).count();
    assert_eq!(n, 1, "one verdict names the highest top: {warns:?}");
    let err = posting::post_setup_unwritten(&world).expect_err("the program must not post");
    for text in ["Face1", "first rapid (move 0)"] {
        assert!(err.contains(text), "{text}: {err}");
    }
}

#[test]
fn a_3_2_plane_over_every_pose_of_a_turning_table_passes() {
    let (_, status, warns) = face_on(Rig::TableA, 30.0, 0.0, 20.0);
    assert_eq!(status, OpStatus::Ok);
    assert_eq!(low(&warns), None, "clearance 20 stands over the turned stock: {warns:?}");
    // The trunnion: the corner (30, 30, 0) or (30, 30, 20) on its sphere,
    // sqrt(2200) from the origin, 29.584 over the plane along B 30.
    let (_, status, warns) = face_on(Rig::Trunnion, 0.0, 30.0, 30.0);
    assert_eq!(status, OpStatus::Ok);
    assert_eq!(low(&warns), None, "clearance 30 stands over the sphere: {warns:?}");
}
