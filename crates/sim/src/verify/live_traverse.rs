//! #3077: the first traverse of a live-tool cross drill on a lathe or a mill-turn, from the turret's index position at the top of the X and Z travel, must pass outside the work and the steel - our own reading of the RS-274 / ISO 6983 lathe rapid (G00: each axis at its own rate, so the path stays in the box of its two ends) and of the shop rule "never traverse through uncut stock or steel"; no vendor material consulted.
//!
//! The lathe post positions C first (with the tool at the index
//! position), then sends X and Z in one G00 to the start: the tip at
//! radius `r` about the spindle, at height `z`. Each axis runs at its
//! own rate, so the tip can pass any point of the box between the index
//! position (X and Z at the top of their travel) and the start: every
//! radius from `r` out, at every height from `z` up. The tool body
//! stands outward from the tip, `tool_radius` wide on each side, so the
//! band of heights starts `tool_radius` under `z`.
//!
//! The law reads each solid revolved about the spindle: its largest
//! radius in that band. Revolved is stricter than the C angle that the
//! post states, and it holds for the fixed steel (a tailstock) at any
//! angle too. A solid that reaches the start's radius in the band is in
//! the way. NOT covered: a program that starts with the turret low near
//! the work (after a manual jog, for example) - decision #3077.

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
    /// The stock or a part (`index` into the work list) reaches
    /// `radius_mm` in the band, at or over the start's radius.
    Work { index: usize, radius_mm: f64 },
    /// Tagged steel: `turning` (a chuck) or fixed (a tailstock, a
    /// table), `index` into its list.
    Steel { index: usize, turning: bool, radius_mm: f64 },
}

/// The band the first traverse can pass: radius at least `r`, height at
/// least `z_lo`. `None` when the start cannot be read (not finite, or no
/// finite tool radius): the caller refuses by name.
fn band(s: &LiveStart) -> Option<(f64, f64)> {
    let LiveStart { start, spindle, tool_radius } = *s;
    let finite = [start.x, start.y, start.z, spindle[0], spindle[1], tool_radius].iter().all(|v| v.is_finite());
    if !finite || tool_radius < 0.0 {
        return None;
    }
    let r = (start.x - spindle[0]).hypot(start.y - spindle[1]);
    Some((r, start.z - tool_radius))
}

/// The largest radius about `spindle` of `t` clipped to heights at or
/// over `z_lo` (`NEG_INFINITY` when no part of it is there). The radius
/// is convex, so its maximum over the clipped polygon is at a vertex of
/// it.
fn triangle_reach(t: &Triangle, spindle: [f64; 2], z_lo: f64) -> f64 {
    let pts = [t.a, t.b, t.c];
    // A non-finite vertex reads as reaching everywhere: the law never
    // passes a solid that it cannot read.
    if pts.iter().any(|p| !(p.x.is_finite() && p.y.is_finite() && p.z.is_finite())) {
        return f64::INFINITY;
    }
    let rad = |p: Point3| (p.x - spindle[0]).hypot(p.y - spindle[1]);
    let mut best = f64::NEG_INFINITY;
    for i in 0..3 {
        let (p, q) = (pts[i], pts[(i + 1) % 3]);
        if p.z >= z_lo {
            best = best.max(rad(p));
        }
        // The edge crosses the plane: its point on the plane is a vertex
        // of the clipped polygon.
        if (p.z < z_lo) != (q.z < z_lo) {
            let dz = q.z - p.z;
            if dz.abs() > 0.0 {
                let s = ((z_lo - p.z) / dz).clamp(0.0, 1.0);
                best = best.max(rad(Point3::new(p.x + s * (q.x - p.x), p.y + s * (q.y - p.y), z_lo)));
            }
        }
    }
    best
}

/// The largest radius about `spindle` of `b` at heights at or over
/// `z_lo`: its four X/Y corners when its top reaches the band.
fn box_reach(b: &FixtureBox, spindle: [f64; 2], z_lo: f64) -> f64 {
    let finite = [b.min.x, b.min.y, b.min.z, b.max.x, b.max.y, b.max.z].iter().all(|v| v.is_finite());
    if !finite {
        return f64::INFINITY;
    }
    if b.max.z < z_lo {
        return f64::NEG_INFINITY;
    }
    let mut best = f64::NEG_INFINITY;
    for x in [b.min.x, b.max.x] {
        for y in [b.min.y, b.max.y] {
            best = best.max((x - spindle[0]).hypot(y - spindle[1]));
        }
    }
    best
}

