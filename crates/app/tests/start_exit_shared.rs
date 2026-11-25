//! The shared start / exit pick mechanism (#3324, design #1448): one
//! capability per kind in the registry, one accessor that reads the picks
//! of every kind, one refusal that names the kind and its reason, and the
//! MCP doors (`edit_operation`'s schema, `list_operation_types`) that say
//! which kinds take a pick.
//!
//! Gate map (#3324): covers app::operation (kinds pick capability,
//! path_picks accessor, sane), app::mcp (tools schema, job kind list).
//! #3330: app::operation (generate, the 5-Axis Contour arm) and
//! motion::finishing3d (ring_seam) through the application.
//! Gate map (#3331): covers app::operation (generate: the Facing, Profile
//! Rough and Profile Finish picks), app::commands (EditOperationParams).

use camcraft_app::mcp::Server;
use camcraft_app::geo2d::Point2;
use camcraft_app::operation::{Carried, Height, HeightReference, KindInfo, OpParams, OpType, PickCapability};
use camcraft_app::path_picks::PickRole;
use serde_json::{json, Value};

// #3331: the turning kinds' picks, in the lathe (Z, X) plane.
#[path = "start_exit_shared/turning.rs"]
mod turning;

fn call(s: &mut Server, name: &str, args: Value) -> (bool, Value) {
    let msg = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": { "name": name, "arguments": args } });
    let reply = s.handle(&msg).expect("an answer");
    let r = reply["result"].clone();
    (r["isError"] == json!(true), r)
}

fn job() -> Server {
    let mut s = Server::new();
    assert!(!call(&mut s, "set_stock", json!({ "box": { "dx": 60.0, "dy": 40.0, "dz": 20.0 } })).0);
    assert!(!call(&mut s, "add_tool", json!({ "catalog": "EM-1000-4FL" })).0);
    s
}

/// The maintainer's case (#1448, 2026-11-22): a start point on a 2D
/// Offset. It is refused, and the refusal names the kind and says why.
#[test]
fn a_pick_on_a_kind_that_does_not_take_it_names_the_kind_and_the_reason() {
    let mut s = job();
    for (kind, label) in [("2D Offset", "2D Offset"), ("2D Pocket", "2D Pocket"), ("3D Pocket", "3D Pocket")] {
        let (err, r) = call(&mut s, "add_operation", json!({ "kind": kind, "tool": "EM-1000-4FL", "name": "Op" }));
        assert!(!err, "{kind}: {r}");
        for (role, what) in [("start", "start point"), ("exit", "exit point")] {
            let patch = json!({ "name": "Op", "params": { "picks": { role: [[35.0, 0.0]] } } });
            let (err, r) = call(&mut s, "edit_operation", patch);
            let text = r.to_string();
            assert!(err, "{kind}: a {what} is refused: {text}");
            assert!(text.contains(label), "{kind}: the refusal names the kind: {text}");
            assert!(text.contains(what), "{kind}: the refusal names the pick: {text}");
            assert!(text.contains("#1448"), "{kind}: the refusal says where the work is tracked: {text}");
        }
        assert!(!call(&mut s, "delete_operation", json!({ "name": "Op" })).0);
    }
}

#[test]
fn every_kind_has_one_pick_capability_and_only_the_taken_ones_take_a_pick() {
    for k in KindInfo::ALL {
        let cap = k.pick();
        match k.ty {
            OpType::Contour => assert_eq!(cap, PickCapability::Ring, "the contour takes a ring pick"),
            // #3330: the 5-axis contour seats its waterline rings at the pick.
            OpType::Contour5D => assert_eq!(cap, PickCapability::Ring, "the 5-axis contour takes a ring pick"),
            OpType::WireContour => assert_eq!(cap, PickCapability::Threading, "the wire takes its threading point"),
            // #3331 slice 1: a point in the lathe (Z, X) plane.
            OpType::Facing | OpType::ProfileRough | OpType::ProfileFinish => {
                assert_eq!(cap, PickCapability::Meridian, "{} takes a lathe-plane pick", k.label)
            }
            // #3325: the Slot's slice of #1448 shipped.
            OpType::Slot => assert_eq!(cap, PickCapability::Ends, "the slot takes a pick at one of its ends"),
            _ => {
                let PickCapability::NotTaken(why) = cap else { panic!("{}: this kind takes no pick", k.label) };
                assert!(!why.is_empty(), "{}: the reason is named", k.label);
            }
        }
    }
    assert!(PickCapability::Ring.takes(PickRole::Start) && PickCapability::Ring.takes(PickRole::Exit));
    // #3332: the wire takes its exit too.
    assert!(PickCapability::Threading.takes(PickRole::Start) && PickCapability::Threading.takes(PickRole::Exit));
    assert!(PickCapability::Meridian.takes(PickRole::Start) && PickCapability::Meridian.takes(PickRole::Exit));
    let probe = KindInfo::ALL.iter().find(|k| k.ty == OpType::Probe).map(KindInfo::pick);
    assert!(matches!(probe, Some(PickCapability::NotTaken(w)) if w.contains("cuts no path")), "{probe:?}");
}

