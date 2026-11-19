//! Corner relief (#2400): dogbone and T-bone relief for inside corners
//! of closed 2D pockets and square-ended slots, with reachability and
//! part-boundary / stock refusals, golden CL, and bounded gouge verification.
//!
//! Provenance: independent end-to-end tests over public CAM geometry rules
//! and the app operation, verify, stock_sim and MCP surfaces.

mod common;
mod pocket_support;

use camcraft_app::cl::{MoveKind, Toolpath};
use camcraft_app::commands::{AddTool, CreateOperation, GenerateOperation, History, ImportModel, SetStock};
use camcraft_app::geo2d::Point2;
use camcraft_app::geometry::{Mesh, Point3, cube};
use camcraft_app::machining2d::PocketStrategy;
use camcraft_app::mcp::Server;
use camcraft_app::model;
use camcraft_app::operation::{
    AreaControls, BoundaryMode, BoundaryShape, ExtraBoundary, FeatureSel, GeoSel, Height,
    HeightReference, OpParams, SrcRef,
};
use camcraft_app::project;
use camcraft_app::recognize;
use camcraft_app::stock::StockShape;
use camcraft_app::stock_sim::simulate_document;
use camcraft_app::tools::catalog;
use camcraft_app::verify::Warning;
use camcraft_app::world::new_world;
use common::{gen_to_cltext, write_golden};
use flecs_ecs::prelude::World;
use pocket_support::{block_with_pocket, height_field, rounded_rect};
use serde_json::{Value, json};

const TOOL: &str = "EM-0600-4FL";
const TOOL_R: f64 = 3.0;

fn setup_world(mesh: Option<Mesh>, dx: f64, dy: f64, dz: f64) -> (World, History) {
    let world = new_world();
    let mut h = History::default();
    if let Some(m) = mesh {
        h.execute(&world, ImportModel::from_mesh("part", m));
    }
    h.execute(
        &world,
        SetStock::new(&world, StockShape::Box { dx, dy, dz }, Default::default()).expect("stock"),
    );
    let t = catalog()
        .into_iter()
        .find(|(n, _)| *n == TOOL)
        .map(|(_, t)| t)
        .expect("EM-0600-4FL in catalog");
    h.execute(&world, AddTool::new(&world, TOOL, t).expect("tool"));
    (world, h)
}

fn patch_pocket(base: &OpParams, patch: Value) -> Result<OpParams, String> {
    let Value::Object(mut tagged) = project::params_to_json(base) else {
        return Err("expected params object".into());
    };
    let Some((key, Value::Object(fields))) = tagged.iter_mut().next().map(|(k, v)| (k.clone(), v)) else {
        return Err("expected tagged variant".into());
    };
    let Value::Object(patch_obj) = patch else {
        return Err("patch must be object".into());
    };
    for (k, v) in patch_obj {
        fields.insert(k, v);
    }
    let next = project::params_from_json(&Value::Object(tagged)).map_err(|e| format!("params_from_json: {e}"))?;
    let back = project::params_to_json(&next);
    let key_fields = back.get(&key).and_then(Value::as_object);
    if !key_fields.is_some_and(|m| m.contains_key("corner_relief")) {
        return Err(format!("params round-trip dropped corner_relief on {key}: {back}"));
    }
    Ok(next)
}

fn base_pocket(strategy: PocketStrategy, top: Height, bottom: Height, stepdown: f64) -> OpParams {
    let mut p = camcraft_app::operation::seed_params(camcraft_app::operation::OpType::Pocket.ordinal());
    if let OpParams::Pocket {
        stepover,
        stock_to_leave,
        top_z,
        bottom_z,
        stepdown: sd,
        strategy: st,
        clearance_z,
        entry,
        ..
    } = &mut p
    {
        *stepover = 3.0;
        *stock_to_leave = 0.0;
        *top_z = top;
        *bottom_z = bottom;
        *sd = stepdown;
        *st = strategy;
        *clearance_z = Height::of(HeightReference::TopOfStock, 5.0);
        *entry = camcraft_app::machining3d::Entry3D::Plunge;
    }
    p
}

