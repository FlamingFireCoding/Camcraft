//! Issue #1950: the whole-model Adaptive 3D Pocket reaches 2r past the
//! stock side, so its stock ring is cut from the air. Steel that stands
//! in that air band (a clamp, a vise jaw) is in the cutter's reach now:
//! the post gate must name the strike and block the post, never pass it.
//!
//! Gate map (#1950): covers app::operation (generate's Pocket3D arm,
//! open_side), app::fixture_struck, app::posting.

use camcraft_app::commands::{AddTool, CreateOperation, GenerateOperation, History, ImportModel, SetStock, TagFixture};
use camcraft_app::geometry::{cube, Point3};
use camcraft_app::links::Links;
use camcraft_app::machining2d::PocketStrategy;
use camcraft_app::machining3d::Entry3D;
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::posting;
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;
use camcraft_app::world::new_world;
use flecs_ecs::prelude::World;

const TOOL: &str = "EM-1000-4FL";
const R: f64 = 5.0;
/// The part's top and the stock's top: the 6 mm between is the cut.
const TOP: f64 = 20.0;
const STOCK_TOP: f64 = 26.0;
/// The stock's half side: the stock is the part's footprint.
const HALF: f64 = 15.0;

/// A 30 mm block under a 30 mm stock, a 10 mm end mill, and a steel
/// block that stands from the floor to over the stock top, its near face
/// `gap` past the stock's +X side.
fn world_with_steel(gap: f64) -> (World, History) {
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, ImportModel::from_mesh("part", cube(Point3::new(-HALF, -HALF, 0.0), Point3::new(HALF, HALF, TOP))));
    let stock = StockShape::Box { dx: 2.0 * HALF, dy: 2.0 * HALF, dz: STOCK_TOP };
    h.execute(&world, SetStock::new(&world, stock, Location::default()).expect("stock"));
    let tool = catalog().into_iter().find(|(n, _)| *n == TOOL).map(|(_, t)| t).expect("catalog");
    h.execute(&world, AddTool::new(&world, TOOL, tool).expect("tool"));
    let x0 = HALF + gap;
    let steel = cube(Point3::new(x0, -5.0, 0.0), Point3::new(x0 + 3.0, 5.0, STOCK_TOP + 4.0));
    h.execute(&world, ImportModel::from_mesh("clamp", steel));
    h.execute(&world, TagFixture::new(&world, "clamp", None).expect("tag the steel"));
    (world, h)
}

fn pocket3d(strategy: PocketStrategy) -> OpParams {
    OpParams::Pocket3D {
        links: Links::ROUGH3D,
        stepover: 4.0,
        stock_to_leave: 0.0,
        clearance_z: Height::fixed(STOCK_TOP + 20.0),
        retract_z: None,
        top_z: Height::fixed(STOCK_TOP),
        bottom_z: Height::fixed(TOP),
        stepdown: 3.0,
        feed: 1200.0,
        plunge_feed: 300.0,
        climb: true,
        strategy,
        entry: Entry3D::Plunge,
        helix_radius: 2.0,
        rest_rough: false,
        optimal_load: camcraft_app::load::DEFAULT_OPTIMAL_LOAD,
        feed_opt: false,
        shell: false,
    }
}

fn gate_after(world: &World, h: &mut History, strategy: PocketStrategy) -> Vec<String> {
    let src = SrcRef::Entity("part".into());
    h.execute(world, CreateOperation::new(world, "Rough", TOOL, src, pocket3d(strategy)).expect("op"));
    h.execute(world, GenerateOperation::new(world, "Rough").expect("generate"));
    posting::verification_gate(world)
}

/// The steel's near face is r + 1 mm past the stock side. The old band
/// keeps the centre on the stock side, so its disc stops r past it; the
/// Adaptive air band puts the centre up to r past the side (its first
/// front a load width less), so its disc reaches almost 2r past the side
/// and enters the steel. The generator does not route round the steel:
/// the post gate must name the strike, never pass it.
#[test]
fn a_clamp_in_the_air_band_blocks_the_post() {
    let gap = R + 1.0;
    let (world, mut h) = world_with_steel(gap);
    let problems = gate_after(&world, &mut h, PocketStrategy::Adaptive);
    assert!(
        problems.iter().any(|p| p.contains("strikes a fixture")),
        "the Adaptive air band reaches the steel {gap} mm past the stock, so the post is blocked: {problems:?}"
    );

    // The contrast: the old band never reaches that far, so the same
    // steel beside an Offset rings pocket is clear.
    let (world, mut h) = world_with_steel(gap);
    let problems = gate_after(&world, &mut h, PocketStrategy::OffsetRings);
    assert!(
        !problems.iter().any(|p| p.contains("strikes a fixture")),
        "the old band stays clear of the steel: {problems:?}"
    );
}