#[test]
fn path_picks_reads_the_contour_pick_and_the_wire_start() {
    let mut contour = seeded(OpType::Contour);
    assert!(contour.path_picks().is_empty(), "a new contour has no pick");
    if let Some(p) = contour.path_picks_mut() {
        p.start = vec![camcraft_app::path_picks::PathPick::xy(Point2::new(35.0, 0.0))];
        p.exit = vec![camcraft_app::path_picks::PathPick::xy(Point2::new(0.0, 25.0))];
    }
    let picks = contour.path_picks();
    let start = picks.first(PickRole::Start).expect("the start");
    let exit = picks.first(PickRole::Exit).expect("the exit");
    assert_eq!((start.xy.x, start.xy.y, start.z), (35.0, 0.0, None));
    assert_eq!((exit.xy.x, exit.xy.y, exit.z), (0.0, 25.0, None));
    assert!(picks.refusal().is_none());

    let mut wire = seeded(OpType::WireContour);
    if let OpParams::WireContour(w) = &mut wire {
        w.picks.start = vec![camcraft_app::path_picks::PathPick::xy(Point2::new(4.0, -3.0))];
    }
    let picks = wire.path_picks();
    let start = picks.first(PickRole::Start).expect("the wire start");
    assert_eq!((start.xy.x, start.xy.y), (4.0, -3.0));
    assert!(picks.first(PickRole::Exit).is_none());

    // The list form refuses a second pick by name (one pick per operation so far).
    let mut two = picks.clone();
    two.start.push(start);
    assert!(two.refusal().is_some_and(|m| m.contains("one start point")), "{:?}", two.refusal());
}

/// The registry's own seed of `ty`, from neutral carried numbers.
fn seeded(ty: OpType) -> OpParams {
    let h = |r, v| Height::of(r, v);
    let carried = Carried {
        feed: 1000.0,
        plunge: 300.0,
        climb: true,
        clearance: h(HeightReference::TopOfStock, 10.0),
        stepover: 3.0,
        stepdown: 2.0,
        top: h(HeightReference::TopOfStock, 0.0),
        bottom: h(HeightReference::TopOfStock, -4.0),
        retract: h(HeightReference::TopOfStock, 5.0),
        tilt: None,
    };
    let k = KindInfo::ALL.iter().find(|k| k.ty == ty).expect("the kind");
    (k.seed)(&carried)
}

#[test]
fn the_mcp_doors_name_the_pick_fields_and_the_kinds_that_take_them() {
    let mut s = Server::new();
    let reply = s.handle(&json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {} })).expect("an answer");
    let edit = reply["result"]["tools"]
        .as_array()
        .and_then(|t| t.iter().find(|t| t["name"] == "edit_operation"))
        .expect("edit_operation")
        .clone();
    let doc = edit["inputSchema"]["properties"]["params"]["description"].as_str().unwrap_or_default().to_owned();
    for want in ["picks", "2D Contour", "list_operation_types"] {
        assert!(doc.contains(want), "the schema names {want}: {doc}");
    }
    let (err, kinds) = call(&mut s, "list_operation_types", json!({}));
    assert!(!err, "{kinds}");
    let text = kinds.to_string();
    assert!(text.contains("start_exit"), "each kind says if it takes a pick: {text:.400}");
}

// #3330: the 5-Axis Contour takes a start and an exit pick.