fn min_dist_at_floor(tp: &Toolpath, corner: Point2, floor_z: f64) -> f64 {
    let mut best = f64::INFINITY;
    for m in &tp.moves {
        if m.kind != MoveKind::Feed {
            continue;
        }
        if (m.from.z - floor_z).abs() > 1e-3 || (m.to.z - floor_z).abs() > 1e-3 {
            continue;
        }
        let ax = m.from.x;
        let ay = m.from.y;
        let bx = m.to.x;
        let by = m.to.y;
        let dx = bx - ax;
        let dy = by - ay;
        let len2 = dx * dx + dy * dy;
        let t = if len2 <= 1e-18 {
            0.0
        } else {
            (((corner.x - ax) * dx + (corner.y - ay) * dy) / len2).clamp(0.0, 1.0)
        };
        let px = ax + t * dx;
        let py = ay + t * dy;
        best = best.min((corner.x - px).hypot(corner.y - py));
    }
    best
}

fn any_feed_outside_x(tp: &Toolpath) -> (f64, f64) {
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for m in &tp.moves {
        if m.kind == MoveKind::Feed {
            lo = lo.min(m.from.x.min(m.to.x));
            hi = hi.max(m.from.x.max(m.to.x));
        }
    }
    (lo, hi)
}

fn any_feed_outside_y(tp: &Toolpath) -> (f64, f64) {
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for m in &tp.moves {
        if m.kind == MoveKind::Feed {
            lo = lo.min(m.from.y.min(m.to.y));
            hi = hi.max(m.from.y.max(m.to.y));
        }
    }
    (lo, hi)
}

fn mcp_call(s: &mut Server, id: &mut i64, name: &str, args: Value) -> Result<Value, String> {
    *id += 1;
    let msg = json!({
        "jsonrpc": "2.0",
        "id": *id,
        "method": "tools/call",
        "params": { "name": name, "arguments": args }
    });
    let reply = s.handle(&msg).expect("JSON-RPC reply");
    let res = &reply["result"];
    if res["isError"] == json!(true) {
        return Err(res["content"][0]["text"].as_str().unwrap_or("").to_owned());
    }
    Ok(res["structuredContent"].clone())
}

#[test]
fn square_cad_pocket_dogbone_clears_corners_and_verifies_without_gouge() {
    // 60 x 40 x 20 block with a 20 x 20 square pocket (floor z 10, top z 20).
    let mesh = block_with_pocket(30.0, 20.0, &rounded_rect(10.0, 10.0, 0.0, 1), 10.0, 20.0);
    let (world, mut h) = setup_world(Some(mesh), 60.0, 40.0, 20.0);
    let rec = recognize::pockets_of(&world, "part").expect("recognize");
    assert_eq!(rec.len(), 1);
    let base = base_pocket(
        PocketStrategy::Waterline,
        Height::of(HeightReference::BottomOfSelection, 10.0),
        Height::of(HeightReference::BottomOfSelection, 0.0),
        5.0,
    );
    let (src, base_op) = recognize::operation_for(&rec[0], &base).expect("op for pocket");
    let params = patch_pocket(&base_op, json!({ "corner_relief": { "kind": "dogbone" } }))
        .expect("corner_relief param supported");

    h.execute(&world, CreateOperation::new(&world, "Pocket1", TOOL, src, params).expect("create"));
    h.execute(&world, GenerateOperation::new(&world, "Pocket1").expect("generate"));

    let e = model::find_by_name(&world, "Pocket1").expect("op entity");
    let cache = model::toolpath_of(&world, &e).expect("toolpath");
    assert!(
        !cache.warnings.iter().any(|w| matches!(w, Warning::Gouge { .. } | Warning::Crossing { .. })),
        "unexpected verify warnings: {:?}",
        cache.warnings
    );

    // Every square corner of [-10, 10]^2 must be reached at floor z = 10.0 by a center
    // within TOOL_R (<= 3.001 mm) so a 20x20 square mating part fits.
    for (cx, cy) in [(-10.0, -10.0), (10.0, -10.0), (10.0, 10.0), (-10.0, 10.0)] {
        let d = min_dist_at_floor(&cache.path, Point2::new(cx, cy), 10.0);
        assert!(d <= TOOL_R + 1e-3, "corner ({cx}, {cy}) left uncut: center dist {d}");
    }

    // Program stock simulation must pass with no unexcused gouge.
    let sim = simulate_document(&world).expect("simulate_document");
    assert!(
        sim.dev.max_gouge <= sim.dev.tol,
        "corner relief exceeded bounds: max_gouge={} tol={}",
        sim.dev.max_gouge,
        sim.dev.tol
    );
}

