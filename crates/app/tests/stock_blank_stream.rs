//! #2769: the stream states the setup's stock as a controller blank
//! (`PARAM StockBlank`, `PARAM StockBlankZBottom`), read from the document:
//! a box, a cylinder with its world axis, or the reason no blank form fits.
//! The Fanuc post's opt-in Manual Guide i option writes G1902 from it.
//!
//! Provenance: our own fixtures; the blank form from FANUC MANUAL GUIDE i
//! (B-63874EN), blank figure - behaviour only.
//!
//! Gate map (M4.4): covers app::posting (op_streams: the StockBlank PARAMs),
//! app::posting_stock_blank, post::post (dump; the engine on fanuc.cps).
//! Lane shape (M4.4): all tests here run in seconds.

use camcraft_app::commands::{AddTool, CreateOperation, CreateSetup, GenerateOperation, History, SetStock};
use camcraft_app::geometry::{Point3, quat_from_euler};
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::post::{cnc, engine};
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;
use camcraft_app::world::new_world;
use flecs_ecs::prelude::World;
use serde_json::{Value, json};
use std::path::Path;

const TOOL: &str = "EM-0400-2FL";

fn read(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// One generated face pass on `stock` placed at `at`.
fn world_with(stock: StockShape, at: Location) -> World {
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, SetStock::new(&world, stock, at).expect("stock"));
    let tool = catalog().into_iter().find(|(n, _)| *n == TOOL).map(|(_, t)| t).expect("tool");
    h.execute(&world, AddTool::new(&world, TOOL, tool).expect("add tool"));
    h.execute(&world, CreateSetup::new(&world, None, None).expect("materialize Setup1"));
    let face = OpParams::Face {
        holes: Default::default(),
        blend_turns: false,
        tilt: None,
        stepover: 3.0,
        clearance_z: Height::fixed(50.0),
        cut_z: Height::fixed(19.5),
        top_z: None,
        stepdown: None,
        feed: 1200.0,
        plunge_feed: 300.0,
        climb: true,
    };
    h.execute(&world, CreateOperation::new(&world, "Face1", TOOL, SrcRef::Stock, face).expect("create"));
    h.execute(&world, GenerateOperation::new(&world, "Face1").expect("generate"));
    world
}

fn records(world: &World) -> Vec<cnc::Record> {
    camcraft_app::posting::op_streams(world).expect("streams").into_iter().flat_map(|(_, r)| r).collect()
}

fn param(records: &[cnc::Record], n: &str) -> Option<Value> {
    records.iter().find_map(|r| match r {
        cnc::Record::Param { name, value } if name == n => Some(value.clone()),
        _ => None,
    })
}

fn blank_of(stock: StockShape, at: Location) -> (Option<Value>, Option<Value>) {
    let r = records(&world_with(stock, at));
    (param(&r, "StockBlank"), param(&r, "StockBlankZBottom"))
}

fn turned(roll: f64, pitch: f64, yaw: f64) -> Location {
    Location { pos: Point3::ZERO, rot: quat_from_euler(roll, pitch, yaw) }
}

#[test]
fn the_stream_states_a_box_and_its_bottom_face() {
    let (shape, bottom) = blank_of(StockShape::Box { dx: 60.0, dy: 40.0, dz: 20.0 }, Location::default());
    assert_eq!(shape, Some(json!("box")));
    assert_eq!(bottom.and_then(|v| v.as_f64()), Some(0.0));
}

#[test]
fn the_stream_states_a_cylinder_by_its_world_axis() {
    let bar = || StockShape::Cylinder { diameter: 50.0, height: 20.0 };
    assert_eq!(blank_of(bar(), Location::default()).0, Some(json!("cylinder-z")));
    // A quarter turn about Y lays the bar's axis along X.
    let (shape, bottom) = blank_of(bar(), turned(0.0, std::f64::consts::FRAC_PI_2, 0.0));
    assert_eq!(shape, Some(json!("cylinder-x")));
    assert!(bottom.is_some(), "a cylinder states its bottom face");
}

#[test]
fn a_stock_turned_off_the_machine_axes_states_why_and_no_bottom() {
    let (shape, bottom) = blank_of(StockShape::Box { dx: 60.0, dy: 40.0, dz: 20.0 }, turned(0.0, 0.0, 0.5));
    assert_eq!(shape, Some(json!("rotated-stock")));
    assert_eq!(bottom, None);
}

#[test]
fn the_fanuc_option_posts_the_documents_box_as_g1902() {
    let world = world_with(StockShape::Box { dx: 60.0, dy: 40.0, dz: 20.0 }, Location::default());
    let r = records(&world);
    let o = |n: &str| param(&r, n).and_then(|v| v.as_f64()).expect(n);
    let (left, front) = (o("StockXYLeft"), o("StockXYFront"));
    let mch: Value = serde_json::from_str(&read("assets/machines/3axis-vmc.mch")).expect("machine json");
    let out = engine::run_with(
        &read("assets/posts/fanuc.cps"),
        &mch,
        &cnc::serialize(&r),
        &json!({ "manualGuideBlank": true }),
    )
    .expect("posts");
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    let fmt = |v: f64| {
        let s = format!("{v:.3}");
        let s = s.trim_end_matches('0').to_owned();
        if s == "-0." { "0.".to_owned() } else { s }
    };
    let want = format!(
        "G1902 B60. D40. H20. I{} J{} K{}",
        fmt(o("OriginX") - left),
        fmt(o("OriginY") - front),
        fmt(o("OriginZ")),
    );
    assert!(out.program.lines().any(|l| l == want), "want {want}:\n{}", out.program);
}

#[test]
fn a_post_without_blank_params_keeps_the_viewer_on_the_original_moves() {
    let world = world_with(
        StockShape::Box {
            dx: 60.0,
            dy: 40.0,
            dz: 20.0,
        },
        Location::default(),
    );
    let original = records(&world);
    assert!(
        param(&original, "StockBlank").is_some(),
        "the source includes blank params"
    );
    let expected: Vec<[f64; 3]> = original
        .iter()
        .filter_map(|r| match r {
            cnc::Record::Move(m) if m.kind != camcraft_app::cl::MoveKind::Rapid => {
                Some([m.x, m.y, m.z])
            }
            _ => None,
        })
        .collect();
    let cps = read("assets/posts/fanuc.cps").replace("stockBlank = true;", "stockBlank = false;");
    let path =
        std::env::temp_dir().join(format!("camcraft-blank-viewer-{}.nc", std::process::id()));
    let result = camcraft_app::posting::post_script(&world, &path, &cps);
    let _ = std::fs::remove_file(&path);
    let posted = result.expect("posts");
    let actual: Vec<_> = posted
        .program
        .lines()
        .enumerate()
        .filter(|(_, line)| line.starts_with("G01"))
        .map(|(line, _)| posted.line_pos[line].expect("the feed line has a viewer position"))
        .collect();
    assert!(!expected.is_empty(), "the fixture has feed moves");
    assert_eq!(
        actual, expected,
        "the viewer uses the original stream's world positions"
    );
}
