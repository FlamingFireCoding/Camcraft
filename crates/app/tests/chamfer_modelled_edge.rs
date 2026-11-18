//! #2435: a 2D Chamfer with no picks on a model whose edge already
//! carries a modelled chamfer. The cone's tip rides the WALL at the
//! corner depth (the section there), never the bevel's top edge inside
//! it; the width moves the tip along the bevel in X/Y as well as Z; and
//! a cone whose tip stands inside the part - the old ride - is a
//! blocking gouge at the post gate, not edge-break credit.
//!
//! Gate map (#2435): covers app::operation (material_source, the
//! Chamfer2D region), motion::machining2d (chamfer_ride), app::stock_sim
//! (breaks::program_breaks), sim::stock_sim (edge_break::on_walls),
//! app::posting (verification_gate).

use camcraft_app::cl::MoveKind;
use camcraft_app::commands::{AddTool, CreateOperation, GenerateOperation, History, ImportModel, SetStock};
use camcraft_app::geometry::{Mesh, Point3, Triangle};
use camcraft_app::model;
use camcraft_app::operation::{self, Height, HeightReference, OpParams, OpType, SrcRef};
use camcraft_app::posting;
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;
use camcraft_app::world::new_world;
use flecs_ecs::prelude::World;

/// The 90-degree chamfer mill (D6) the #303 gate rides.
const CONE: &str = "CM-0600-90";

/// A 40 x 20 x 10 brick centred on the origin in plan, z 0..10, whose
/// four top edges carry a modelled `bevel` x 45-degree chamfer: the
/// walls stand at x = +-20, y = +-10 up to z = 10 - bevel, the top face
/// is inset by `bevel`. Outward-facing CCW triangles.
fn bevelled_brick(bevel: f64) -> Mesh {
    let ring = |inset: f64, z: f64| {
        let (x, y) = (20.0 - inset, 10.0 - inset);
        [Point3::new(-x, -y, z), Point3::new(x, -y, z), Point3::new(x, y, z), Point3::new(-x, y, z)]
    };
    let (b, m, t) = (ring(0.0, 0.0), ring(0.0, 10.0 - bevel), ring(bevel, 10.0));
    let mut facets = Vec::new();
    let mut quad = |a: Point3, b: Point3, c: Point3, d: Point3| {
        facets.push(Triangle::new(a, b, c));
        facets.push(Triangle::new(a, c, d));
    };
    quad(b[3], b[2], b[1], b[0]);
    quad(t[0], t[1], t[2], t[3]);
    for i in 0..4 {
        let j = (i + 1) % 4;
        quad(b[i], b[j], m[j], m[i]);
        quad(m[i], m[j], t[j], t[i]);
    }
    Mesh::from_facets(facets)
}

fn chamfer(width: f64) -> OpParams {
    let base = OpParams::Face {
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
    };
    let mut p = operation::retype_params(&base, OpType::Chamfer2D.ordinal());
    if let OpParams::Chamfer2D { width: w, .. } = &mut p {
        *w = width;
    }
    p
}

/// The finished part (the stock IS the bevelled brick) and one no-pick
/// 2D Chamfer of `width` on it.
fn job(width: f64) -> World {
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, ImportModel::from_mesh("brick", bevelled_brick(1.0)));
    let stock = SetStock::new(&world, StockShape::FromModel { source: "brick".into() }, Default::default());
    h.execute(&world, stock.expect("stock"));
    let tool = catalog().into_iter().find(|(n, _)| *n == CONE).map(|(_, t)| t).expect("catalog cone");
    h.execute(&world, AddTool::new(&world, CONE, tool).expect("tool"));
    let op = CreateOperation::new(&world, "OP", CONE, SrcRef::Entity("brick".into()), chamfer(width));
    h.execute(&world, op.expect("create"));
    h.execute(&world, GenerateOperation::new(&world, "OP").expect("generate"));
    world
}