#[test]
fn square_slot_tbone_rule_and_per_corner_override() {
    // 60 x 40 x 20 block with a 30 x 12 square-ended slot (x in [-15, 15], y in [-6, 6], floor z 10).
    let mesh = block_with_pocket(30.0, 20.0, &rounded_rect(15.0, 6.0, 0.0, 1), 10.0, 20.0);
    let (world, mut h) = setup_world(Some(mesh), 60.0, 40.0, 20.0);
    let rec = recognize::pockets_of(&world, "part").expect("recognize");
    assert_eq!(rec.len(), 1);
    let base = base_pocket(
        PocketStrategy::Waterline,
        Height::of(HeightReference::BottomOfSelection, 10.0),
        Height::of(HeightReference::BottomOfSelection, 0.0),
        10.0,
    );
    let (src, base_op) = recognize::operation_for(&rec[0], &base).expect("op for slot");

    // 1. Long-edge rule: travels along the 30 mm X walls so Y walls stay flush at |y - tool_r| <= 3.0
    //    while X reaches +/-15.0 (cutting the short end walls).
    let long_edge = patch_pocket(
        &base_op,
        json!({ "corner_relief": { "kind": "t_bone", "tbone_direction": "long_edge" } }),
    )
    .expect("long_edge corner_relief supported");
    h.execute(&world, CreateOperation::new(&world, "SlotTBone", TOOL, src.clone(), long_edge).expect("create"));
    h.execute(&world, GenerateOperation::new(&world, "SlotTBone").expect("generate"));
    let e = model::find_by_name(&world, "SlotTBone").expect("op");
    let cache = model::toolpath_of(&world, &e).expect("toolpath");
    assert!(cache.warnings.is_empty(), "warnings: {:?}", cache.warnings);
    for (cx, cy) in [(-15.0, -6.0), (15.0, -6.0), (15.0, 6.0), (-15.0, 6.0)] {
        let d = min_dist_at_floor(&cache.path, Point2::new(cx, cy), 10.0);
        assert!(d <= TOOL_R + 1e-3, "slot corner ({cx}, {cy}) not cleared: {d}");
    }
    let (min_x, max_x) = any_feed_outside_x(&cache.path);
    let (min_y, max_y) = any_feed_outside_y(&cache.path);
    assert!(min_x <= -14.99 && max_x >= 14.99, "T-bone must extend into X end walls: {min_x}..{max_x}");
    assert!(min_y >= -3.001 && max_y <= 3.001, "long Y walls must stay un-notched: {min_y}..{max_y}");

    // 2. Vertical rule with per-corner overrides: (15, 6) = dogbone, (-15, -6) = none.
    let overrides = patch_pocket(
        &base_op,
        json!({
            "corner_relief": {
                "kind": "t_bone",
                "tbone_direction": "vertical",
                "overrides": [
                    { "at": [15.0, 6.0], "choice": "dogbone" },
                    { "at": [-15.0, -6.0], "choice": "none" }
                ]
            }
        }),
    )
    .expect("per-corner overrides supported");
    h.execute(&world, CreateOperation::new(&world, "SlotOverride", TOOL, src, overrides).expect("create"));
    h.execute(&world, GenerateOperation::new(&world, "SlotOverride").expect("generate"));
    let e2 = model::find_by_name(&world, "SlotOverride").expect("op2");
    let cache2 = model::toolpath_of(&world, &e2).expect("toolpath2");
    assert!(cache2.warnings.is_empty(), "warnings: {:?}", cache2.warnings);
    assert!(min_dist_at_floor(&cache2.path, Point2::new(15.0, 6.0), 10.0) <= TOOL_R + 1e-3);
    assert!(min_dist_at_floor(&cache2.path, Point2::new(-15.0, 6.0), 10.0) <= TOOL_R + 1e-3);
    // (-15, -6) left unrelieved by override -> center stops sqrt(2)*3 = 4.2426 away.
    let skipped = min_dist_at_floor(&cache2.path, Point2::new(-15.0, -6.0), 10.0);
    assert!(skipped > 4.0, "override none must leave (-15, -6) unrelieved: {skipped}");
}

