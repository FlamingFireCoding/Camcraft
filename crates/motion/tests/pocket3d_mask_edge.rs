//! Issue #1749 (reopened): a 3D Pocket that runs after an op which
//! already cleared the stock past the part's open edge (an outside
//! profile) still clears its notch out to that edge. The re-machining
//! mask's boundary is air that the earlier op cut, not a wall: before
//! the fix the ladder eroded the MASKED target by the radius, so the
//! cutter stayed a radius off the mask boundary and a band one radius
//! wide stood along the open edge at the full stock height. That is
//! the ridge that held a user's soft jaws out of Setup 2.
//!
//! Provenance: our own design (the M3.1 D mask law, read with the
//! "tool centre on boundary" containment of public CAM documentation);
//! no vendor material consulted.
//!
//! Gate map (#1749): covers camcraft_motion::machining3d (ladder, open_side).

use camcraft_motion::cl::{MoveKind, Toolpath};
use camcraft_motion::geo2d::Region;
use camcraft_motion::geometry::{Point3, Triangle};
use camcraft_motion::machining2d::PocketStrategy;
use camcraft_motion::machining3d::{open_sides, pocket3d_offset_prev, pocket3d_prev, Entry3D, Pocket3DParams};
use camcraft_motion::stock_sim::{Columns, Dexel};

/// The cutter radius.
const R: f64 = 4.0;
/// The notch floor (the slab's top) and the wall's top.
const FLOOR: f64 = -10.0;
const TOP: f64 = 0.0;
/// The stock top the mask stands at where nothing has cut yet.
const STOCK_TOP: f64 = 1.0;
/// The part's half width: the silhouette is [-20, 20]^2.
const H: f64 = 20.0;

