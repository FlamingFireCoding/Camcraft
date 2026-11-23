//! One ring set for every start side of an area round a part (#3744).
//! The rings are the level curves of the distance from the part (the
//! area's holes), clipped to the cutter-centre area: Inside walks them
//! from the part out, Outside walks the very same pieces from the far
//! edge in ([`mirror`]), and Auto only picks one of the two ([`resolve`]).
//! The start side changes the start point and the ring order, never the
//! rings.
//!
//! Provenance: contour-parallel offset rings measured from the selected
//! contour (Held, "On the Computational Geometry of Pocket Machining",
//! Springer LNCS 500, 1991); the mirror and the Auto rule are ours. No
//! vendor material consulted.

use crate::geo2d::{Loop, Point2, Region};

use super::face::GenError;
use super::level_order::{Piece, WALL, on_wall};
use super::pass_start::{PassStart, StartSide};
use super::pocket_strategy::PocketStrategy;

/// The pieces of one part's ring set, from the part out (the Inside
/// walk), turned into the Outside walk of the same set: each piece runs
/// the other way (the uncut side swaps with the cut side, so climb stays
/// climb), the area's outer edge becomes ring 0, ring `j` of `top`
/// becomes ring `top + 1 - j`, and the part wall is cut last. Core
/// pieces (a part with no hole) keep their own order.
pub(super) fn mirror(pieces: &mut [Piece]) {
    let top = pieces.iter().filter(|p| !p.core && p.level != WALL).map(|p| p.level).max().unwrap_or(0);
    for p in pieces.iter_mut().filter(|p| !p.core) {
        p.pts.reverse();
        p.level = match p.level {
            WALL => 0,
            0 => WALL,
            j => top.saturating_add(1).saturating_sub(j),
        };
    }
}

/// The one place where Auto chooses a start side for the rings of an
/// area round a part (#3744): Outside when the stock footprint is known
/// and the outer edge of the cutter-centre area stands in open air all
/// round (an Outside start there owes no wall-first cut), else Inside.
/// Every other case (a pocket, a row strategy, a side already chosen)
/// is returned as it is.
pub(super) fn resolve(center: &Region, strategy: PocketStrategy, tool_r: f64, side: &StartSide) -> Result<StartSide, GenError> {
    let rings = matches!(strategy, PocketStrategy::Waterline | PocketStrategy::OffsetRings);
    if side.start != PassStart::Auto || !side.around_part || !rings {
        return Ok(side.clone());
    }
    if !(tool_r.is_finite() && tool_r > 0.0) {
        return Err(GenError::Invalid("the tool radius must be positive and finite".into()));
    }
    let open = match side.stock.as_ref() {
        Some(stock) => {
            let mut open = true;
            for l in outer_loops(center)? {
                if on_wall(l.points(), Some(stock), tool_r, 0.25 * tool_r) {
                    open = false;
                    break;
                }
            }
            open
        }
        None => false,
    };
    let start = if open { PassStart::Outside } else { PassStart::Inside };
    Ok(StartSide { start, ..side.clone() })
}

/// The counter-clockwise loops of `region` that no other one encloses:
/// its outer edges (material nested in a hole has its own loop inside).
fn outer_loops(region: &Region) -> Result<Vec<&Loop>, GenError> {
    let ccw: Vec<&Loop> = region.loops().iter().filter(|l| l.is_ccw()).collect();
    let mut out = Vec::new();
    for (i, l) in ccw.iter().enumerate() {
        let Some(&p) = l.points().first() else { continue };
        let mut nested = false;
        for (k, o) in ccw.iter().enumerate() {
            if k != i && Region::new(vec![(*o).clone()])?.contains(p) {
                nested = true;
                break;
            }
        }
        if !nested {
            out.push(*l);
        }
    }
    Ok(out)
}

/// A ring piece as a set of points: open pieces from the smaller end,
/// closed rings from their smallest vertex and counter-clockwise. Two
/// walks of one ring in any start point and sense read the same.
#[cfg(test)]
pub(super) fn normal(p: &Piece) -> (bool, Vec<Point2>) {
    let lex = |a: &Point2, b: &Point2| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y));
    let mut pts = p.pts.clone();
    if p.closed {
        if pts.len() > 1 && pts.first() == pts.last() {
            pts.pop();
        }
        let area: f64 = (0..pts.len()).map(|i| {
            let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
            a.x * b.y - b.x * a.y
        }).sum();
        if area < 0.0 {
            pts.reverse();
        }
        if let Some(k) = (0..pts.len()).min_by(|&a, &b| lex(&pts[a], &pts[b])) {
            pts.rotate_left(k);
        }
    } else if let (Some(a), Some(b)) = (pts.first(), pts.last())
        && lex(b, a).is_lt()
    {
        pts.reverse();
    }
    (p.closed, pts)
}

#[cfg(test)]
#[path = "ring_side_tests.rs"]
mod tests;
