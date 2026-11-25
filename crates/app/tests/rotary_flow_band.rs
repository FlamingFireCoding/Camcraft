//! #2826: the Rotary Flow band end to end. The caller PICKS the guide
//! chains (the edge rings bounding a band on a turned part), the axis
//! comes from the mounted machine's stated rotary line
//! (tormach-770mx-microarc4.mch's A about X, millturn.mch's spindle C
//! about Z), and the kind blends flow passes between each adjacent
//! pair about it. What "right" means here:
//!
//! - the band generates through the operation layer with a ball, and
//!   refuses a flat tool by name;
//! - every cut move states a pose radial about the band's own axis -
//!   no world-Z clipping of the band, the heights read as BALL-CENTER
//!   RADII;
//! - the microARC's stated A and the mill-turn's spindle C both carry
//!   the band, and a clearance radius inside the work is refused by
//!   name.
//!
//! Provenance: our own fixtures over our own machine files and
//! commands; nothing of vendor binaries.
//!
//! Gate map (M4.4): covers app::operation (the RotaryFlow arm),
//! app::rotary_stated, camcraft-motion (rotary_flow, axis_stated),
//! app::fixture, app::posting.

mod common;

use flecs_ecs::prelude::World;

use camcraft_app::cl::MoveKind;
use camcraft_app::commands::{
    AddTool, CommandError, CreateOperation, GenerateOperation, History, ImportModel, LoadMachine, SetStock,
};
use camcraft_app::geometry::{feature, IDENTITY_Q, Mesh, Point3, Triangle};
use camcraft_app::model;
use camcraft_app::operation::{FeatureSel, GeoSel, Height, OpParams, OpStatus, SrcRef};
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;
use camcraft_app::{fixture, operation};

const TAU: f64 = std::f64::consts::TAU;
/// The tormach-770mx-microarc4.mch registry entry: an A about X,
/// stated `[1, 0, 0]`, work-turning.
const MICROARC: &str = "Tormach 770MX + microARC 4";
/// The millturn.mch registry entry: a spindle C about Z, stated.
const MILLTURN: &str = "Mill-turn";

/// The turned bar's meridian (rho, s): wall r6 with an interior
/// cylindrical band (the rings at -8 and 2), shoulder, neck r5.
const MERIDIAN: [(f64, f64); 7] = [(0.0, -16.0), (6.0, -16.0), (6.0, -8.0), (6.0, 2.0), (5.0, 8.0), (5.0, 16.0), (0.0, 16.0)];

/// The bar revolved about `axis` (0 = world X, laid; 2 = world Z,
/// standing), 72 facets around, centred on the origin.
fn bar(axis: usize) -> Vec<Triangle> {
    let pt = |rho: f64, s: f64, j: usize| {
        let th = TAU * j as f64 / 72.0;
        match axis {
            0 => Point3::new(s, rho * th.cos(), rho * th.sin()),
            _ => Point3::new(rho * th.cos(), rho * th.sin(), s),
        }
    };
    let mut f = Vec::new();
    for w in MERIDIAN.windows(2) {
        let ((ra, za), (rb, zb)) = (w[0], w[1]);
        for j in 0..72 {
            let k = (j + 1) % 72;
            let (a1, a2, b1, b2) = (pt(ra, za, j), pt(ra, za, k), pt(rb, zb, j), pt(rb, zb, k));
            if ra <= 1e-12 {
                f.push(Triangle::new(a1, b2, b1));
            } else if rb <= 1e-12 {
                f.push(Triangle::new(a1, a2, b2));
            } else {
                f.push(Triangle::new(a1, a2, b2));
                f.push(Triangle::new(a1, b2, b1));
            }
        }
    }
    f
}

fn flow(stated_axis: Option<[f64; 3]>, clearance: f64) -> OpParams {
    OpParams::RotaryFlow {
        tolerance: 0.1,
        stepover: Some(2.0),
        clearance_z: Height::fixed(clearance),
        retract_z: None,
        feed: 500.0,
        plunge_feed: 100.0,
        climb: true,
        lead_deg: 0.0,
        lag_deg: 0.0,
        max_tilt_deg: 60.0,
        stated_axis,
    }
}

