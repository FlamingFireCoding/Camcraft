//! #3567: a later pass of a 2D Offset "Pass starts at: Outside" plan must
//! not plunge or helix into uncut stock at a dead end. The case is the
//! hand trace of PR #3353: a 100 x 40 stock band around a round part of
//! radius 15, tool D8, stepover 4, whose rings split. Each later pass of
//! a split level starts on the part wall, one stepover away from the cut
//! already walked, so it has no cleared column of its own.
//!
//! The check is a 2-D material sim on a 0.25 mm grid. A later ENTRY that
//! is a plunge or a helix cuts stock at the full tool width and is the
//! fault; a ramp along the pass enters the same stock in a gradual way
//! and is the chosen entry (maintainer decision on #3567), so the pin
//! counts only plunges and helices after the first planar floor feed.
//!
//! Gate map: covers camcraft_motion::machining2d (pass_start,
//! level_rings, level_order, descent). Provenance: our own regression of
//! the review of PR #3353; the radial-engagement measure of the public
//! milling handbooks; no vendor material consulted.

use camcraft_motion::cl::{MoveKind, Toolpath};
use camcraft_motion::entry::Entry3D;
use camcraft_motion::geo2d::{Loop, Point2, Region};
use camcraft_motion::links::Links;
use camcraft_motion::load::DEFAULT_OPTIMAL_LOAD;
use camcraft_motion::machining2d::{CornerRelief, PassStart, PocketParams, PocketStrategy, pocket_started};

const R: f64 = 4.0;
const STEP: f64 = 4.0;
const CELL: f64 = 0.25;
const PART: f64 = 15.0;

fn lp(pts: Vec<Point2>) -> Loop {
    let Ok(l) = Loop::new(pts) else { panic!("fixture loop") };
    l
}

fn rect(hx: f64, hy: f64) -> Loop {
    lp(vec![Point2::new(-hx, -hy), Point2::new(hx, -hy), Point2::new(hx, hy), Point2::new(-hx, hy)])
}

fn circle(r: f64) -> Loop {
    let n = 256;
    lp((0..n)
        .map(|k| {
            let a = f64::from(k) * std::f64::consts::TAU / f64::from(n);
            Point2::new(r * a.cos(), r * a.sin())
        })
        .collect())
}

fn params(climb: bool) -> PocketParams {
    PocketParams {
        blend_radius: None,
        tool_diameter: 2.0 * R,
        stepover: STEP,
        stock_to_leave: 0.0,
        clearance_z: 10.0,
        retract_z: None,
        top_z: 0.0,
        bottom_z: -3.0,
        stepdown: 5.0,
        feed: 1000.0,
        plunge_feed: 300.0,
        climb,
        strategy: PocketStrategy::Waterline,
        entry: Entry3D::Helix,
        helix_radius: 1.6,
        optimal_load: DEFAULT_OPTIMAL_LOAD,
        blend_turns: false,
        links: Links::PLANAR,
    }
}

/// The material grid: true where stock still stands.
struct Grid {
    lo: Point2,
    nx: usize,
    ny: usize,
    cells: Vec<bool>,
}

impl Grid {
    fn new(stock: &Region) -> Grid {
        let Some((lo, hi)) = stock.bounds() else { panic!("stock bounds") };
        let nx = ((hi.x - lo.x) / CELL).ceil() as usize + 1;
        let ny = ((hi.y - lo.y) / CELL).ceil() as usize + 1;
        let mut cells = vec![false; nx * ny];
        for j in 0..ny {
            for i in 0..nx {
                let p = Point2::new(lo.x + (i as f64 + 0.5) * CELL, lo.y + (j as f64 + 0.5) * CELL);
                cells[j * nx + i] = stock.contains(p);
            }
        }
        Grid { lo, nx, ny, cells }
    }

    /// Clear the disc at `c`; returns the area removed (mm^2).
    fn cut(&mut self, c: Point2) -> f64 {
        let i0 = (((c.x - R - self.lo.x) / CELL).floor().max(0.0)) as usize;
        let j0 = (((c.y - R - self.lo.y) / CELL).floor().max(0.0)) as usize;
        let i1 = ((((c.x + R - self.lo.x) / CELL).ceil()).max(0.0) as usize).min(self.nx);
        let j1 = ((((c.y + R - self.lo.y) / CELL).ceil()).max(0.0) as usize).min(self.ny);
        let mut n = 0usize;
        for j in j0..j1 {
            for i in i0..i1 {
                let p = Point2::new(self.lo.x + (i as f64 + 0.5) * CELL, self.lo.y + (j as f64 + 0.5) * CELL);
                if (p.x - c.x).hypot(p.y - c.y) <= R && self.cells[j * self.nx + i] {
                    self.cells[j * self.nx + i] = false;
                    n += 1;
                }
            }
        }
        n as f64 * CELL * CELL
    }
}

/// The stock (mm^2 of the 2-D grid) that a later entry cuts after the
/// first planar floor feed: a plunge or a helix that lands in a column no
/// earlier pass of the level cleared. A ramp along the pass is a gradual
/// entry and is not counted (maintainer decision on #3567).
fn late_entry_cut(path: &Toolpath, stock: &Region, floor: f64, top: f64) -> f64 {
    let mut g = Grid::new(stock);
    let (mut seen, mut late) = (false, 0.0);
    for m in path.moves.iter().filter(|m| m.kind != MoveKind::Rapid) {
        if m.from.z.min(m.to.z) >= top - 1e-9 {
            continue;
        }
        let planar = (m.from.z - floor).abs() < 1e-9 && (m.to.z - floor).abs() < 1e-9;
        // A plunge (the axial kind) or a helix (a descending arc) enters
        // at the full tool width. A straight descending feed is a ramp
        // along the pass: it lowers the tool while it travels and is the
        // entry this pin asks for.
        let entry = m.to.z < m.from.z - 1e-9 && (m.kind == MoveKind::Plunge || m.arc.is_some());
        for (a, b) in m.chords(0.005) {
            let n = ((b.x - a.x).hypot(b.y - a.y) / (CELL * 0.5)).ceil().max(1.0) as usize;
            for s in 0..=n {
                let t = s as f64 / n as f64;
                let got = g.cut(Point2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t));
                if seen && entry && !planar {
                    late += got;
                }
            }
        }
        seen |= planar;
    }
    late
}

fn band() -> (Region, Region) {
    // The machining region reaches one tool radius past the stock, as an
    // Outside containment's area does; the part is the hole.
    let Ok(region) = Region::new(vec![rect(50.0 + R, 20.0 + R), circle(PART)]) else { panic!("region") };
    let Ok(stock) = Region::new(vec![rect(50.0, 20.0), circle(PART)]) else { panic!("stock") };
    (region, stock)
}

/// #3567 pin: a later pass of a split Outside level starts in cleared
/// stock - never a plunge or a helix at the far wall into uncut stock.
#[test]
fn outside_entries_after_the_first_land_in_cleared_stock() {
    let (region, stock) = band();
    let p = params(true);
    let path = match pocket_started(&region, &p, &[], &CornerRelief::default(), PassStart::Outside) {
        Ok(t) => t,
        Err(e) => panic!("{e:?}"),
    };
    let late = late_entry_cut(&path, &stock, p.bottom_z, p.top_z);
    assert!(late < 1.0, "the later entries cut {late:.2} mm^2 of uncut stock");
}
