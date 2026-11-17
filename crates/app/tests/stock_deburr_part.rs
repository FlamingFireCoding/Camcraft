//! #2313: a Deburr that takes the STOCK as its source breaks the stock's
//! edges, not a part's. Before this fix its bevel kept the edge-break
//! credit (#303) over every part's columns, and it had no stock-source
//! part law, so a cone that rode `X35. Y-35. Z19.` through a part posted.
//! Now its cut into a part is a gouge: at generation (the solid law
//! against the setup's parts), and for a part added after generation the
//! post refuses by name - the op and the part - also for a cut too shallow
//! for the sim's lattice (0.05 mm), on aligned stock and on stock turned a
//! quarter about Z. A Deburr of a part's own edge still posts: on a part
//! source, and off a from-model stock that IS the part. A stock-source
//! Deburr clear of every part still posts.
//!
//! Provenance: our own plate fixture (`stock_source_holes_support`) over
//! the solid law and the column sim; no vendor material consulted.
//!
//! Gate map: covers app::stock_sim_breaks (breaks_stock_edge,
//! stock_part_of), app::operation::keep_law (is_stock_kept_kind,
//! parts_in_stock), app::operation::kept_rejudge, app::posting_sim,
//! app::posting.

mod common;
mod pocket_support;
mod stock_source_holes_support;

use camcraft_app::geometry::quat_from_yaw;
use stock_source_holes_support::*;

/// Aligned stock, and stock turned a quarter about Z: both judged by the
/// program sim (#2177).
const TURNS: [(&str, f64); 2] = [("aligned", 0.0), ("quarter", std::f64::consts::FRAC_PI_2)];

/// A Deburr with the 90-degree cone: a 1 mm break, the tip 1 mm under
/// the stock's top rim (z 19).
fn deburr() -> OpParams {
    OpParams::Deburr {
        width: 1.0,
        clearance_z: Height::fixed(30.0),
        top_z: Height::fixed(TOP - 1.0),
        bottom_z: Height::fixed(TOP - 1.5),
        feed: 600.0,
        plunge_feed: 200.0,
    }
}

/// The plate "part" in the roomy `2 HALF` square box stock turned by
/// `yaw` about Z.
fn plate(yaw: f64) -> (World, History) {
    let (world, mut h) = job(Model::hole(3.0, 8.0), HALF, false);
    if yaw != 0.0 {
        let at = Location { pos: Point3::new(0.0, 0.0, 0.0), rot: quat_from_yaw(yaw) };
        h.execute(&world, SetStock::new(&world, StockShape::Box { dx: 2.0 * HALF, dy: 2.0 * HALF, dz: TOP }, at).expect("stock"));
    }
    (world, h)
}

/// Add the plate "other" over `x0..x1`, `y -5..5`, `z 0..TOP`.
fn add_other(world: &World, h: &mut History, x0: f64, x1: f64) {
    let mut f = Vec::new();
    common::solid_part::plate(&common::solid_part::rect(x0, -5.0, x1, 5.0), None, 0.0, TOP, &mut f);
    h.execute(world, ImportModel::from_mesh("other", Mesh::from_facets(f)));
}

/// The stock-source "S1" Deburr generated clean round the stock's edges.
fn clean_deburr(yaw: f64) -> (World, History) {
    let (world, mut h) = plate(yaw);
    let warns = generated(&world, &mut h, CHAMFER, deburr());
    let blocking: Vec<_> = warns.iter().filter(|w| w.blocks_post()).collect();
    assert!(blocking.is_empty(), "yaw {yaw}: a clean generation: {blocking:?}");
    (world, h)
}

#[test]
fn a_stock_source_deburr_into_a_part_known_at_generation_never_posts() {
    // The review's probe: "other" reaches the stock's edge at x 35, so
    // the cone breaks the top edge of "other" by about 1 mm.
    for (label, yaw) in TURNS {
        let (world, mut h) = plate(yaw);
        add_other(&world, &mut h, 30.0, 35.0);
        let warns = generated(&world, &mut h, CHAMFER, deburr());
        // The op's own verdict: a blocking gouge on S1 (before #2313 it
        // had none, and the credited bevel hid the cut from the sim).
        assert!(gouges(&warns) > 0, "{label}: the solid law finds the cut into 'other': {warns:?}");
        assert_not_posted(&world, &format!("deburr-known-{label}"), "gouge: program cuts below the model");
        // With the part known, the post's line is the sim's located
        // gouge; the live re-judge names only a part added later.
    }
}

#[test]
fn a_part_added_under_a_stock_source_deburr_is_refused_by_name() {
    // 1 mm (the edge at x 35) and 0.05 mm (x 34.05, past the cone's
    // reach of x 34 at the top: under the sim's lattice).
    for (label, yaw) in TURNS {
        for (cut, x1) in [("1mm", 35.0), ("0.05mm", 34.05)] {
            let (world, mut h) = clean_deburr(yaw);
            assert_posts(&world, &format!("deburr-added-{label}-{cut}-before"));
            add_other(&world, &mut h, 30.0, x1);
            assert_not_posted(&world, &format!("deburr-added-{label}-{cut}"), "S1 now cuts part 'other'");
        }
    }
}

#[test]
fn a_stock_source_deburr_clear_of_every_part_still_posts() {
    // "other" stands 10 mm inside the stock's edge: the 1 mm break
    // never reaches it.
    for (label, yaw) in TURNS {
        let (world, mut h) = clean_deburr(yaw);
        add_other(&world, &mut h, 18.0, 24.0);
        assert_posts(&world, &format!("deburr-clear-{label}"));
    }
}

/// A plain 30 x 30 x 20 plate "part" (no hole: a hole's rim is a second
/// ride the links cross) with the stock made from it (the stock IS the
/// part), and an S1 Deburr off `src` generated clean round its edges.
fn from_model(src: SrcRef) -> World {
    let (world, mut h) = job_without_part(HALF);
    let mut f = Vec::new();
    common::solid_part::plate(&common::solid_part::rect(-15.0, -15.0, 15.0, 15.0), None, 0.0, TOP, &mut f);
    h.execute(&world, ImportModel::from_mesh("part", Mesh::from_facets(f)));
    h.execute(&world, SetStock::new(&world, StockShape::FromModel { source: "part".into() }, Location::default()).expect("stock"));
    h.execute(&world, CreateOperation::new(&world, "S1", CHAMFER, src, deburr()).expect("create"));
    h.execute(&world, GenerateOperation::new(&world, "S1").expect("generate"));
    let e = model::find_by_name(&world, "S1").expect("op");
    let tp = model::toolpath_of(&world, &e).expect("cached path");
    assert!(!tp.path.moves.is_empty(), "the cut was generated");
    let blocking: Vec<_> = tp.warnings.iter().filter(|w| w.blocks_post()).collect();
    assert!(blocking.is_empty(), "a clean generation: {blocking:?}");
    world
}

#[test]
fn a_deburr_of_the_parts_own_edge_still_posts() {
    assert_posts(&from_model(SrcRef::Entity("part".into())), "deburr-part-source");
    // The stock IS the part: the edges it breaks are the part's own.
    assert_posts(&from_model(SrcRef::Stock), "deburr-from-model-stock");
}