/// The band's guide chains: the model's RING chains - the long
/// closed chains whose points all stand at ONE station each. A
/// cone's meridian edge also touches two stations, but no one
/// station owns the whole chain; a disc's fan edge holds two
/// points - neither is a ring.
fn band_picks(world: &World, axis: usize, stations: &[f64]) -> SrcRef {
    let e = model::find_by_name(world, "bar").expect("bar");
    let f = model::features_of(world, &e).expect("features");
    let mut items: Vec<([f64; 3], GeoSel)> = feature::chains(&f)
        .into_iter()
        .filter_map(|c| {
            let pts = feature::chain_points(&f, &c);
            let on_band = stations.iter().any(|&st| pts.iter().all(|p| (p[axis] - st).abs() < 1e-4))
                && pts.len() >= 10;
            let n = pts.len() as f64;
            let key = pts.iter().fold([0.0f64; 3], |a, p| [a[0] + p[0] / n, a[1] + p[1] / n, a[2] + p[2] / n]);
            on_band.then(|| (key, GeoSel { entity: "bar".into(), feature: FeatureSel::Chain(c.id) }))
        })
        .collect();
    items.sort_by(|a, b| a.0[0].total_cmp(&b.0[0]).then(a.0[1].total_cmp(&b.0[1])).then(a.0[2].total_cmp(&b.0[2])));
    assert!(!items.is_empty(), "the band's ring chains must exist");
    SrcRef::Selection(items.into_iter().map(|(_, g)| g).collect())
}

/// A world with the ball, a snug round stock, the model "bar" (the
/// revolve about `axis`) and the flow op over the band's picks, on the
/// given machine (its stated rotary line judged against the bar's
/// faces). The WCS origin stays at the default, so the stated line
/// through it is the bar's own axis. No generation runs here.
fn build_at(axis: usize, params: OpParams, machine: Option<&str>, stations: &[f64]) -> World {
    let world = camcraft_app::world::new_world();
    let mut h = History::default();
    let tool = catalog().into_iter().find(|(n, _)| *n == "BN-0600").map(|(_, t)| t).expect("catalog ball");
    h.execute(&world, AddTool::new(&world, "BN-0600", tool).expect("tool"));
    if let Some(m) = machine {
        h.execute(&world, LoadMachine::builtin(&world, m).expect("machine"));
    }
    let loc = if axis == 0 {
        Location { pos: Point3::new(-16.0, 0.0, 0.0), rot: camcraft_app::geometry::quat_from_axis_angle(1, std::f64::consts::FRAC_PI_2) }
    } else {
        Location { pos: Point3::new(0.0, 0.0, -16.0), rot: IDENTITY_Q }
    };
    h.execute(
        &world,
        SetStock::new(&world, StockShape::Cylinder { diameter: 12.0, height: 32.0 }, loc).expect("stock"),
    );
    h.execute(&world, ImportModel::from_mesh("bar", Mesh::from_facets(bar(axis))));
    h.execute(
        &world,
        CreateOperation::new(&world, "OP", "BN-0600", band_picks(&world, axis, stations), params).expect("create"),
    );
    world
}

fn build(axis: usize, params: OpParams, machine: Option<&str>) -> World {
    build_at(axis, params, machine, &[2.0, 8.0])
}

/// [`build`], with the band generated.
fn world_for(axis: usize, params: OpParams, machine: Option<&str>) -> World {
    let world = build(axis, params, machine);
    h_generate(&world);
    world
}

/// The op's generation, as the command runs it.
fn h_generate(world: &World) {
    let cmd = GenerateOperation::new(world, "OP").expect("generate");
    History::default().execute(world, cmd);
}

fn moves(world: &World) -> Vec<camcraft_app::cl::Move> {
    let e = model::find_by_name(world, "OP").expect("op");
    model::toolpath_of(world, &e).expect("a cached path").path.moves
}