#[test]
fn pocket_waterline_square_dogbone_matches_golden() {
    let mesh = block_with_pocket(30.0, 20.0, &rounded_rect(10.0, 10.0, 0.0, 1), 10.0, 20.0);
    let (world, mut h) = setup_world(Some(mesh), 60.0, 40.0, 20.0);
    let rec = recognize::pockets_of(&world, "part").expect("recognize");
    let base = base_pocket(
        PocketStrategy::Waterline,
        Height::of(HeightReference::BottomOfSelection, 10.0),
        Height::of(HeightReference::BottomOfSelection, 0.0),
        10.0,
    );
    let (src, base_op) = recognize::operation_for(&rec[0], &base).expect("op");
    let params = patch_pocket(&base_op, json!({ "corner_relief": { "kind": "dogbone" } }))
        .expect("corner_relief param supported");
    h.execute(&world, CreateOperation::new(&world, "GoldenDogbone", TOOL, src, params).expect("create"));
    h.execute(&world, GenerateOperation::new(&world, "GoldenDogbone").expect("generate"));
    let e = model::find_by_name(&world, "GoldenDogbone").expect("op");
    let cache = model::toolpath_of(&world, &e).expect("toolpath");
    assert!(cache.warnings.is_empty(), "golden warnings: {:?}", cache.warnings);
    write_golden("pocket_waterline_square_dogbone.cl", &gen_to_cltext(&cache.path));
}

#[test]
fn unreachable_corner_in_narrow_keyway_is_refused_by_name() {
    // 20 x 20 main pocket with a 4 mm wide rectangular keyway [10, 18] x [-2, 2].
    // The 6 mm cutter (radius 3) cannot fit inside the 4 mm keyway to reach corners (18, +/-2).
    let mesh = height_field(
        &[-25.0, -10.0, 10.0, 18.0, 25.0],
        &[-20.0, -10.0, -2.0, 2.0, 10.0, 20.0],
        |i, j| if (i == 1 && (1..=3).contains(&j)) || (i == 2 && j == 2) { 10.0 } else { 20.0 },
    );
    let (world, mut h) = setup_world(Some(mesh), 50.0, 40.0, 20.0);
    let rec = recognize::pockets_of(&world, "part").expect("recognize");
    assert_eq!(rec.len(), 1);
    let base = base_pocket(
        PocketStrategy::Waterline,
        Height::of(HeightReference::BottomOfSelection, 10.0),
        Height::of(HeightReference::BottomOfSelection, 0.0),
        10.0,
    );
    let (src, base_op) = recognize::operation_for(&rec[0], &base).expect("op");
    let params = patch_pocket(&base_op, json!({ "corner_relief": { "kind": "dogbone" } }))
        .expect("corner_relief param supported");
    h.execute(&world, CreateOperation::new(&world, "NarrowKeyway", TOOL, src, params).expect("create"));
    let op = model::operation_of(&world, &model::find_by_name(&world, "NarrowKeyway").unwrap()).unwrap();
    let err = camcraft_app::operation::generate(&world, &op, "NarrowKeyway")
        .expect_err("unreachable corner must be refused")
        .to_string();
    assert!(
        err.to_lowercase().contains("corner relief") && err.to_lowercase().contains("reach"),
        "refusal must name corner relief reachability: {err}"
    );
}