fn p(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

/// A solid axis box as twelve outward triangles.
fn box_facets(min: [f64; 3], max: [f64; 3]) -> Vec<Triangle> {
    let v = |i: usize| p([min[0], max[0]][i & 1], [min[1], max[1]][(i >> 1) & 1], [min[2], max[2]][(i >> 2) & 1]);
    let quad = |a: usize, b: usize, c: usize, d: usize| [Triangle::new(v(a), v(b), v(c)), Triangle::new(v(a), v(c), v(d))];
    let mut t = Vec::with_capacity(12);
    for q in [quad(4, 5, 7, 6), quad(0, 2, 3, 1), quad(0, 1, 5, 4), quad(2, 6, 7, 3), quad(1, 3, 7, 5), quad(2, 0, 4, 6)] {
        t.extend(q);
    }
    t
}

/// A slab to the floor, and a wall over its -X half up to TOP: the +X
/// half is a notch open on its +X, +Y and -Y sides. Its inner wall at
/// X 0 meets the open +Y edge at (0, 20).
fn notched_part() -> Vec<Triangle> {
    let mut f = box_facets([-H, -H, -20.0], [H, H, FLOOR]);
    f.extend(box_facets([-H, -H, FLOOR], [0.0, H, TOP]));
    f
}

/// The whole-model footprint the app hands the ladder: the silhouette
/// grown past its open sides (`open_sides`, the #1749 law).
fn footprint(f: &[Triangle]) -> Region {
    let silhouette = camcraft_motion::geometry::slice::project(f, 64).expect("silhouette");
    open_sides(&silhouette, R, &Region::empty(), None).expect("open sides")
}

/// What the op ahead left: an outside profile cleared every column
/// outside the silhouette to the floor; inside it, the stock stands.
fn mask() -> Dexel {
    let mut d = Dexel::block(-30.0, -30.0, 30.0, 30.0, STOCK_TOP, 0.5).expect("dexel");
    for iy in 0..d.ny {
        for ix in 0..d.nx {
            let x = d.x0 + (ix as f64 + 0.5) * d.cell;
            let y = d.y0 + (iy as f64 + 0.5) * d.cell;
            if x.abs() > H || y.abs() > H {
                d.set(iy * d.nx + ix, FLOOR);
            }
        }
    }
    d
}

fn params() -> Pocket3DParams {
    Pocket3DParams {
        links: camcraft_motion::links::Links::ROUGH3D,
        tool_diameter: 2.0 * R,
        stepover: 3.0,
        stock_to_leave: 0.0,
        clearance_z: 10.0,
        top_z: STOCK_TOP,
        bottom_z: FLOOR,
        stepdown: 2.5,
        feed: 500.0,
        plunge_feed: 100.0,
        climb: true,
        strategy: PocketStrategy::OffsetRings,
        retract_z: None,
        entry: Entry3D::Plunge,
        helix_radius: 0.0,
        optimal_load: camcraft_motion::load::DEFAULT_OPTIMAL_LOAD,
    }
}

/// Every feed's tip, sampled at most 0.25 mm apart.
fn feed_samples(t: &Toolpath) -> Vec<Point3> {
    let mut out = Vec::new();
    for m in t.moves.iter().filter(|m| m.kind != MoveKind::Rapid) {
        let len = ((m.to.x - m.from.x).powi(2) + (m.to.y - m.from.y).powi(2) + (m.to.z - m.from.z).powi(2)).sqrt();
        let n = ((len / 0.25).ceil() as usize).clamp(1, 4000);
        for k in 0..=n {
            let s = k as f64 / n as f64;
            let q = m.arc.and_then(|a| a.point_at(m.from, m.to, s)).unwrap_or_else(|| {
                p(m.from.x + (m.to.x - m.from.x) * s, m.from.y + (m.to.y - m.from.y) * s, m.from.z + (m.to.z - m.from.z) * s)
            });
            out.push(q);
        }
    }
    out
}

/// The stock the path leaves over (x, y): the lowest tip whose flat-end
/// disc covers the column, or the stock top where none does.
fn carried(s: &[Point3], x: f64, y: f64) -> f64 {
    s.iter().filter(|q| (q.x - x).hypot(q.y - y) <= R - 1e-6).map(|q| q.z).fold(STOCK_TOP, f64::min)
}

/// No tip goes under the floor, and no disc enters the wall below its top.
fn assert_no_gouge(s: &[Point3]) {
    for q in s {
        assert!(q.z >= FLOOR - 1e-6, "tip at ({:.3}, {:.3}, {:.3}) is under the floor", q.x, q.y, q.z);
        if q.z < TOP - 1e-6 {
            let dx = (-H - q.x).max(q.x).max(0.0);
            let dy = (-H - q.y).max(q.y - H).max(0.0);
            let d = dx.hypot(dy);
            assert!(d >= R - 0.01, "tip at ({:.3}, {:.3}, {:.3}) is {d:.3} from the wall", q.x, q.y, q.z);
        }
    }
}

/// The open edge, from the wall to the notch's far corner, 0.05 mm
/// inside the part: the stock left there is at the floor.
fn assert_open_edge_cleared(s: &[Point3], what: &str) {
    for k in 0..=40 {
        let x = 0.05 + f64::from(k) * (H - 0.1) / 40.0;
        for y in [H - 0.05, -(H - 0.05)] {
            let left = carried(s, x, y);
            assert!(left <= FLOOR + 1e-6, "{what}: stock stands at Z {left:.3} over ({x:.3}, {y:.3}); the floor is {FLOOR}");
        }
    }
}

/// The reopened reading: after an op that cleared past the open edge,
/// the masked ladder still reaches the open edge beside the wall. Before
/// the fix the stock stood at Z 1.000 over (0.05, 19.95).
#[test]
fn a_masked_pocket_still_clears_the_open_edge_beside_a_wall() {
    let f = notched_part();
    let region = footprint(&f);
    let grid = mask();
    let cols = Columns::of_grid(&grid);
    for (what, path) in [
        ("plain ladder", pocket3d_prev(&f, &region, &params(), Some(&cols))),
        ("offset shell", pocket3d_offset_prev(&f, &region, &params(), Some(&cols))),
    ] {
        let path = path.unwrap_or_else(|e| panic!("{what}: the notch generates: {e}"));
        let s = feed_samples(&path);
        assert_no_gouge(&s);
        assert_open_edge_cleared(&s, what);
    }
}

/// The mask still only removes cuts: no cutting move lies where its disc
/// meets no standing column (all the stock past the cleared ring is
/// air), and a fully machined mask still has nothing to cut.
#[test]
fn the_mask_still_keeps_the_cutter_off_cleared_ground() {
    let f = notched_part();
    let region = footprint(&f);
    let grid = mask();
    let path = pocket3d_prev(&f, &region, &params(), Some(&Columns::of_grid(&grid))).expect("the notch generates");
    for q in feed_samples(&path) {
        let dx = (q.x.abs() - H).max(0.0);
        let dy = (q.y.abs() - H).max(0.0);
        assert!(dx.hypot(dy) <= R + 0.01, "a cut at ({:.3}, {:.3}, {:.3}) touches no standing stock", q.x, q.y, q.z);
    }
    let done = Dexel::block(-30.0, -30.0, 30.0, 30.0, FLOOR, 0.5).expect("dexel");
    assert!(pocket3d_prev(&f, &region, &params(), Some(&Columns::of_grid(&done))).is_err(), "nothing standing, nothing to cut");
}
