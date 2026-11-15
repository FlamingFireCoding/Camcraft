//! #1700: a stock copied from an incomplete model (#1610) carries the
//! model's hole itself, beside the fidelity-unknown sentinel it already
//! carried. Its refusal names that hole, not the pre-v59 "saved before
//! CamCraft recorded mesh fidelity" note, also once the source model is
//! deleted - across undo and redo, a setup clone, a whole re-import of the
//! source, and save/load.
//!
//! Gate map (#1700): covers app::mesh_open (hole_behind, note_for),
//! app::model_fidelity (op_problem), app::commands::set_stock,
//! app::commands::setup (clone), app::commands::setup_delete,
//! app::commands::update_model (stock copies), app::project (apply,
//! validate: a stock's `mesh_open`), app::posting (verification_report).
//!
//! Provenance: our own pins over our own laws; no vendor material.

use camcraft_app::commands::{
    AddTool, CreateOperation, CreateSetup, DeleteEntity, DeleteSetup, GenerateOperation, History, ImportModel, SetStock,
    UpdateModel,
};
use camcraft_app::geometry::{cube, Mesh, Point3};
use camcraft_app::operation::{Height, HeightReference, OpParams, SrcRef};
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;
use camcraft_app::world::new_world;
use camcraft_app::{mesh_open, model, model_fidelity, posting, project};
use flecs_ecs::prelude::World;

const MILL: &str = "EM-1000-4FL";
const HOLE: &str = "closed shell #7 (6 faces) meshes open: 4 boundary edges lie more than 1.00e-2 mm from any other face";
const SAID: &str = "imported incomplete - closed shell #7 (6 faces) meshes open";
const OLD: &str = "saved before CamCraft recorded";

fn brick() -> Mesh {
    cube(Point3::new(-20.0, -10.0, 0.0), Point3::new(20.0, 10.0, 10.0))
}

fn face() -> OpParams {
    OpParams::Face {
        holes: Default::default(),
        tilt: None,
        stepover: 3.0,
        clearance_z: Height::of(HeightReference::TopOfStock, 25.0),
        cut_z: Height::of(HeightReference::TopOfStock, 0.0),
        top_z: None,
        stepdown: None,
        feed: 1200.0,
        plunge_feed: 300.0,
        climb: true,
        blend_turns: false,
    }
}

fn incomplete(name: &str) -> ImportModel {
    let mut cmd = ImportModel::from_mesh(name, brick()).with_fidelity(Some(model_fidelity::UNKNOWN_MM));
    cmd.hole = Some(HOLE.to_string());
    cmd
}

fn hole(world: &World, name: &str) -> Option<String> {
    model::find_by_name(world, name).and_then(|e| mesh_open::hole_of(world, &e))
}

/// The stock this document's operations read.
fn stock_name(world: &World) -> String {
    camcraft_app::setup::stock_entity_name(world, camcraft_app::setup::active(world).as_deref())
}

/// The incomplete "part", a stock copied from it, and a Face op on the
/// stock, generated.
fn copied_stock() -> (World, History) {
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, incomplete("part"));
    h.execute(
        &world,
        SetStock::new(&world, StockShape::FromModel { source: "part".into() }, Default::default()).expect("stock"),
    );
    let tool = catalog().into_iter().find(|(n, _)| *n == MILL).map(|(_, t)| t).expect("catalog mill");
    h.execute(&world, AddTool::new(&world, MILL, tool).expect("tool"));
    h.execute(&world, CreateOperation::new(&world, "OP", MILL, SrcRef::Stock, face()).expect("create"));
    h.execute(&world, GenerateOperation::new(&world, "OP").expect("generate"));
    (world, h)
}

/// The op's refusal names the source and its hole, never the old note.
#[track_caller]
fn names_the_hole(world: &World) {
    let report = posting::verification_report(world);
    let want = format!("OP: model 'part' was {SAID}");
    assert!(report.problems.iter().any(|p| p.starts_with(&want)), "{:?}", report.problems);
    assert!(!report.problems.iter().any(|p| p.contains(OLD)), "{:?}", report.problems);
}

#[test]
fn a_stock_copy_carries_its_models_hole() {
    let (world, _h) = copied_stock();
    let stock = stock_name(&world);
    assert_eq!(hole(&world, &stock).as_deref(), Some(HOLE), "the copy carries the hole");
    names_the_hole(&world);
}