#[test]
fn relief_exiting_stock_or_thin_wall_or_exclude_is_refused_by_name() {
    // Case 1: thin 1 mm outer part wall (22 x 22 block with 20 x 20 pocket): a T-bone pokes 3 mm outside the part!
    let thin_mesh = block_with_pocket(11.0, 11.0, &rounded_rect(10.0, 10.0, 0.0, 1), 10.0, 20.0);
    let (w1, mut h1) = setup_world(Some(thin_mesh), 40.0, 40.0, 20.0);
    let rec1 = recognize::pockets_of(&w1, "part").expect("recognize");
    let base = base_pocket(
        PocketStrategy::Waterline,
        Height::of(HeightReference::BottomOfSelection, 10.0),
        Height::of(HeightReference::BottomOfSelection, 0.0),
        10.0,
    );
    let (src1, base_op1) = recognize::operation_for(&rec1[0], &base).expect("op");
    let tbone = patch_pocket(
        &base_op1,
        json!({ "corner_relief": { "kind": "t_bone", "tbone_direction": "horizontal" } }),
    )
    .expect("t_bone supported");
    h1.execute(&w1, CreateOperation::new(&w1, "ThinWall", TOOL, src1, tbone).expect("create"));
    let op1 = model::operation_of(&w1, &model::find_by_name(&w1, "ThinWall").unwrap()).unwrap();
    let err1 = camcraft_app::operation::generate(&w1, &op1, "ThinWall")
        .expect_err("relief breaking out of thin part wall must refuse")
        .to_string();
    assert!(err1.to_lowercase().contains("corner relief"), "must name corner relief: {err1}");

    // Case 2: pocket 0.5 mm from stock edge (40 x 40 stock around a 39 x 39 pocket boundary via Include)
    let (w2, mut h2) = setup_world(None, 40.0, 40.0, 20.0);
    let plate = cube(Point3::new(-19.5, -19.5, 0.0), Point3::new(19.5, 19.5, 10.0));
    h2.execute(&w2, ImportModel::from_mesh("pocket_box", plate));
    let top_face = model::features_of(&w2, &model::find_by_name(&w2, "pocket_box").unwrap())
        .unwrap()
        .faces
        .iter()
        .find(|f| f.normal[2] > 0.9)
        .expect("top face")
        .id;
    let src2 = SrcRef::Selection(vec![GeoSel { entity: "pocket_box".into(), feature: FeatureSel::Face(top_face) }]);
    let p2 = patch_pocket(
        &base_pocket(PocketStrategy::Waterline, Height::fixed(20.0), Height::fixed(10.0), 10.0),
        json!({ "corner_relief": { "kind": "dogbone" } }),
    )
    .expect("dogbone supported");
    h2.execute(&w2, CreateOperation::new(&w2, "OutStock", TOOL, src2.clone(), p2.clone()).expect("create"));
    let op2 = model::operation_of(&w2, &model::find_by_name(&w2, "OutStock").unwrap()).unwrap();
    let err2 = camcraft_app::operation::generate(&w2, &op2, "OutStock")
        .expect_err("relief exiting stock must refuse")
        .to_string();
    assert!(err2.to_lowercase().contains("stock"), "must name stock boundary: {err2}");

    // Case 3: Exclude boundary standing 0.2 mm past corner (10, 10) inside 60 x 40 stock
    let (w3, mut h3) = setup_world(None, 60.0, 40.0, 20.0);
    let pocket_slab = cube(Point3::new(-10.0, -10.0, 0.0), Point3::new(10.0, 10.0, 10.0));
    let guard_slab = cube(Point3::new(10.2, 8.0, 0.0), Point3::new(18.0, 18.0, 5.0));
    h3.execute(&w3, ImportModel::from_mesh("area_box", pocket_slab));
    h3.execute(&w3, ImportModel::from_mesh("guard_box", guard_slab));
    let f_area = model::features_of(&w3, &model::find_by_name(&w3, "area_box").unwrap())
        .unwrap()
        .faces
        .iter()
        .find(|f| f.normal[2] > 0.9)
        .unwrap()
        .id;
    let f_guard = model::features_of(&w3, &model::find_by_name(&w3, "guard_box").unwrap())
        .unwrap()
        .faces
        .iter()
        .find(|f| f.normal[2] > 0.9)
        .unwrap()
        .id;
    h3.execute(&w3, CreateOperation::new(&w3, "ExclCorner", TOOL, SrcRef::Selection(vec![GeoSel { entity: "area_box".into(), feature: FeatureSel::Face(f_area) }]), p2).expect("create"));
    let e3 = model::find_by_name(&w3, "ExclCorner").unwrap();
    let area = AreaControls {
        extra: vec![ExtraBoundary {
            mode: BoundaryMode::Exclude,
            shape: BoundaryShape::Picks(vec![GeoSel { entity: "guard_box".into(), feature: FeatureSel::Face(f_guard) }]),
        }],
        ..Default::default()
    };
    h3.execute(&w3, camcraft_app::commands::EditOperationParams::with_area(&w3, "ExclCorner", model::operation_of(&w3, &e3).unwrap().params, area).expect("edit"));
    let op3 = model::operation_of(&w3, &model::find_by_name(&w3, "ExclCorner").unwrap()).unwrap();
    let err3 = camcraft_app::operation::generate(&w3, &op3, "ExclCorner")
        .expect_err("relief entering Exclude boundary must refuse")
        .to_string();
    assert!(
        err3.to_lowercase().contains("exclude") || err3.to_lowercase().contains("corner relief"),
        "must name Exclude obstruction: {err3}"
    );
}