/// A spherical cap (radius 12, to 60 degrees off the pole) on z = 0.
fn dome() -> camcraft_app::geometry::Mesh {
    use camcraft_app::geometry::{Mesh, Point3, Triangle};
    const TAU: f64 = std::f64::consts::TAU;
    let (radius, polar_max, n_lat, n_lon) = (12.0, std::f64::consts::FRAC_PI_3, 6usize, 24usize);
    let ring = |k: usize| -> Vec<Point3> {
        let ph = polar_max * k as f64 / n_lat as f64;
        let (z, rr) = (radius * ph.cos(), radius * ph.sin());
        (0..n_lon).map(|j| {
            let th = TAU * j as f64 / n_lon as f64;
            Point3::new(rr * th.cos(), rr * th.sin(), z)
        }).collect()
    };
    let apex = Point3::new(0.0, 0.0, radius);
    let first = ring(1);
    let mut f: Vec<Triangle> = (0..n_lon).map(|j| Triangle::new(apex, first[j], first[(j + 1) % n_lon])).collect();
    for k in 1..n_lat {
        let (a, b) = (ring(k), ring(k + 1));
        for j in 0..n_lon {
            let n = (j + 1) % n_lon;
            f.push(Triangle::new(a[j], b[j], b[n]));
            f.push(Triangle::new(a[j], b[n], a[n]));
        }
    }
    Mesh::from_facets(f)
}

/// The dome on a 40 x 40 x 12 block, with the catalog ball.
fn dome_world() -> flecs_ecs::prelude::World {
    use camcraft_app::commands::{AddTool, History, ImportModel, SetStock};
    use camcraft_app::geometry::{Point3, IDENTITY_Q};
    let world = camcraft_app::world::new_world();
    let mut h = History::default();
    let at = camcraft_app::schema::Location { pos: Point3::ZERO, rot: IDENTITY_Q };
    let stock = SetStock::new(&world, camcraft_app::stock::StockShape::Box { dx: 40.0, dy: 40.0, dz: 12.0 }, at);
    h.execute(&world, stock.expect("stock"));
    let t = camcraft_app::tools::catalog().into_iter().find(|(n, _)| *n == "BN-0600").map(|(_, t)| t).expect("catalog ball");
    h.execute(&world, AddTool::new(&world, "BN-0600", t).expect("tool"));
    h.execute(&world, ImportModel::from_mesh("part", dome()));
    world
}

fn contour5d(start: Option<(f64, f64)>, exit: Option<(f64, f64)>) -> camcraft_app::schema::Operation {
    let pick = |(x, y)| camcraft_app::path_picks::PathPick::xy(Point2::new(x, y));
    let params = OpParams::Contour5D {
        tolerance: 0.1,
        clearance_z: Height::fixed(25.0),
        retract_z: None,
        top_z: Height::fixed(11.0),
        bottom_z: Height::fixed(7.0),
        feed: 500.0,
        plunge_feed: 100.0,
        climb: true,
        lead_deg: 15.0,
        lag_deg: 0.0,
        max_tilt_deg: 60.0,
        shank_avoid: false,
        picks: camcraft_app::path_picks::PathPicks { start: start.map(pick).into_iter().collect(), exit: exit.map(pick).into_iter().collect() },
    };
    camcraft_app::schema::Operation {
        tool: "BN-0600".into(),
        src: camcraft_app::operation::SrcRef::Entity("part".into()),
        params,
        stamp: None,
        setup: None,
        area: Default::default(),
        coolant: Default::default(),
        after: Default::default(),
        suppressed: false,
    }
}

/// Each move's ball centre: its tip backed up 3 mm (the ball's radius)
/// along the pose the machine holds there (the pivot law).
fn centres(t: &camcraft_app::cl::Toolpath) -> Vec<(camcraft_app::cl::MoveKind, [f64; 3], [f64; 3])> {
    let mut held = [0.0, 0.0, 1.0];
    t.moves.iter().map(|m| {
        let from = [m.from.x + 3.0 * held[0], m.from.y + 3.0 * held[1], m.from.z + 3.0 * held[2]];
        held = m.axis.unwrap_or(held);
        (m.kind, from, [m.to.x + 3.0 * held[0], m.to.y + 3.0 * held[1], m.to.z + 3.0 * held[2]])
    }).collect()
}

