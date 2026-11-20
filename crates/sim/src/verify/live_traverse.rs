//! #3077: the first traverse of a live-tool cross drill on a lathe or a mill-turn, from the turret's index position at the top of the X and Z travel, must pass outside the work and the steel - our own reading of the RS-274 / ISO 6983 lathe rapid (G00: each axis at its own rate, so the path stays in the box of its two ends) and of the shop rule "never traverse through uncut stock or steel"; no vendor material consulted.
//!
//! The lathe post positions C first (with the tool at the index
//! position), then sends X and Z in one G00 to the start: the tip at
//! radius `r` about the spindle, at height `z`. Each axis runs at its
//! own rate, so the tip can pass any point of the box between the index
//! position (X and Z at the top of their travel) and the start: every
//! radius from `r` out, at every height from `z` up, in the station's
//! half-plane. The tool body stands outward from the tip, `tool_radius`
//! wide on each side, so the swept solid is the slab of the part frame
//! `u >= r`, `|v| <= tool_radius`, `z >= z - tool_radius` (`u` the
//! start's radial direction, `v` across it).
//!
//! C stands at its angle before X and Z move, so the work and the steel
//! that turns with it (a chuck) stand in the part frame as modelled: the
//! slab is read against them exactly (each facet clipped by the slab's
//! four planes). The fixed steel (a tailstock, a table) stands in the
//! machine frame at an angle the part frame does not state: it is read
//! revolved about the spindle, its largest radius at the band's heights.
//! NOT covered: a program that starts with the turret low near the work
//! (after a manual jog, for example) - decision #3077.

use crate::fixture::FixtureBox;
use crate::geometry::{Point3, Triangle};

/// The start of a live-tool cross drill as the law reads it: the tip
/// `start` in the part frame, the spindle line through `spindle` (X, Y)
/// along Z, and the radius of the tool.
#[derive(Clone, Copy, Debug)]
pub struct LiveStart {
    pub start: Point3,
    pub spindle: [f64; 2],
    pub tool_radius: f64,
}

/// What stands in the way of the first traverse.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LiveHit {
    /// The stock or a part (`index` into the work list, then the work
    /// boxes) meets the slab; `radius_mm` is its largest radius there.
    Work { index: usize, radius_mm: f64 },
    /// Tagged steel: `turning` (a chuck, read in the slab) or fixed (a
    /// tailstock, a table, read revolved), `index` into its list.
    Steel { index: usize, turning: bool, radius_mm: f64 },
}

/// The start, the spindle or the tool radius of a live-tool first
/// traverse is not finite, or the start stands on the spindle line: the
/// traverse cannot be judged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LiveUnjudged;

/// What the live-tool first traverse is judged against: `work`, the
/// stock and the parts (each a facet list; `work_boxes` a stock that has
/// only its box), and the tagged steel (`fixture::workholding_for`),
/// `turning` (a chuck) and `fixed` (a tailstock, a table).
#[derive(Clone, Copy, Debug, Default)]
pub struct LiveSolids<'a> {
    pub work: &'a [&'a [Triangle]],
    pub work_boxes: &'a [FixtureBox],
    pub turning: &'a [FixtureBox],
    pub fixed: &'a [FixtureBox],
}

/// The slab the first traverse can sweep, in the part frame.
struct Slab {
    o: [f64; 2],
    u: [f64; 2],
    r: f64,
    w: f64,
    z_lo: f64,
}

impl Slab {
    /// `None` when the start cannot be read: the caller refuses by name.
    fn of(s: &LiveStart, tol: f64) -> Option<Slab> {
        let LiveStart { start, spindle, tool_radius } = *s;
        let finite = [start.x, start.y, start.z, spindle[0], spindle[1], tool_radius, tol].iter().all(|v| v.is_finite());
        if !finite || tool_radius < 0.0 {
            return None;
        }
        let (dx, dy) = (start.x - spindle[0], start.y - spindle[1]);
        let r = dx.hypot(dy);
        if r <= 1e-9 {
            return None;
        }
        Some(Slab { o: spindle, u: [dx / r, dy / r], r: r - tol.max(0.0), w: tool_radius, z_lo: start.z - tool_radius })
    }