#[test]
fn the_copy_still_names_the_hole_once_its_source_is_deleted_across_undo() {
    let (world, mut h) = copied_stock();
    h.execute(&world, DeleteEntity::new(&world, "part").expect("delete the source"));
    assert!(model::find_by_name(&world, "part").is_none());
    names_the_hole(&world);
    assert!(h.undo(&world), "undo the delete");
    names_the_hole(&world);
    assert!(h.redo(&world), "redo the delete");
    names_the_hole(&world);
}

#[test]
fn undoing_the_stock_takes_the_hole_with_it() {
    let (world, mut h) = copied_stock();
    let stock = stock_name(&world);
    // A box over the copy drops the hole; undo brings the copy back with it.
    h.execute(&world, SetStock::new(&world, StockShape::Box { dx: 50.0, dy: 30.0, dz: 12.0 }, Default::default()).expect("box"));
    assert_eq!(hole(&world, &stock), None, "a box has no hole");
    assert!(h.undo(&world));
    assert_eq!(hole(&world, &stock).as_deref(), Some(HOLE), "undo restores the copy and its hole");
}

#[test]
fn a_whole_reimport_of_the_source_clears_the_copys_hole_and_undo_restores_it() {
    let (world, mut h) = copied_stock();
    let stock = stock_name(&world);
    h.execute(&world, UpdateModel::new(&world, "part", brick()).expect("whole re-import"));
    assert_eq!(hole(&world, &stock), None, "the copy holds the whole mesh now");
    assert!(h.undo(&world));
    assert_eq!(hole(&world, &stock).as_deref(), Some(HOLE), "undo trades the hole back with the facets");
}

#[test]
fn a_setup_clone_and_an_undone_setup_delete_keep_the_hole() {
    let (world, mut h) = copied_stock();
    h.execute(&world, CreateSetup::new(&world, None, None).expect("materialize Setup1"));
    h.execute(&world, CreateSetup::new(&world, Some("Setup2"), Some("Setup1")).expect("clone into Setup2"));
    let clone = camcraft_app::setup::stock_entity_name(&world, Some("Setup2"));
    assert_eq!(hole(&world, &clone).as_deref(), Some(HOLE), "the clone's copy carries it");
    h.execute(&world, DeleteSetup::new(&world, "Setup2").expect("delete Setup2"));
    assert!(model::find_by_name(&world, &clone).is_none(), "its stock went with it");
    assert!(h.undo(&world));
    assert_eq!(hole(&world, &clone).as_deref(), Some(HOLE), "an undone delete restores it");
}

#[test]
fn the_copys_hole_rides_the_project_file_after_its_source_is_gone() {
    let (world, mut h) = copied_stock();
    h.execute(&world, DeleteEntity::new(&world, "part").expect("delete the source"));
    let stock = stock_name(&world);
    let dir = std::env::temp_dir();
    let file = |tag: &str| dir.join(format!("camcraft-1700-{tag}-{}.camproj", std::process::id()));
    let (a, b, c) = (file("a"), file("b"), file("c"));
    project::save(&world, &a).expect("save");
    let text = std::fs::read_to_string(&a).expect("read");
    let doc: serde_json::Value = serde_json::from_str(&text).expect("json");
    let row = doc["entities"].as_array().and_then(|r| r.iter().find(|e| e["name"] == stock.as_str())).cloned().expect("stock row");
    assert_eq!(row["mesh_open"], serde_json::json!(HOLE), "{row}");
    assert_eq!(row["fidelity_unknown"], serde_json::json!(true), "{row}");
    let loaded = new_world();
    project::load(&loaded, &a).expect("load");
    assert_eq!(hole(&loaded, &stock).as_deref(), Some(HOLE));
    names_the_hole(&loaded);
    project::save(&loaded, &b).expect("save again");
    assert_eq!(text, std::fs::read_to_string(&b).expect("read"), "byte for byte");
    // A hole on a stock without the mark is corrupt, as on a model.
    let bad = text.replacen("\"fidelity_unknown\": true", "\"fidelity_unknown\": false", 1);
    assert_ne!(bad, text, "file layout changed");
    std::fs::write(&c, bad).expect("write");
    let err = project::load(&new_world(), &c).expect_err("a hole without its mark is corrupt");
    assert!(err.to_string().contains("names a mesh hole but is not a model marked fidelity unknown"), "{err}");
    for f in [a, b, c] {
        let _ = std::fs::remove_file(f);
    }
}