#[test]
fn mcp_set_operation_and_project_roundtrip_preserve_corner_relief() {
    let mesh = block_with_pocket(30.0, 20.0, &rounded_rect(10.0, 10.0, 0.0, 1), 10.0, 20.0);
    let (world, mut h) = setup_world(Some(mesh), 60.0, 40.0, 20.0);
    let rec = recognize::pockets_of(&world, "part").expect("recognize");
    let base = base_pocket(
        PocketStrategy::Waterline,
        Height::of(HeightReference::BottomOfSelection, 10.0),
        Height::of(HeightReference::BottomOfSelection, 0.0),
        10.0,
    );
    let (src, base_op) = recognize::operation_for(&rec[0], &base).expect("op");

    // 1. Project file save/load round-trip preserves corner_relief, and default none omits it.
    let edited = patch_pocket(
        &base_op,
        json!({ "corner_relief": { "kind": "t_bone", "tbone_direction": "horizontal" } }),
    )
    .expect("patch corner_relief");
    h.execute(&world, CreateOperation::new(&world, "SavedOp", TOOL, src, edited).expect("create"));
    let tmp = std::env::temp_dir().join(format!("camcraft_2400_rt_{}.camproj", std::process::id()));
    project::save(&world, &tmp).expect("save");
    let text = std::fs::read_to_string(&tmp).expect("read saved");
    assert!(text.contains("corner_relief"), "saved JSON must carry non-default corner_relief");
    let reloaded = new_world();
    project::load(&reloaded, &tmp).expect("load");
    let _ = std::fs::remove_file(&tmp);
    let op2 = model::operation_of(&reloaded, &model::find_by_name(&reloaded, "SavedOp").unwrap()).unwrap();
    let json2 = project::params_to_json(&op2.params);
    assert_eq!(json2["pocket"]["corner_relief"]["kind"], json!("t_bone"));
    assert_eq!(json2["pocket"]["corner_relief"]["tbone_direction"], json!("horizontal"));

    // 2. MCP add_operation / edit_operation accepts corner_relief, resets cleanly to none,
    // and enforces Trochoidal refusal on Adaptive2D.
    let mut s = Server::new();
    let mut id = 0;
    let _ = s.handle(&json!({ "jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {} }));
    mcp_call(&mut s, &mut id, "add_tool", json!({ "catalog": TOOL })).expect("tool");
    mcp_call(&mut s, &mut id, "set_stock", json!({ "box": { "dx": 40.0, "dy": 40.0, "dz": 10.0 } })).expect("stock");
    mcp_call(
        &mut s,
        &mut id,
        "add_operation",
        json!({
            "kind": "pocket",
            "tool": TOOL,
            "source": "stock",
            "name": "P1"
        }),
    )
    .expect("add_operation");
    let edited_mcp = mcp_call(
        &mut s,
        &mut id,
        "edit_operation",
        json!({
            "name": "P1",
            "params": {
                "corner_relief": {
                    "kind": "dogbone"
                }
            }
        }),
    )
    .expect("edit_operation accepts corner_relief");
    assert_eq!(edited_mcp["params"]["pocket"]["corner_relief"]["kind"], json!("dogbone"));

    let reset_mcp = mcp_call(
        &mut s,
        &mut id,
        "edit_operation",
        json!({
            "name": "P1",
            "params": {
                "corner_relief": {
                    "kind": "none"
                }
            }
        }),
    )
    .expect("edit_operation resets corner_relief to none");
    assert!(
        reset_mcp["params"]["pocket"].get("corner_relief").is_none(),
        "resetting corner_relief to none omits default field: {reset_mcp}"
    );
}

#[test]
fn adaptive2d_spiral_and_offset2d_support_relief_while_trochoidal_and_do_not_touch_refuse() {
    let mesh = block_with_pocket(30.0, 20.0, &rounded_rect(10.0, 10.0, 0.0, 1), 10.0, 20.0);
    let (world, _h) = setup_world(Some(mesh), 60.0, 40.0, 20.0);
    let rec = recognize::pockets_of(&world, "part").expect("recognize");
    let base = base_pocket(
        PocketStrategy::Waterline,
        Height::of(HeightReference::BottomOfSelection, 10.0),
        Height::of(HeightReference::BottomOfSelection, 0.0),
        10.0,
    );
    let (src, base_op) = recognize::operation_for(&rec[0], &base).expect("op");

    // Retype Pocket -> Adaptive2D (Spiral) and apply Dogbone corner relief.
    let adp_base = camcraft_app::operation::retype_params(
        &base_op,
        camcraft_app::operation::OpType::Adaptive2D.ordinal(),
    );
    let mut adp_dog = adp_base.clone();
    if let OpParams::Adaptive2D { corner_relief, .. } = &mut adp_dog {
        corner_relief.kind = camcraft_app::machining2d::CornerReliefKind::Dogbone;
    }
    let adp_op = camcraft_app::schema::Operation {
        tool: TOOL.into(),
        src: src.clone(),
        params: adp_dog.clone(),
        stamp: None,
        setup: None,
        area: Default::default(),
        coolant: Default::default(),
        after: Default::default(),
        suppressed: false,
    };
    let (tp, warns) = camcraft_app::operation::generate(&world, &adp_op, "AdpDog").expect("adaptive2d dogbone");
    assert!(!tp.moves.is_empty());
    assert!(warns.is_empty(), "adaptive2d dogbone verify must be silent: {warns:?}");

    // Trochoidal method with corner relief refuses by name.
    let mut adp_troch = adp_dog;
    if let OpParams::Adaptive2D { method, .. } = &mut adp_troch {
        *method = camcraft_app::machining2d::AdaptiveMethod::Trochoidal;
    }
    assert!(
        adp_troch.named_refusal().is_some(),
        "Adaptive2D Trochoidal + corner relief must be refused at validation"
    );
    let troch_op = camcraft_app::schema::Operation { params: adp_troch, ..adp_op.clone() };
    let troch_err = camcraft_app::operation::generate(&world, &troch_op, "AdpTroch")
        .expect_err("Adaptive2D Trochoidal + corner relief refuses generation")
        .to_string();
    assert!(troch_err.to_lowercase().contains("trochoidal"), "{troch_err}");

    // Retype Pocket -> Offset2D and apply Dogbone cleanly, then refuse when a DoNotTouch face collides with a corner.
    let off_base = camcraft_app::operation::retype_params(
        &base_op,
        camcraft_app::operation::OpType::Offset2D.ordinal(),
    );
    let mut off_dog = off_base;
    if let OpParams::Offset2D { corner_relief, .. } = &mut off_dog {
        corner_relief.kind = camcraft_app::machining2d::CornerReliefKind::Dogbone;
    }
    let off_op = camcraft_app::schema::Operation { params: off_dog, ..adp_op };
    let (off_tp, off_warns) = camcraft_app::operation::generate(&world, &off_op, "OffDog").expect("offset2d dogbone");
    assert!(!off_tp.moves.is_empty());
    assert!(off_warns.is_empty(), "offset2d dogbone verify must be silent: {off_warns:?}");

    // Separate clean DoNotTouch check on Offset2D: guard at (10.6, 10.6)..(18, 18) sits > 5 mm
    // from normal pocket pass (7, 7), but < 4.0 mm from dogbone spur end (7.879, 7.879) under 1 mm clearance.
    let (w4, mut h4) = setup_world(None, 60.0, 60.0, 20.0);
    h4.execute(&w4, ImportModel::from_mesh("area_box", cube(Point3::new(-10.0, -10.0, 0.0), Point3::new(10.0, 10.0, 10.0))));
    h4.execute(&w4, ImportModel::from_mesh("guard_box", cube(Point3::new(10.6, 10.6, 10.0), Point3::new(18.0, 18.0, 20.0))));
    let f_area = model::features_of(&w4, &model::find_by_name(&w4, "area_box").unwrap())
        .unwrap()
        .faces
        .iter()
        .find(|f| f.normal[2] > 0.9)
        .unwrap()
        .id;
    let f_guard = model::features_of(&w4, &model::find_by_name(&w4, "guard_box").unwrap())
        .unwrap()
        .faces
        .iter()
        .find(|f| f.normal[2] > 0.9)
        .unwrap()
        .id;
    let mut off_dnt_params = off_op.params.clone();
    if let OpParams::Offset2D { top_z, bottom_z, .. } = &mut off_dnt_params {
        *top_z = Height::fixed(20.0);
        *bottom_z = Height::fixed(10.0);
    }
    let mut off_dnt = camcraft_app::schema::Operation {
        tool: TOOL.into(),
        src: SrcRef::Selection(vec![GeoSel { entity: "area_box".into(), feature: FeatureSel::Face(f_area) }]),
        params: off_dnt_params,
        stamp: None,
        setup: None,
        area: Default::default(),
        coolant: Default::default(),
        after: Default::default(),
        suppressed: false,
    };
    off_dnt.area.do_not_touch.clearance = 1.0;
    off_dnt.area.do_not_touch.picks = vec![GeoSel { entity: "guard_box".into(), feature: FeatureSel::Face(f_guard) }];
    let dnt_err = camcraft_app::operation::generate(&w4, &off_dnt, "OffDnt")
        .expect_err("DoNotTouch clearance on guard face must refuse corner relief")
        .to_string();
    assert!(
        dnt_err.to_lowercase().contains("do not touch") || dnt_err.to_lowercase().contains("corner relief"),
        "must name Do not touch / corner relief: {dnt_err}"
    );
}
