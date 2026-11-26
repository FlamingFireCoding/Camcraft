//! Provenance: #4031's cutter hand - the persistence contract and the
//! refusal law, using our own starter catalog (the shape #1453's
//! `tap_hand.rs` set for the tap's thread hand).
//! Gate map: covers app::project (tool JSON), app::tool_gate, app::posting,
//! app::tool_library, app::commands (tool, operation), app::operation
//! (stamp_tool), tools::tools.

use camcraft_app::commands::{
    AddTool, ChangeTool, CreateOperation, GenerateOperation, History, ImportModel, ImportTools,
    LoadMachine, SelectPost, SetStock,
};
use camcraft_app::drilling::{CycleChoice, Hole};
use camcraft_app::geo2d::Point2;
use camcraft_app::geometry::{Mesh, Point3, Triangle};
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::schema::{Location, PostRef};
use camcraft_app::stock::StockShape;
use camcraft_app::tool_library::{export_tool_library, parse_tool_library};
use camcraft_app::world::new_world;
use camcraft_app::{model, posting, project, tools};
use flecs_ecs::prelude::World;

/// The posts that start a spindle M3 and never reverse it (the refusal
/// targets, #4031), and one that reverses, for the allow side.
const M3_ONLY: &[&str] = &["Haas", "Tormach PathPilot", "Doosan DNM", "Grbl 1.1", "grblHAL", "Makera Carvera", "Heidenhain", "Heidenhain Klartext"];

fn drill(hand: Option<&str>) -> tools::Tool {
    let t = tools::catalog().into_iter().find(|(n, _)| *n == "DR-0300").expect("catalog drill").1;
    let mut v = project::tool_to_json(&t);
    if let Some(hand) = hand {
        v["cutter_hand"] = serde_json::json!(hand);
    }
    project::tool_from_json(&v).expect("drill JSON")
}

fn tool(world: &World) -> tools::Tool {
    model::find_by_name(world, "Drill").and_then(|e| model::tool_of(world, &e)).expect("Drill tool")
}

fn assert_left(t: &tools::Tool) {
    assert_eq!(project::tool_to_json(t)["cutter_hand"], "left");
}

fn flat_plate() -> Mesh {
    // The already-drilled puck of rigid_tapping.rs / tap_hand.rs: a flat
    // plate with a pilot hole, so no unrelated reach refusal.
    let meridian = [(0.0, 0.0), (20.0, 0.0), (20.0, 10.0), (3.1, 10.0), (3.1, 2.0), (0.0, 2.0)];
    let v = |j: usize, r: f64, z: f64| {
        let th = std::f64::consts::TAU * j as f64 / 16.0;
        Point3::new(r * th.cos(), r * th.sin(), z)
    };
    let mut facets = Vec::new();
    for seg in meridian.windows(2) {
        let ((ra, za), (rb, zb)) = (seg[0], seg[1]);
        for j in 0..16 {
            let (a1, a2, b1, b2) = (v(j, ra, za), v(j + 1, ra, za), v(j, rb, zb), v(j + 1, rb, zb));
            if ra <= 1e-12 {
                facets.push(Triangle::new(a1, b2, b1));
            } else if rb <= 1e-12 {
                facets.push(Triangle::new(a1, a2, b2));
            } else {
                facets.push(Triangle::new(a1, a2, b2));
                facets.push(Triangle::new(a1, b2, b1));
            }
        }
    }
    Mesh::from_facets(facets)
}

fn drill_params() -> OpParams {
    OpParams::Drill {
        break_through: None,
        break_through_from: Default::default(),
        clearance_z: Height::fixed(25.0), retract_z: None, top_z: Height::fixed(10.0),
        bottom_z: None, cycle: CycleChoice::Drill, peck: 0.0, dwell: 0.0, feed: 100.0,
        plunge_feed: 100.0,
        holes: vec![Hole {
            xy: Point2::new(0.0, 0.0), radius: 3.05, entry_z: 10.0, floor_z: 2.0,
            through: false, stepped: false,
        }],
        cross: Vec::new(),
        picks: Default::default(),
    }
}

/// A world with a drilled plate and the named-hand drill, posting with
/// the document's own post (the built-in Fanuc unless selected away).
fn job(hand: Option<&str>) -> (World, History) {
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, LoadMachine::builtin(&world, "VMC 3-axis").expect("machine"));
    h.execute(&world, SetStock::new(&world, StockShape::Cylinder { diameter: 40.0, height: 10.0 }, Location::default()).expect("stock"));
    h.execute(&world, ImportModel::from_mesh("plate", flat_plate()));
    h.execute(&world, AddTool::new(&world, "Drill", drill(hand)).expect("add drill"));
    (world, h)
}

fn output(tag: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("camcraft-4031-{tag}-{}.nc", std::process::id()))
}