/// Every feed move's end point: the cone's tip along its ride.
fn tips(world: &World) -> Vec<Point3> {
    let e = model::find_by_name(world, "OP").expect("op");
    let t = model::toolpath_of(world, &e).expect("generated");
    assert!(t.warnings.is_empty(), "{:?}", t.warnings);
    t.path.moves.iter().filter(|m| m.kind == MoveKind::Feed).map(|m| m.to).collect()
}

/// Whether `p` lies on the axis-aligned rectangle +-hx by +-hy in plan.
fn on_rect(p: Point3, hx: f64, hy: f64) -> bool {
    let near = |a: f64, b: f64| (a - b).abs() <= 1e-6;
    let (x, y) = (p.x.abs(), p.y.abs());
    (near(x, hx) && y <= hy + 1e-6) || (near(y, hy) && x <= hx + 1e-6)
}

fn gouge_line(world: &World) -> Option<String> {
    posting::verification_gate(world).into_iter().find(|l| l.starts_with("gouge:"))
}

#[test]
fn a_no_pick_chamfer_on_a_modelled_chamfer_rides_the_wall() {
    // The issue's numbers: a 1 mm chamfer on a 1 x 45 bevel. The tip
    // stands on the wall (x = +-20, y = +-10) at the bevel's foot, z 9
    // - not on the bevel's top edge (+-19, +-9), 1 mm inside the wall.
    let world = job(1.0);
    let tips = tips(&world);
    assert!(!tips.is_empty(), "the chamfer rides");
    for p in &tips {
        assert!((p.z - 9.0).abs() <= 1e-6, "tip at the corner depth: {p:?}");
        assert!(on_rect(*p, 20.0, 10.0), "tip on the wall, not inside it: {p:?}");
    }
    assert_eq!(gouge_line(&world), None, "the modelled chamfer's own ride posts");
}

#[test]
fn the_width_moves_the_tip_along_the_bevel_not_only_in_z() {
    // Half the bevel: the cone's face IS the modelled bevel, its tip on
    // the bevel at z 9.5 - half a millimetre out from the top edge.
    let world = job(0.5);
    let tips = tips(&world);
    assert!(!tips.is_empty(), "the chamfer rides");
    for p in &tips {
        assert!((p.z - 9.5).abs() <= 1e-6, "tip at the corner depth: {p:?}");
        assert!(on_rect(*p, 19.5, 9.5), "tip on the bevel at that depth: {p:?}");
    }
    assert_eq!(gouge_line(&world), None, "a cone on the modelled bevel cuts nothing");
}

#[test]
fn a_cone_tip_inside_the_part_is_a_gouge_not_edge_break_credit() {
    // The pre-#2435 ride, put back by hand: the tip and the declared
    // edge on the bevel's top edge (+-19, +-9), 1 mm inside the wall at
    // z 9. The cone cuts a V 0.7 mm into the bevel face; its record
    // declares an edge that is no wall, and must forgive none of it.
    let world = job(1.0);
    let inward = |p: Point3| Point3::new(p.x.clamp(-19.0, 19.0), p.y.clamp(-9.0, 9.0), p.z);
    let e = model::find_by_name(&world, "OP").expect("op");
    let mut t = model::toolpath_of(&world, &e).expect("cached");
    for m in &mut t.path.moves {
        m.from = inward(m.from);
        m.to = inward(m.to);
    }
    assert!(!t.path.breaks.is_empty(), "the chamfer declares its loop");
    for b in &mut t.path.breaks {
        for q in &mut b.edge {
            *q = inward(*q);
        }
    }
    e.entity_view(&world).set(camcraft_app::schema::Toolpath::new(t.path, t.warnings));
    let line = gouge_line(&world).expect("a cone tip inside the part posted");
    assert!(line.contains("worst 0.") || line.contains("worst 1."), "charged at the V's depth: {line}");
}