/// The start, the spindle or the tool radius of a live-tool first
/// traverse is not finite: the traverse cannot be judged.
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

/// The live-tool first-traverse law (#3077). `Ok(Some(hit))` names the
/// first solid that reaches the start's radius (less `tol`) in the band;
/// `Ok(None)` when the traverse is clear. A work box is named as
/// `LiveHit::Work` with its index after the facet lists.
///
/// # Errors
/// [`LiveUnjudged`] when the start cannot be read: the caller refuses by
/// name.
pub fn live_first_traverse(s: &LiveStart, solids: &LiveSolids<'_>, tol: f64) -> Result<Option<LiveHit>, LiveUnjudged> {
    let (r, z_lo) = band(s).ok_or(LiveUnjudged)?;
    let reaches = |reach: f64| reach > r - tol;
    for (index, solid) in solids.work.iter().enumerate() {
        let reach = solid.iter().map(|t| triangle_reach(t, s.spindle, z_lo)).fold(f64::NEG_INFINITY, f64::max);
        if reaches(reach) {
            return Ok(Some(LiveHit::Work { index, radius_mm: reach }));
        }
    }
    for (i, b) in solids.work_boxes.iter().enumerate() {
        let reach = box_reach(b, s.spindle, z_lo);
        if reaches(reach) {
            return Ok(Some(LiveHit::Work { index: solids.work.len() + i, radius_mm: reach }));
        }
    }
    for (list, turns) in [(solids.turning, true), (solids.fixed, false)] {
        for (index, b) in list.iter().enumerate() {
            let reach = box_reach(b, s.spindle, z_lo);
            if reaches(reach) {
                return Ok(Some(LiveHit::Steel { index, turning: turns, radius_mm: reach }));
            }
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

    /// A start at radius 15, Z 10, a 4 mm drill: the band is radius 15
    /// and out, Z 8 and up.
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
    }

    #[test]
    fn steel_just_under_the_band_is_passed_and_steel_in_its_tool_width_is_not() {
        let jaw = |top: f64| b([-30.0, -30.0, -40.0], [30.0, 30.0, top]);
        // The band starts at Z 8 (the tip at Z 10, less the tool radius).
        assert_eq!(steel(&start(), &[jaw(7.9)], &[]), Ok(None));
        assert!(matches!(steel(&start(), &[jaw(8.1)], &[]), Ok(Some(LiveHit::Steel { turning: true, .. }))));
        // The spindle line through X 100: the same jaw is read about it
        // (its far corner at radius 133.4) and still counts.
        let off = LiveStart { start: Point3::new(115.0, 0.0, 10.0), spindle: [100.0, 0.0], ..start() };
        assert!(matches!(steel(&off, &[jaw(8.1)], &[]), Ok(Some(LiveHit::Steel { .. }))));
    }

    #[test]
    fn a_stock_known_only_by_its_box_is_read_as_work() {
        let block = b([-20.0, -20.0, -40.0], [20.0, 20.0, 20.0]);
        let solids = LiveSolids { work_boxes: &[block], ..LiveSolids::default() };
        let hit = live_first_traverse(&start(), &solids, TOL);
        assert!(matches!(hit, Ok(Some(LiveHit::Work { index: 0, .. }))), "{hit:?}");
    }

    #[test]
    fn a_flange_over_the_hole_is_read_where_its_facet_crosses_the_band() {
        // One facet from (40, 0, 0) to (0, 0, 20) and (0, 1, 20): its
        // vertices in the band reach radius 1 only, but where it crosses
        // Z 8 it reaches radius 24.
        let t = Triangle::new(Point3::new(40.0, 0.0, 0.0), Point3::new(0.0, 0.0, 20.0), Point3::new(0.0, 1.0, 20.0));
        let work: &[&[Triangle]] = &[&[t]];
        let hit = live_first_traverse(&start(), &LiveSolids { work, ..LiveSolids::default() }, TOL);
        let Ok(Some(LiveHit::Work { index: 0, radius_mm })) = hit else { panic!("{hit:?}") };
        // The edge to (0, 1, 20) crosses Z 8 at (24, 0.4): the largest.
        assert!((radius_mm - 24f64.hypot(0.4)).abs() < 1e-9, "{radius_mm}");
        // A plain bar of radius 10 is inside the start's radius.
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
    }
}
