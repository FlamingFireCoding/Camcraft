//! Slot wall lap (#2906): one lap of the cutter-centre stadium after the
//! rows, so the round ends are cut to size. Plain circle and line geometry
//! (a stadium boundary as two half circles and two tangent sides); our own
//! design, no vendor material.

use crate::geo2d::Point2;

use super::arcs::Seg;

/// The lap of the cutter-centre stadium of half-width `h` on `a` -> `b`
/// (unit axis `u`, unit normal `n = rot90(u)`), from the tangent point
/// the last row ends on and back to it. `at_b` says that point is
/// `b + h n` (else `a + h n`). `ccw` walks the lap with the slot's air
/// on the left (the climb sense of the rows); `!ccw` walks it the other
/// way. Each half circle is two quarter arcs through its tip, so no
/// single arc spans 180 degrees. Returns the walk (for the ladder) and
/// the lap's vertices (for the pass polyline), without the start point.
///
/// The rows reach each round end only at their own end points; the
/// tool does not follow the arc between two row ends, and leaves up to
/// `h (1 - cos)` of a wide slot's ends uncut (0.86 mm on a 10 mm slot
/// with a 6 mm tool). The lap rides that arc exactly, and never leaves
/// the stadium: every point of it is on the boundary.
pub(super) fn wall_lap(a: Point2, b: Point2, u: Point2, n: Point2, h: f64, at_b: bool, ccw: bool) -> (Vec<Seg>, Vec<Point2>) {
    let at = |c: Point2, s: f64, t: f64| Point2::new(c.x + n.x * s + u.x * t, c.y + n.y * s + u.y * t);
    // The boundary in CCW order (air on the left): `b`'s cap from -n
    // through its tip to +n, the +n side back to `a`, `a`'s cap from +n
    // through its tip to -n, the -n side on to `b`.
    let t = [at(b, -h, 0.0), at(b, h, 0.0), at(a, h, 0.0), at(a, -h, 0.0)];
    let tip_b = at(b, 0.0, h);
    let tip_a = at(a, 0.0, -h);
    let start = if at_b { 1usize } else { 2 };
    let mut segs = Vec::with_capacity(6);
    let mut pts = Vec::with_capacity(6);
    for k in 0..4usize {
        let (i, j) = if ccw { ((start + k) % 4, (start + k + 1) % 4) } else { ((start + 4 - k) % 4, (start + 3 - k) % 4) };
        // Edge {0,1} is `b`'s cap and edge {2,3} is `a`'s; the other two
        // are the straight sides.
        let cap = match (i.min(j), i.max(j)) {
            (0, 1) => Some((b, tip_b)),
            (2, 3) => Some((a, tip_a)),
            _ => None,
        };
        match cap {
            Some((center, tip)) => {
                segs.push(Seg::Arc { center, ccw, to: tip });
                segs.push(Seg::Arc { center, ccw, to: t[j] });
                pts.push(tip);
                pts.push(t[j]);
            }
            None => {
                segs.push(Seg::Line(vec![t[j]]));
                pts.push(t[j]);
            }
        }
    }
    (segs, pts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lap(at_b: bool, ccw: bool) -> (Vec<Seg>, Vec<Point2>) {
        let (a, b) = (Point2::new(-25.0, 15.0), Point2::new(25.0, 15.0));
        wall_lap(a, b, Point2::new(1.0, 0.0), Point2::new(0.0, 1.0), 2.0, at_b, ccw)
    }

    #[test]
    fn the_lap_closes_on_its_start() {
        for (at_b, ccw) in [(true, true), (true, false), (false, true), (false, false)] {
            let (segs, pts) = lap(at_b, ccw);
            let start = if at_b { Point2::new(25.0, 17.0) } else { Point2::new(-25.0, 17.0) };
            assert_eq!(segs.len(), 6, "two straights and four quarter arcs");
            assert_eq!(pts.last().copied(), Some(start), "at_b={at_b} ccw={ccw}");
        }
    }

    #[test]
    fn ccw_from_b_runs_the_plus_n_side_first() {
        let (segs, pts) = lap(true, true);
        assert!(matches!(segs.first(), Some(Seg::Line(_))));
        assert_eq!(pts.first().copied(), Some(Point2::new(-25.0, 17.0)));
        assert_eq!(pts.get(1).copied(), Some(Point2::new(-27.0, 15.0)), "a's tip");
    }

    #[test]
    fn cw_from_b_turns_round_b_first() {
        let (segs, pts) = lap(true, false);
        assert!(matches!(segs.first(), Some(Seg::Arc { ccw: false, .. })));
        assert_eq!(pts.first().copied(), Some(Point2::new(27.0, 15.0)), "b's tip");
    }
}