#[test]
fn left_hand_cutter_survives_tool_json_roundtrip_and_legacy_defaults_right() {
    let left = drill(Some("left"));
    assert_left(&left);
    assert_eq!(drill(None), drill(Some("right")), "legacy tools default to right-hand");
    let mut definition = project::tool_to_json(&drill(None));
    for bad in [serde_json::json!("backwards"), serde_json::json!(7), serde_json::Value::Null] {
        definition["cutter_hand"] = bad.clone();
        let err = project::tool_from_json(&definition).expect_err("unknown hand must refuse");
        assert!(err.contains("cutter hand") || err.contains("cutter_hand"), "{bad}: {err}");
    }
}

#[test]
fn creation_refuses_a_left_hand_cutter_by_name_on_every_m3_only_post() {
    for post in M3_ONLY {
        let (world, _h) = job(Some("left"));
        let mut h = History::default();
        h.execute(&world, SelectPost::new(&world, PostRef::Named((*post).into())).expect("select post"));
        let err = match CreateOperation::new(&world, "Drill1", "Drill", SrcRef::Entity("plate".into()), drill_params()) {
            Err(e) => e.to_string(),
            Ok(_) => panic!("{post} accepted a left-hand cutter"),
        };
        assert!(err.contains("left-hand cutter") && err.contains(post), "{post}: {err}");
    }
}

#[test]
fn creation_allows_a_left_hand_cutter_on_a_reversing_post() {
    let (world, _h) = job(Some("left"));
    let op = CreateOperation::new(&world, "Drill1", "Drill", SrcRef::Entity("plate".into()), drill_params())
        .expect("the built-in Fanuc post reverses the spindle");
    let _ = op;
}

#[test]
fn an_edit_to_left_hand_refuses_generation_and_post_until_the_post_reverses() {
    let (world, mut h) = job(None);
    h.execute(&world, CreateOperation::new(&world, "Drill1", "Drill", SrcRef::Entity("plate".into()), drill_params()).expect("drill op"));
    h.execute(&world, GenerateOperation::new(&world, "Drill1").expect("generate right-hand drill"));
    let path = output("right");
    let posted = posting::post_setup(&world, &path).expect("right-hand drill posts");
    assert!(!posted.program.is_empty());
    let _ = std::fs::remove_file(&path);

    // Edit the tool to left-hand: the op's own post is the built-in
    // Fanuc, which reverses - the op stays honest.
    h.execute(&world, ChangeTool::new(&world, "Drill", drill(Some("left"))).expect("change hand"));
    assert_left(&tool(&world));
    h.execute(&world, GenerateOperation::new(&world, "Drill1").expect("Fanuc regenerates a left-hand cutter"));

    // Switch the mount to an M3-only post: generation and post refuse by
    // name, and no NC file is written.
    h.execute(&world, SelectPost::new(&world, PostRef::Named("Haas".into())).expect("select Haas"));
    let gen_err = match GenerateOperation::new(&world, "Drill1") {
        Err(e) => e.to_string(),
        Ok(_) => panic!("generation accepted a left-hand cutter on Haas"),
    };
    assert!(gen_err.contains("left-hand cutter"), "{gen_err}");
    let path = output("haas");
    let _ = std::fs::remove_file(&path);
    let post_err = posting::post_setup(&world, &path).expect_err("posting refuses");
    assert!(post_err.contains("left-hand cutter") && post_err.contains("Drill1"), "{post_err}");
    assert!(!path.exists(), "a refusal writes no NC file");
}

#[test]
fn hand_survives_project_and_library_and_the_library_steps_to_version_7() {
    let (world, _h) = job(Some("left"));
    let saved = project::save_text(&world).expect("save");
    let value: serde_json::Value = serde_json::from_str(&saved).expect("project JSON");
    assert!(value["version"].as_u64().expect("version") >= 83, "old builds must refuse left-hand projects");
    let loaded = new_world();
    project::open(&loaded, &saved).expect("load");
    assert_left(&tool(&loaded));

    let library = export_tool_library(&world).expect("export library");
    let value: serde_json::Value = serde_json::from_str(&library).expect("library JSON");
    assert_eq!(value["version"], 7, "old library readers must refuse a left-hand cutter");
    let lib = parse_tool_library(&library).expect("read the v7 library");
    let imported = new_world();
    let mut ih = History::default();
    ih.execute(&imported, ImportTools::new(&imported, lib).expect("import"));
    assert_left(&tool(&imported));

    // A library without a left-hand cutter keeps its older number.
    let (right_world, _h) = job(None);
    let legacy = export_tool_library(&right_world).expect("right-hand library");
    let value: serde_json::Value = serde_json::from_str(&legacy).expect("library JSON");
    assert_eq!(value["version"], 1, "right-hand-only libraries keep their format");
    assert_eq!(parse_tool_library(&legacy).expect("legacy library").len(), 1);
}
