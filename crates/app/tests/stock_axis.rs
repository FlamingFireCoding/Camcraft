//! #2576: a stock laid along X or Y, or turned by custom angles - the turn
//! lives in the stock's `Location.rot`, lands through the undoable
//! `SetStock` (also from the MCP `set_stock` tool's `axis` / `angles_deg`),
//! survives the project file, and a DXF profile is placed on a turned
//! stock's real top face or refused by name.
//!
//! Gate map: covers app::mcp (set_stock, undo, save_project), app::commands
//! (SetStock + History, ImportDxf), app::project (save_text, open),
//! app::sketch (stock_face), app::stock_follow (sim_frame).
//!
//! Provenance: our own command, file and placement laws; standard
//! rigid-body math. No vendor material.

use serde_json::{json, Value};

use camcraft_app::commands::{History, SetStock};
use camcraft_app::geometry::stock_axis::{StockAxis, stock_axis_of, stock_axis_rot};
use camcraft_app::geometry::{Point3, quat_rotate_point};
use camcraft_app::mcp::Server;
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::world::new_world;
use camcraft_app::{model, project, sketch};

fn rot_of(axis: StockAxis, angles: [f64; 3]) -> [f64; 4] {
    stock_axis_rot(axis, angles).expect("finite angles have a turn")
}

fn stock_rot(world: &flecs_ecs::prelude::World) -> [f64; 4] {
    let stock = camcraft_app::setup::stock_entity(world, None).expect("a stock");
    model::transform_of(world, &stock).expect("a stock pose").rot
}

fn same_turn(a: [f64; 4], b: [f64; 4]) -> bool {
    1.0 - a.iter().zip(b).map(|(x, y)| x * y).sum::<f64>().abs() <= 1e-9
}

/// SetStock stores the X turn; undo restores the upright stock.
#[test]
fn set_stock_lays_a_cylinder_along_x_and_undo_restores_the_old_turn() {
    let world = new_world();
    let mut h = History::default();
    let cyl = StockShape::Cylinder { diameter: 40.0, height: 100.0 };
    h.execute(&world, SetStock::new(&world, cyl.clone(), Location::default()).expect("upright"));
    let laid = Location { rot: rot_of(StockAxis::X, [0.0; 3]), ..Default::default() };
    h.execute(&world, SetStock::new(&world, cyl, laid).expect("laid"));
    assert_eq!(stock_axis_of(stock_rot(&world)).0, StockAxis::X);
    assert!(h.undo(&world));
    assert!(same_turn(stock_rot(&world), [0.0, 0.0, 0.0, 1.0]), "undo restores the upright turn");
}

/// A custom turn survives a save and an open, angle for angle.
#[test]
fn a_turned_stock_survives_the_project_file() {
    let world = new_world();
    let mut h = History::default();
    let rot = rot_of(StockAxis::Custom, [30.0, 15.0, -10.0]);
    let at = Location { pos: Point3::new(5.0, -7.0, 2.0), rot };
    h.execute(&world, SetStock::new(&world, StockShape::Box { dx: 50.0, dy: 40.0, dz: 20.0 }, at).expect("stock"));
    let text = project::save_text(&world).expect("saves");
    let again = new_world();
    project::open(&again, &text).expect("opens");
    assert!(same_turn(stock_rot(&again), rot), "the turn is kept: {:?}", stock_rot(&again));
    let (axis, angles) = stock_axis_of(stock_rot(&again));
    assert_eq!(axis, StockAxis::Custom);
    for (got, want) in angles.iter().zip([30.0, 15.0, -10.0]) {
        assert!((got - want).abs() < 1e-6, "angles {angles:?}");
    }
    let laid = Location { rot: rot_of(StockAxis::Y, [0.0; 3]), ..Default::default() };
    h.execute(&world, SetStock::new(&world, StockShape::Cylinder { diameter: 30.0, height: 80.0 }, laid).expect("laid"));
    let again = new_world();
    project::open(&again, &project::save_text(&world).expect("saves")).expect("opens");
    assert_eq!(stock_axis_of(stock_rot(&again)).0, StockAxis::Y, "a laid cylinder reopens laid");
}

fn mcp_call(server: &mut Server, id: i64, name: &str, args: Value) -> (bool, Value) {
    let msg = json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call",
        "params": { "name": name, "arguments": args } });
    let reply = server.handle(&msg).expect("a request gets a response");
    let result = &reply["result"];
    let is_err = result["isError"] == json!(true);
    let body = if is_err { result["content"][0]["text"].clone() } else { result["structuredContent"].clone() };
    (is_err, body)
}

