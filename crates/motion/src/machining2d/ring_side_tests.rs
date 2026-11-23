//! #3744 pins of the one ring set, on a dog-bone part (two discs of
//! radius 12 at X -15 and X 15, joined by a neck 8 mm wide) in a stock
//! box of 80 x 50, a tool of radius 4 and a step of 2.

use super::{mirror, normal, resolve};
use crate::geo2d::{Loop, Point2, Region};
use crate::machining2d::level_order::{Piece, WALL};
use crate::machining2d::level_rings::level_pieces;
use crate::machining2d::pass_start::{PassStart, StartSide};
use crate::machining2d::pocket_strategy::PocketStrategy;

const R: f64 = 4.0;
const STEP: f64 = 2.0;

fn region(loops: Vec<Vec<Point2>>) -> Region {
    let loops = loops.into_iter().map(|l| Loop::new(l).unwrap_or_else(|e| panic!("loop: {e:?}"))).collect();
    Region::new(loops).unwrap_or_else(|e| panic!("region: {e:?}"))
}

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<Point2> {
    vec![Point2::new(x0, y0), Point2::new(x1, y0), Point2::new(x1, y1), Point2::new(x0, y1)]
}

fn disc(cx: f64, r: f64) -> Vec<Point2> {
    (0..128)
        .map(|k| {
            let a = f64::from(k) * std::f64::consts::TAU / 128.0;
            Point2::new(cx + r * a.cos(), r * a.sin())
        })
        .collect()
}

/// The stock, and the band the tool's disc may sweep round the part: the
/// stock grown by the tool radius less the part, and its cutter-centre
/// area (the stock box less the part grown by the radius).
fn dog_bone() -> (Region, Region, Region) {
    let part = region(vec![disc(-15.0, 12.0)])
        .union(&region(vec![disc(15.0, 12.0)]))
        .and_then(|p| p.union(&region(vec![rect(-15.0, -4.0, 15.0, 4.0)])))
        .unwrap_or_else(|e| panic!("part: {e:?}"));
    let stock = region(vec![rect(-40.0, -25.0, 40.0, 25.0)]);
    let band = stock.offset(R).and_then(|s| s.difference(&part)).unwrap_or_else(|e| panic!("band: {e:?}"));
    let center = band.offset(-R).unwrap_or_else(|e| panic!("center: {e:?}"));
    (stock, band, center)
}

fn pieces(start: PassStart) -> Vec<Piece> {
    let (_, band, center) = dog_bone();
    match level_pieces(&band, &center, R, STEP, true, start, true) {
        Ok(Some(p)) => p,
        other => panic!("{start:?}: {:?}", other.map(|o| o.map(|p| p.len()))),
    }
}

/// Each piece in its normal form, sorted: the ring set as a multiset.
fn set(ps: &[Piece]) -> Vec<(bool, Vec<Point2>)> {
    let mut out: Vec<(bool, Vec<Point2>)> = ps.iter().map(normal).collect();
    out.sort_by(|a, b| {
        let key = |p: &(bool, Vec<Point2>)| p.1.first().map_or((0.0, 0.0), |q| (q.x, q.y));
        let (ka, kb) = (key(a), key(b));
        ka.0.total_cmp(&kb.0).then(ka.1.total_cmp(&kb.1)).then(a.1.len().cmp(&b.1.len()))
    });
    out
}

fn same(a: &[(bool, Vec<Point2>)], b: &[(bool, Vec<Point2>)]) -> bool {
    a.len() == b.len()
        && a.iter().zip(b).all(|(x, y)| {
            x.0 == y.0 && x.1.len() == y.1.len() && x.1.iter().zip(&y.1).all(|(p, q)| (p.x - q.x).hypot(p.y - q.y) <= 1e-3)
        })
}