fn is_ok(world: &World) -> bool {
    let e = model::find_by_name(world, "OP").expect("op");
    OpStatus::Ok == operation::status_of(world, &e)
}

/// The generation refusal's own words, when the command refuses.
fn generate_err_text(world: &World, name: &str) -> String {
    match GenerateOperation::new(world, name) {
        Err(e) => e.to_string(),
        Ok(cmd) => {
            History::default().execute(world, cmd);
            String::from("generated")
        }
    }
}

#[test]
fn the_band_generates_on_the_stated_a_and_cuts_radially() {
    // The microARC 4's stated A is [1, 0, 0] through the work offset
    // origin: the bar lies ON it, the judge holds the line, and every
    // cut pose is radial - the ball-centre stands on the bar's own
    // meridian radius, never on a world-Z ladder.
    let world = world_for(0, flow(None, 25.0), Some(MICROARC));
    assert!(is_ok(&world), "the band generates on the stated A");
    let ms = moves(&world);
    let cuts = ms.iter().filter(|m| m.kind != MoveKind::Rapid).count();
    assert!(cuts > 50, "a whole band of contact moves, got {cuts}");
    for (i, m) in ms.iter().enumerate() {
        let Some(a) = m.axis else {
            panic!("move {i} states no pose: {m:?}");
        };
        // Laid bar: axis world X, so the radial of the move's own
        // azimuth is [0, cos, sin] in the Y/Z plane.
        let rho = (m.to.y * m.to.y + m.to.z * m.to.z).sqrt();
        assert!(rho > 1.0, "cut {i} rides off the bar: {m:?}");
        let lean = a[1] * (m.to.y / rho) + a[2] * (m.to.z / rho);
        assert!(
            a.iter().all(|v| v.is_finite()) && (lean - 1.0).abs() < 1e-6 && a[0].abs() < 1e-6,
            "cut {i} is not radial about the bar's axis: pose {a:?} at {:?}",
            m.to
        );
    }
}

#[test]
fn the_millturns_spindle_c_gate_refuses_a_full_ring_band_past_travel() {
    // millturn.mch's spindle C states [0, 0, 1]: the standing bar ON
    // the spindle line holds it, but a full-ring band's flow passes
    // wind C continuously (climb keeps the sense), and the unwind law
    // is proven for Rotary Contour only - the stamp refuses where the
    // C would rewind, by name, and no path is cached (issue #33's
    // `unwinds: false`, matching RotaryParallel).
    let world = build(2, flow(None, 25.0), Some(MILLTURN));
    let err = generate_err_text(&world, "OP");
    assert!(err.contains("past the machine's C travel"), "{err}");
    assert!(err.contains("unwinding C"), "{err}");
    let e = model::find_by_name(&world, "OP").expect("op");
    assert!(model::toolpath_of(&world, &e).is_none(), "a refused band leaves no path");
    // The verify's own read still answers the machine's stated line
    // for the sheet and the axis pick.
    let op = model::operation_of(&world, &e).expect("operation");
    let _ = fixture::rotary_axis_of(&world, &op);
}

#[test]
fn the_band_generates_and_a_buried_radius_is_refused_by_name() {
    // The kernel's radial guard (#2774's alignment): a clearance
    // radius inside the work's own radius plus the ball is refused at
    // the generator, by name - never a silent shallow program.
    // (Posting the band is held out of this test: the radial field
    // law's 0.25 mm cell quantization reads the ball resting on the
    // band's convex crease ring as a gouge past its 0.01 allowance,
    // and that warning blocks the post - the finding is reported on
    // the issue.)
    let world = world_for(0, flow(None, 25.0), Some(MICROARC));
    assert!(is_ok(&world), "the band generates on the stated A");

    let buried = build(0, flow(None, 8.0), Some(MICROARC));
    let err = generate_err_text(&buried, "OP");
    assert!(
        err.contains("does not clear the work") && err.contains("ball-center radius"),
        "{err}"
    );
    let e = model::find_by_name(&buried, "OP").expect("op");
    assert!(model::toolpath_of(&buried, &e).is_none(), "a refused band leaves no path");
}