/// Each ring's (first, last) ball centre: the feed run at one level
/// that a plunge begins. A closed ring returns to its first centre.
fn rings(t: &camcraft_app::cl::Toolpath) -> Vec<([f64; 3], [f64; 3])> {
    use camcraft_app::cl::MoveKind;
    let c = centres(t);
    let mut out = Vec::new();
    let mut k = 0;
    while k < c.len() {
        if c[k].0 != MoveKind::Plunge {
            k += 1;
            continue;
        }
        let first = c[k].2;
        let mut last = first;
        k += 1;
        while k < c.len() && c[k].0 == MoveKind::Feed && (c[k].2[2] - first[2]).abs() < 1e-6 {
            last = c[k].2;
            k += 1;
        }
        out.push((first, last));
    }
    out
}

fn same(a: [f64; 3], b: [f64; 3]) -> bool {
    (0..3).all(|i| (a[i] - b[i]).abs() < 1e-6)
}

#[test]
fn contour5d_starts_at_the_picked_start_and_leaves_at_the_picked_exit() {
    let world = dome_world();
    let generate = |op| camcraft_app::operation::generate(&world, &op, "").map(|(t, _)| t);
    let auto = generate(contour5d(None, None)).expect("the automatic ladder");
    // Start alone: every closed ring begins on the pick's ray (+X).
    let started = generate(contour5d(Some((13.0, 0.0)), None)).expect("the started ladder");
    let closed: Vec<_> = rings(&started).into_iter().filter(|(a, b)| same(*a, *b)).collect();
    assert!(!closed.is_empty(), "the dome ladder has closed rings");
    for (a, _) in &closed {
        assert!(a[1].abs() < 1e-6 && a[0] > 0.0, "a ring starts off the start pick's ray: {a:?}");
    }
    assert_eq!(rings(&started).len(), rings(&auto).len(), "the pick seats the rings, it adds none");
    // Start and exit: each seated ring leaves on the exit pick's ray (+Y).
    let both = generate(contour5d(Some((13.0, 0.0)), Some((0.0, 13.0)))).expect("the picked ladder");
    let seated: Vec<_> = rings(&both).into_iter().filter(|(a, _)| a[1].abs() < 1e-6 && a[0] > 0.0).collect();
    assert_eq!(seated.len(), closed.len(), "every closed ring is seated");
    for (_, b) in &seated {
        assert!(b[0].abs() < 1e-6 && b[1] > 0.0, "a ring leaves off the exit pick's ray: {b:?}");
    }
    // Without a pick the path is the automatic ladder, run for run.
    assert_eq!(generate(contour5d(None, None)).expect("again"), auto);
}

#[test]
fn contour5d_refuses_a_pick_far_from_every_ring_by_name() {
    let world = dome_world();
    for (op, what) in [(contour5d(Some((90.0, 0.0)), None), "start point (90.000, 0.000)"), (contour5d(None, Some((0.0, 90.0))), "exit point (0.000, 90.000)")] {
        let e = camcraft_app::operation::generate(&world, &op, "").err().map(|e| format!("{e:?}"));
        assert!(e.as_deref().is_some_and(|m| m.contains(what) && m.contains("farther than")), "{what}: {e:?}");
    }
}

#[test]
fn contour5d_picks_ride_edit_operation_and_undo_restores_them() {
    let mut s = job();
    assert!(!call(&mut s, "add_tool", json!({ "catalog": "BN-0600" })).0);
    let (err, r) = call(&mut s, "add_operation", json!({ "kind": "5-Axis Contour", "tool": "BN-0600", "name": "C5" }));
    assert!(!err, "{r}");
    let patch = json!({ "name": "C5", "params": { "picks": { "start": [[35.25, 0.75]], "exit": [[0.75, 25.25]] } } });
    let (err, r) = call(&mut s, "edit_operation", patch);
    assert!(!err, "the 5-axis contour takes both picks: {r}");
    let held = |s: &mut Server| call(s, "get_job", json!({})).1.to_string();
    let job_text = held(&mut s);
    assert!(job_text.contains("35.25") && job_text.contains("25.25"), "the job carries the picks: {job_text:.600}");
    assert!(!call(&mut s, "undo", json!({})).0);
    let job_text = held(&mut s);
    assert!(!job_text.contains("35.25") && !job_text.contains("25.25"), "undo restores the automatic seam");
    assert!(!call(&mut s, "redo", json!({})).0);
    assert!(held(&mut s).contains("35.25"), "redo states the picks again");
}
