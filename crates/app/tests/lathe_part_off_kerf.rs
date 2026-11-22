//! #3452: the issue's case through the document and the lathe post. A
//! Ø30 pin Z-70..0 with a 1 x 45° back chamfer, a Ø34 bar run on to
//! Z-84, a Part op at the bottom of the model (Z-70) with the 3 mm
//! GT-0300 blade on orientation 2 (the -Z corner). The kerf stands
//! wholly behind the part's end face, Z-73..-70: every posted feed is
//! at Z-73 (before the fix: Z-71.5, a kerf Z-71.5..-68.5 that took
//! 1.5 mm and the chamfer off the part), and the part keeps its 70 mm.
//! The +Z corner (orientation 1) posts the face itself, Z-70.
//!
//! Gate map (M4.4): covers app::operation (the Part arm and its kerf
//! verdict), app::posting (the blade-corner shift), app::commands.
//! Lane shape: seconds.

use flecs_ecs::prelude::World;

use camcraft_app::cl::MoveKind;
use camcraft_app::commands::{
    AddTool, ChangeTool, CreateOperation, GenerateOperation, History, ImportModel, LoadMachine, SetStock,
};
use camcraft_app::geometry::{Mesh, Point3, Triangle};
use camcraft_app::machine::{Coolant, Machine};
use camcraft_app::model;
use camcraft_app::operation::{self, Height, OpParams, OpStatus, SrcRef};
use camcraft_app::post::{cnc, engine};
use camcraft_app::posting;
use camcraft_app::schema::Location;
use camcraft_app::tools::catalog;
use camcraft_app::world::new_world;

const TAU: f64 = std::f64::consts::TAU;
const LATHE_CPS: &str = include_str!("../../../assets/posts/fanuc-lathe.cps");
const GT: &str = "GT-0300";
const W: f64 = 3.0;
/// The issue's pin: Ø30, Z-70..0, a 1 x 45° chamfer at the back face.
const PIN: [(f64, f64); 5] = [(0.0, -70.0), (14.0, -70.0), (15.0, -69.0), (15.0, 0.0), (0.0, 0.0)];
const END_FACE: f64 = -70.0;

fn revolve(meridian: &[(f64, f64)], segs: usize) -> Mesh {
    let v = |j: usize, r: f64, z: f64| {
        let th = TAU * j as f64 / segs as f64;
        Point3::new(r * th.cos(), r * th.sin(), z)
    };
    let mut facets = Vec::new();
    for seg in meridian.windows(2) {
        let ((ra, za), (rb, zb)) = (seg[0], seg[1]);
        for j in 0..segs {
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

fn pin_world(corner: Option<u8>) -> (World, History) {
    let world = new_world();
    let mut history = History::default();
    let at = Location { pos: Point3::new(0.0, 0.0, -84.0), ..Location::default() };
    let cmd = SetStock::new(&world, camcraft_app::stock::StockShape::Cylinder { diameter: 34.0, height: 84.0 }, at);
    history.execute(&world, cmd.expect("bar stock"));
    let lathe = Machine {
        model: "TestBench lathe".into(),
        kind: Some("lathe".into()),
        spindle_max: Some(12000.0),
        coolants: vec![Coolant { name: "Flood".into(), m_code: "M8".into() }],
        ..Default::default()
    };
    history.execute(&world, LoadMachine::new(&world, "lathe.mch".into(), Some(lathe)).expect("lathe"));
    let mut blade = catalog().into_iter().find(|(n, _)| *n == GT).map(|(_, t)| t).expect("catalog blade");
    assert!((blade.width - W).abs() < 1e-12, "the fixture's blade is 3 mm wide");
    history.execute(&world, AddTool::new(&world, GT, blade.clone()).expect("blade"));
    blade.orientation = corner;
    history.execute(&world, ChangeTool::new(&world, GT, blade).expect("a sane corner"));
    history.execute(&world, ImportModel::from_mesh("pin", revolve(&PIN, 32)));
    let part = OpParams::Part {
        tolerance: 0.05,
        clearance_z: Height::fixed(10.0),
        top_z: Height::fixed(0.0),
        bottom_z: Height::fixed(END_FACE),
        stepdown: 2.0,
        feed: 120.0,
        plunge_feed: 60.0,
        bar_load: false,
        bar_advance: 0.0,
        bar_unload: false,
        bar_transfer: false,
        inspect_tol: 0.0,
    };
    let cmd = CreateOperation::new(&world, "P1", GT, SrcRef::Entity("pin".into()), part).expect("create");
    history.execute(&world, cmd);
    history.execute(&world, GenerateOperation::new(&world, "P1").expect("generate"));
    let e = model::find_by_name(&world, "P1").expect("op");
    assert_eq!(operation::status_of(&world, &e), OpStatus::Ok, "the part-off generates");
    (world, history)
}

/// The posted feed moves' Z (the corner the blade was touched off on).
fn posted_feed_z(world: &World) -> Vec<f64> {
    posting::stream_records(world)
        .expect("posts")
        .into_iter()
        .filter_map(|r| match r {
            cnc::Record::Move(m) if m.kind != MoveKind::Rapid => Some(m.z),
            _ => None,
        })
        .collect()
}

#[test]
fn a_part_off_at_the_end_face_keeps_the_whole_part() {
    // Orientation 2: the posted Z is the -Z corner, the kerf runs W
    // up from it.
    let (world, _h) = pin_world(Some(2));
    let zs = posted_feed_z(&world);
    assert!(!zs.is_empty(), "the part-off posts feeds");
    for z in &zs {
        let (lo, hi) = (*z, *z + W);
        assert!(hi <= END_FACE + 1e-6, "the kerf Z{lo}..Z{hi} enters the part past its end face Z{END_FACE}");
        assert!((lo + 73.0).abs() < 1e-6, "the -Z corner posts at Z{lo}, wanted Z-73");
    }
    let kept = 0.0 - zs.iter().map(|z| z + W).fold(f64::NEG_INFINITY, f64::max);
    assert!((kept - 70.0).abs() < 1e-6, "the part comes off {kept} mm long, wanted the model's 70");
    let records = posting::stream_records(&world).expect("posts");
    let text = engine::run(LATHE_CPS, &posting::machine_value(&world), &cnc::serialize(&records))
        .expect("the lathe script runs")
        .program;
    assert!(text.contains("X0. Z-73. "), "the blade reaches the axis at Z-73: {text}");
    assert!(!text.contains("Z-71.5 "), "never the old centred kerf: {text}");
}

#[test]
fn the_plus_z_corner_posts_the_end_face_itself() {
    // Orientation 1 (+Z corner): the same kerf, written at its +Z
    // flank - on the face, never past it.
    let (world, _h) = pin_world(Some(1));
    let zs = posted_feed_z(&world);
    assert!(!zs.is_empty());
    assert!(zs.iter().all(|z| (z - END_FACE).abs() < 1e-6), "the +Z corner posts Z-70: {zs:?}");
}