/// The stock turn the server's document holds, read through a save.
fn saved_rot(s: &mut Server, id: i64, dir: &std::path::Path) -> [f64; 4] {
    let path = dir.join(format!("stock-{id}.camproj")).to_string_lossy().into_owned();
    let (err, body) = mcp_call(s, id, "save_project", json!({ "path": path }));
    assert!(!err, "save refused: {body}");
    let world = new_world();
    project::load(&world, std::path::Path::new(&path)).expect("loads");
    stock_rot(&world)
}

/// MCP `set_stock` takes `axis` or `angles_deg`, refuses both or a bad
/// axis by name, and `undo` walks the turn back.
#[test]
fn mcp_set_stock_takes_an_axis_and_undo_walks_it_back() {
    let dir = std::env::temp_dir().join(format!("camcraft-2576-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let mut s = Server::new();
    let cyl = json!({ "diameter": 40.0, "height": 100.0 });
    let (err, body) = mcp_call(&mut s, 1, "set_stock", json!({ "cylinder": cyl }));
    assert!(!err, "upright refused: {body}");
    assert_eq!(body["axis"], json!("Along Z"));
    let (err, body) = mcp_call(&mut s, 2, "set_stock", json!({ "cylinder": cyl, "axis": "y" }));
    assert!(!err, "axis y refused: {body}");
    assert_eq!(body["axis"], json!("Along Y"));
    assert_eq!(stock_axis_of(saved_rot(&mut s, 3, &dir)).0, StockAxis::Y);
    let (err, body) = mcp_call(&mut s, 4, "set_stock", json!({ "cylinder": cyl, "angles_deg": { "yaw": 45.0 } }));
    assert!(!err, "angles refused: {body}");
    assert_eq!(body["axis"], json!("Custom"));
    let (err, body) = mcp_call(&mut s, 5, "set_stock", json!({ "cylinder": cyl, "axis": "x", "angles_deg": {} }));
    assert!(err && body.as_str().is_some_and(|t| t.contains("not both")), "both refused: {body}");
    let (err, body) = mcp_call(&mut s, 6, "set_stock", json!({ "cylinder": cyl, "axis": "w" }));
    assert!(err && body.as_str().is_some_and(|t| t.contains("bad axis")), "bad axis refused: {body}");
    let (err, body) = mcp_call(&mut s, 7, "undo", json!({}));
    assert!(!err, "undo refused: {body}");
    assert_eq!(stock_axis_of(saved_rot(&mut s, 8, &dir)).0, StockAxis::Y, "undo restores the Y turn");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A box laid square takes a profile on the face that is on top; a laid
/// cylinder is refused by name, and DXF import says so.
#[test]
fn a_profile_lies_on_a_laid_box_and_a_laid_cylinder_is_refused() {
    let world = new_world();
    let mut h = History::default();
    let (dx, dy, dz) = (50.0, 40.0, 20.0);
    let at = Location { pos: Point3::new(3.0, 4.0, 5.0), rot: rot_of(StockAxis::X, [0.0; 3]) };
    h.execute(&world, SetStock::new(&world, StockShape::Box { dx, dy, dz }, at).expect("box"));
    let top = [-0.5, 0.5]
        .iter()
        .flat_map(|&x| [-0.5, 0.5].map(move |y| (x, y)))
        .flat_map(|(x, y)| [0.0, 1.0].map(move |z| Point3::new(x * dx, y * dy, z * dz)))
        .map(|p| quat_rotate_point(at.rot, p).z + at.pos.z)
        .fold(f64::MIN, f64::max);
    let face = sketch::stock_face(&world, None).expect("a laid box has a top face").expect("a stock");
    assert!((face.pos.z - top).abs() < 1e-9, "the face is at the top: {} vs {top}", face.pos.z);
    let up = quat_rotate_point(face.rot, Point3::new(0.0, 0.0, 1.0));
    assert!(up.z > 1.0 - 1e-9, "the face looks up: {up:?}");

    let laid = Location { rot: rot_of(StockAxis::Y, [0.0; 3]), ..Default::default() };
    h.execute(&world, SetStock::new(&world, StockShape::Cylinder { diameter: 40.0, height: 100.0 }, laid).expect("cyl"));
    assert_eq!(sketch::stock_face(&world, None), Err(sketch::NO_TOP_FACE));
    assert!(sketch::face_frame(&world, None).is_none());
    let dxf = b"0\nSECTION\n2\nENTITIES\n0\nENDSEC\n0\nEOF\n";
    let opts = camcraft_app::commands::DxfOptions::default();
    let refused = camcraft_app::commands::ImportDxf::from_bytes(&world, None, "p.dxf", dxf, &opts);
    assert!(
        matches!(&refused, Err(e) if e.to_string().contains("no flat top face")),
        "DXF import names the refusal: {:?}",
        refused.as_ref().err().map(ToString::to_string)
    );
}