    fn radius(&self, p: Point3) -> f64 {
        (p.x - self.o[0]).hypot(p.y - self.o[1])
    }

    /// The four half-spaces of the slab, each as `plane(k, p) >= 0`:
    /// along the station's radius, its two sides, and the band's floor.
    fn plane(&self, k: usize, p: Point3) -> f64 {
        let (dx, dy) = (p.x - self.o[0], p.y - self.o[1]);
        let across = -dx * self.u[1] + dy * self.u[0];
        match k {
            0 => dx * self.u[0] + dy * self.u[1] - self.r,
            1 => self.w - across,
            2 => self.w + across,
            _ => p.z - self.z_lo,
        }
    }

    /// The largest radius of the part of `t` inside the slab, or `None`
    /// when no part of it is there (each plane clips the polygon in turn).
    fn meets(&self, t: &Triangle) -> Option<f64> {
        let pts = [t.a, t.b, t.c];
        if pts.iter().any(|p| !(p.x.is_finite() && p.y.is_finite() && p.z.is_finite())) {
            // A facet the law cannot read is never passed.
            return Some(f64::INFINITY);
        }
        let mut poly: Vec<Point3> = pts.to_vec();
        for k in 0..4 {
            let mut out = Vec::with_capacity(poly.len() + 1);
            for i in 0..poly.len() {
                let (p, q) = (poly[i], poly[(i + 1) % poly.len()]);
                let (fp, fq) = (self.plane(k, p), self.plane(k, q));
                if fp >= 0.0 {
                    out.push(p);
                }
                if (fp >= 0.0) != (fq >= 0.0) {
                    let s = (fp / (fp - fq)).clamp(0.0, 1.0);
                    out.push(Point3::new(p.x + s * (q.x - p.x), p.y + s * (q.y - p.y), p.z + s * (q.z - p.z)));
                }
            }
            poly = out;
            if poly.is_empty() {
                return None;
            }
        }
        poly.iter().map(|p| self.radius(*p)).reduce(f64::max)
    }

    /// The largest radius where `facets` meet the slab.
    fn meets_all(&self, facets: &[Triangle]) -> Option<f64> {
        facets.iter().filter_map(|t| self.meets(t)).reduce(f64::max)
    }

    /// A box read revolved: its largest radius (its four X/Y corners)
    /// when its top reaches the band's heights, and when that radius
    /// reaches the start's.
    fn reaches_revolved(&self, b: &FixtureBox) -> Option<f64> {
        if !box_finite(b) {
            return Some(f64::INFINITY);
        }
        if b.max.z < self.z_lo {
            return None;
        }
        let reach = [b.min.x, b.max.x]
            .iter()
            .flat_map(|&x| [b.min.y, b.max.y].map(|y| (x - self.o[0]).hypot(y - self.o[1])))
            .fold(f64::NEG_INFINITY, f64::max);
        (reach >= self.r).then_some(reach)
    }
}

fn box_finite(b: &FixtureBox) -> bool {
    [b.min.x, b.min.y, b.min.z, b.max.x, b.max.y, b.max.z].iter().all(|v| v.is_finite())
}

/// The twelve facets of `b`. The slab runs out to infinity, so it meets
/// a bounded box only through its faces.
fn box_facets(b: &FixtureBox) -> Vec<Triangle> {
    let c = |i: usize| {
        Point3::new(if i & 1 == 0 { b.min.x } else { b.max.x }, if i & 2 == 0 { b.min.y } else { b.max.y }, if i & 4 == 0 { b.min.z } else { b.max.z })
    };
    let quads = [[0, 1, 3, 2], [4, 6, 7, 5], [0, 4, 5, 1], [2, 3, 7, 6], [0, 2, 6, 4], [1, 5, 7, 3]];
    quads.iter().flat_map(|q| [Triangle::new(c(q[0]), c(q[1]), c(q[2])), Triangle::new(c(q[0]), c(q[2]), c(q[3]))]).collect()
}