/// #3744 pin: Inside and Outside cut the same multiset of rings (within
/// a micron, after a start-point and sense normalisation). Only the ring
/// order differs: Inside's first ring (on the part) is Outside's last,
/// and Outside's first ring is the area's outer edge.
#[test]
fn inside_and_outside_cut_the_same_rings() {
    let inside = pieces(PassStart::Inside);
    let outside = pieces(PassStart::Outside);
    assert!(inside.len() > 4, "the fixture has rings: {}", inside.len());
    assert!(same(&set(&inside), &set(&outside)), "the ring sets differ: {} Inside pieces, {} Outside pieces", inside.len(), outside.len());
    let at = |ps: &[Piece], lev: u32| set(&ps.iter().filter(|p| p.level == lev).map(|p| Piece { level: lev, pts: p.pts.clone(), closed: p.closed, core: p.core }).collect::<Vec<_>>());
    assert!(same(&at(&inside, 0), &at(&outside, WALL)), "the part wall is Inside's first ring and Outside's last");
    assert!(same(&at(&inside, WALL), &at(&outside, 0)), "the outer edge is Outside's first ring and Inside's last");
    // The order runs the other way: the ring next to the part is cut
    // early by Inside and late by Outside.
    let top = inside.iter().filter(|p| p.level != WALL).map(|p| p.level).max().unwrap_or(0);
    assert!(same(&at(&inside, 1), &at(&outside, top)), "ring 1 of Inside is ring {top} of Outside");
}

/// The mirror keeps the climb sense: a piece walked the other way has its
/// uncut side on the other hand, so each Outside piece is the Inside
/// piece reversed, never the same walk.
#[test]
fn the_mirror_reverses_each_piece_and_swaps_the_ends_of_the_order() {
    let mut ps = vec![
        Piece { level: 0, pts: vec![Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)], closed: false, core: false },
        Piece { level: 2, pts: vec![Point2::new(0.0, 1.0), Point2::new(1.0, 1.0)], closed: false, core: false },
        Piece { level: 3, pts: vec![Point2::new(0.0, 2.0), Point2::new(1.0, 2.0)], closed: false, core: false },
        Piece { level: WALL, pts: vec![Point2::new(0.0, 3.0), Point2::new(1.0, 3.0)], closed: false, core: false },
        Piece { level: 5, pts: vec![Point2::new(9.0, 9.0), Point2::new(8.0, 9.0)], closed: true, core: true },
    ];
    mirror(&mut ps);
    let levels: Vec<u32> = ps.iter().map(|p| p.level).collect();
    assert_eq!(levels, vec![WALL, 2, 1, 0, 5]);
    assert_eq!(ps[0].pts.first().map(|p| p.x), Some(1.0), "reversed");
    assert_eq!(ps[4].pts.first().map(|p| p.x), Some(9.0), "a core piece keeps its walk");
}

fn side(start: PassStart, stock: Option<Region>, around_part: bool) -> StartSide {
    StartSide { start, stock, around_part }
}

/// #3744 pin: Auto round a part takes one of the two sides of the one
/// ring set: Outside when the stock box leaves the outer edge in air,
/// Inside when the stock is unknown or the outer edge stands in the
/// stock. A chosen side, a pocket and a row strategy stay as they are.
#[test]
fn auto_picks_one_side_of_the_one_ring_set() {
    let (stock, _, center) = dog_bone();
    let pick = |s: &StartSide, strategy| resolve(&center, strategy, R, s).map(|r| r.start).unwrap_or_else(|e| panic!("{e:?}"));
    let w = PocketStrategy::Waterline;
    assert_eq!(pick(&side(PassStart::Auto, Some(stock.clone()), true), w), PassStart::Outside);
    assert_eq!(pick(&side(PassStart::Auto, None, true), w), PassStart::Inside);
    let big = region(vec![rect(-60.0, -45.0, 60.0, 45.0)]);
    assert_eq!(pick(&side(PassStart::Auto, Some(big.clone()), true), w), PassStart::Inside, "the outer edge stands in the stock");
    assert_eq!(pick(&side(PassStart::Auto, Some(stock.clone()), false), w), PassStart::Auto, "a pocket keeps Auto");
    assert_eq!(pick(&side(PassStart::Auto, Some(stock.clone()), true), PocketStrategy::Zigzag), PassStart::Auto);
    assert_eq!(pick(&side(PassStart::Inside, Some(stock), true), w), PassStart::Inside);
    assert!(resolve(&center, w, f64::NAN, &side(PassStart::Auto, Some(big), true)).is_err(), "NaN is refused");
}