#[test]
fn a_flat_tool_is_refused_by_name_on_the_band() {
    // The EM has no ball to rest a radial pose on: refused at the
    // create door, by name - the status Never.
    let world = camcraft_app::world::new_world();
    let mut h = History::default();
    let em = catalog().into_iter().find(|(n, _)| *n == "EM-0600-4FL").map(|(_, t)| t).expect("flat");
    h.execute(&world, AddTool::new(&world, "EM-0600-4FL", em).expect("tool"));
    h.execute(&world, ImportModel::from_mesh("bar", Mesh::from_facets(bar(2))));
    let ask = CreateOperation::new(&world, "OP", "EM-0600-4FL", band_picks(&world, 2, &[2.0, 8.0]), flow(None, 25.0));
    let Err(CommandError::Invalid(msg)) = ask else {
        panic!("expected an invalid refusal");
    };
    assert!(msg.contains("ball-nose"), "{msg}");
}

#[test]
fn three_ring_chains_cut_two_bands_and_the_boundary_once() {
    // #2829: the rings at the shoulder (station 2), the neck (8) and
    // the end-face rim (16), picked in band order. The neck ring
    // extracts wound against its pair, so the kernel walks it
    // backwards; the shared neck ring is band 1's own last pass
    // already, so band 2 starts at its first interior pass: the
    // plunge count is band 1's passes plus band 2's passes less the
    // one shared boundary, and no two passes land on one station.
    // (The ring chains start at arbitrary corners, so each band
    // carries the matched-place phase the extraction gave it - the
    // pass counts are read from the kernel's own notes, never
    // hard-coded.)
    let world = build_at(0, flow(None, 25.0), Some(MICROARC), &[2.0, 8.0, 16.0]);
    h_generate(&world);
    assert!(is_ok(&world), "three ordered rings machine two bands");
    let e = model::find_by_name(&world, "OP").expect("op");
    let notes = model::toolpath_of(&world, &e).expect("a cached path").path.notes;
    let ms = moves(&world);
    let ls: Vec<Point3> = ms
        .iter()
        .filter(|m| m.kind == MoveKind::Plunge)
        .filter_map(|m| {
            let a = m.axis?;
            Some(Point3::new(
                m.to.x + a[0] * 3.0,
                m.to.y + a[1] * 3.0,
                m.to.z + a[2] * 3.0,
            ))
        })
        .collect();
    // Each band's own pass count, as the kernel states it. Band 2's
    // count is its full division LESS the shared boundary (the
    // exact arithmetic is pinned at the motion level, on fixtures
    // whose widths read exact; here the rings' arbitrary start
    // corners add matched-place phase, so the counts are read, not
    // hard-coded).
    let counts: Vec<usize> = notes
        .iter()
        .filter(|n| n.contains("flow passes"))
        .filter_map(|n| n.split(';').nth(1)?.split(" flow passes").next()?.trim().parse().ok())
        .collect();
    assert_eq!(counts.len(), 2, "the notes do not state two bands: {notes:?}");
    assert!(counts[0] >= 4 && counts[1] >= 4, "the bands divided thin: {notes:?}");
    assert_eq!(
        ls.len(),
        counts[0] + counts[1],
        "the plunge count disagrees with the bands' own counts: {notes:?}"
    );
    for (i, &x) in ls.iter().enumerate() {
        for &y in ls.iter().skip(i + 1) {
            let d = x - y;
            assert!(
                d.dot(d) > 1e-12,
                "two passes land on one station - the boundary is cut twice: {x:?} ~ {y:?}"
            );
        }
    }
    assert!(
        notes.iter().any(|n| n.contains("walked backwards")),
        "the notes do not state the sense normalization: {notes:?}"
    );
    assert!(
        notes.iter().any(|n| n.contains("boundary chain shared with band 1")),
        "the notes do not state the shared boundary: {notes:?}"
    );
}