/// The live-tool first-traverse law (#3077). `Ok(Some(hit))` names the
/// first solid in the way: the work or the turning steel that meets the
/// slab (less `tol` on the start's radius), or the fixed steel that
/// reaches the start's radius at the band's heights. `Ok(None)` when the
/// traverse is clear.
///
/// # Errors
/// [`LiveUnjudged`] when the start cannot be read: the caller refuses by
/// name.
pub fn live_first_traverse(s: &LiveStart, solids: &LiveSolids<'_>, tol: f64) -> Result<Option<LiveHit>, LiveUnjudged> {
    let slab = Slab::of(s, tol).ok_or(LiveUnjudged)?;
    for (index, facets) in solids.work.iter().enumerate() {
        if let Some(radius_mm) = slab.meets_all(facets) {
            return Ok(Some(LiveHit::Work { index, radius_mm }));
        }
    }
    let boxed = |b: &FixtureBox| if box_finite(b) { slab.meets_all(&box_facets(b)) } else { Some(f64::INFINITY) };
    for (i, b) in solids.work_boxes.iter().enumerate() {
        if let Some(radius_mm) = boxed(b) {
            return Ok(Some(LiveHit::Work { index: solids.work.len() + i, radius_mm }));
        }
    }
    for (index, b) in solids.turning.iter().enumerate() {
        if let Some(radius_mm) = boxed(b) {
            return Ok(Some(LiveHit::Steel { index, turning: true, radius_mm }));
        }
    }
    for (index, b) in solids.fixed.iter().enumerate() {
        if let Some(radius_mm) = slab.reaches_revolved(b) {
            return Ok(Some(LiveHit::Steel { index, turning: false, radius_mm }));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-6;

    fn b(lo: [f64; 3], hi: [f64; 3]) -> FixtureBox {
        FixtureBox { min: Point3::new(lo[0], lo[1], lo[2]), max: Point3::new(hi[0], hi[1], hi[2]) }
    }

    /// A start at radius 15 on +X, Z 10, a 4 mm drill: the slab is X 15
    /// and out, Y -2..2, Z 8 and up.
    fn start() -> LiveStart {
        LiveStart { start: Point3::new(15.0, 0.0, 10.0), spindle: [0.0, 0.0], tool_radius: 2.0 }
    }

    fn steel(s: &LiveStart, turning: &[FixtureBox], fixed: &[FixtureBox]) -> Result<Option<LiveHit>, LiveUnjudged> {
        live_first_traverse(s, &LiveSolids { turning, fixed, ..LiveSolids::default() }, TOL)
    }

    #[test]
    fn a_tailstock_over_the_hole_is_in_the_way_and_a_chuck_under_it_is_not() {
        let chuck = b([-40.0, -40.0, -110.0], [40.0, 40.0, -80.0]);
        let tail = b([-30.0, -30.0, 25.0], [30.0, 30.0, 60.0]);
        assert_eq!(steel(&start(), &[chuck], &[]), Ok(None));
        let hit = steel(&start(), &[chuck], &[tail]);
        let Ok(Some(LiveHit::Steel { index: 0, turning: false, radius_mm })) = hit else { panic!("{hit:?}") };
        assert!((radius_mm - 30.0 * 2f64.sqrt()).abs() < 1e-9, "{radius_mm}");
        // A thin tailstock centre inside the start's radius is passed.
        let centre = b([-5.0, -5.0, 25.0], [5.0, 5.0, 60.0]);
        assert_eq!(steel(&start(), &[], &[centre]), Ok(None));
        // Fixed steel is read revolved: a block on the far side counts.
        let far = b([-30.0, -10.0, 20.0], [-20.0, 10.0, 30.0]);
        assert!(matches!(steel(&start(), &[], &[far]), Ok(Some(LiveHit::Steel { turning: false, .. }))));
    }

    #[test]
    fn steel_that_turns_is_read_in_the_station_slab() {
        let jaw = |top: f64| b([-30.0, -30.0, -40.0], [30.0, 30.0, top]);
        // The slab starts at Z 8 (the tip at Z 10, less the tool radius).
        assert_eq!(steel(&start(), &[jaw(7.9)], &[]), Ok(None));
        assert!(matches!(steel(&start(), &[jaw(8.1)], &[]), Ok(Some(LiveHit::Steel { turning: true, .. }))));
        // A jaw on the far side of the spindle stands out of the station's
        // half-plane: C turned it there before X and Z move.
        let far = b([-30.0, -10.0, -40.0], [-20.0, 10.0, 30.0]);
        assert_eq!(steel(&start(), &[far], &[]), Ok(None));
        // A jaw 3 mm beside the station's line (the tool is 2 mm wide).
        let beside = b([20.0, 3.0, -40.0], [30.0, 10.0, 30.0]);
        assert_eq!(steel(&start(), &[beside], &[]), Ok(None));
        let touching = b([20.0, 1.9, -40.0], [30.0, 10.0, 30.0]);
        assert!(matches!(steel(&start(), &[touching], &[]), Ok(Some(LiveHit::Steel { .. }))));
    }

    #[test]
    fn the_spindle_line_off_the_origin_moves_the_slab() {
        // The start at radius 15 about X 100: the slab is X 115 and out.
        let off = LiveStart { start: Point3::new(115.0, 0.0, 10.0), spindle: [100.0, 0.0], ..start() };
        let near = b([110.0, -5.0, 0.0], [120.0, 5.0, 30.0]);
        assert!(matches!(steel(&off, &[near], &[]), Ok(Some(LiveHit::Steel { turning: true, .. }))));
        let world_origin = b([10.0, -5.0, 0.0], [20.0, 5.0, 30.0]);
        assert_eq!(steel(&off, &[world_origin], &[]), Ok(None));
    }

    #[test]
    fn a_square_bar_is_read_at_the_face_the_station_looks_at() {
        // A 44 square bar (corner radius 31.1): a station on its face's
        // normal at radius 27 clears it; one on its diagonal at 27 does
        // not, as its corner stands at 31.1 there.
        let bar = b([-22.0, -22.0, -40.0], [22.0, 22.0, 20.0]);
        let solids = LiveSolids { work_boxes: &[bar], ..LiveSolids::default() };
        assert_eq!(live_first_traverse(&LiveStart { start: Point3::new(27.0, 0.0, 10.0), ..start() }, &solids, TOL), Ok(None));
        let d = 27.0 / 2f64.sqrt();
        let hit = live_first_traverse(&LiveStart { start: Point3::new(d, d, 10.0), ..start() }, &solids, TOL);
        assert!(matches!(hit, Ok(Some(LiveHit::Work { index: 0, .. }))), "{hit:?}");
    }

    #[test]
    fn a_flange_over_the_hole_is_read_where_its_facet_crosses_the_slab() {
        // One facet from (40, 0, 0) to (0, 0, 20) and (0, 1, 20): its
        // vertices in the band are inside the start's radius, but where
        // it crosses Z 8 it reaches X 24.
        let t = Triangle::new(Point3::new(40.0, 0.0, 0.0), Point3::new(0.0, 0.0, 20.0), Point3::new(0.0, 1.0, 20.0));
        let work: &[&[Triangle]] = &[&[t]];
        let hit = live_first_traverse(&start(), &LiveSolids { work, ..LiveSolids::default() }, TOL);
        let Ok(Some(LiveHit::Work { index: 0, radius_mm })) = hit else { panic!("{hit:?}") };
        // The edge to (0, 1, 20) crosses Z 8 at (24, 0.4): the largest.
        assert!((radius_mm - 24f64.hypot(0.4)).abs() < 1e-9, "{radius_mm}");
        // A plain bar face of radius 10 is inside the start's radius.
        let bar = Triangle::new(Point3::new(10.0, 0.0, -80.0), Point3::new(10.0, 0.0, 20.0), Point3::new(0.0, 10.0, 20.0));
        let work: &[&[Triangle]] = &[&[bar]];
        assert_eq!(live_first_traverse(&start(), &LiveSolids { work, ..LiveSolids::default() }, TOL), Ok(None));
    }

    #[test]
    fn a_start_that_cannot_be_read_is_not_judged() {
        let mut s = start();
        s.start.z = f64::NAN;
        assert_eq!(steel(&s, &[], &[]), Err(LiveUnjudged));
        let mut s = start();
        s.tool_radius = f64::INFINITY;
        assert_eq!(steel(&s, &[], &[]), Err(LiveUnjudged));
        // A start on the spindle line has no station direction.
        let on_axis = LiveStart { start: Point3::new(0.0, 0.0, 10.0), ..start() };
        assert_eq!(steel(&on_axis, &[], &[]), Err(LiveUnjudged));
    }
}
