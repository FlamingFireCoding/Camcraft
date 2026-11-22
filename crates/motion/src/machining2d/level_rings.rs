//! Pass start Inside / Outside (#3316): the rings of one planar area as
//! level curves of the distance from the START side only, clipped to the
//! cutter-centre area as open pieces. A ring of the whole area
//! (`waterscan`) splits at a narrow place into closed loops that carry
//! arcs of both sides, so no order of whole loops cuts the far side late
//! enough; a level curve of one side has no arc of the other side.
//!
//! Outside: ring 0 is the stretch of the area's boundary on its outer
//! edge, ring j the outer edge (holes filled) eroded by `j` stepovers and
//! clipped to the area, and the walls (the holes' and keep-outs' edges)
//! come last. Inside is the mirror: ring 0 on the holes, ring j the holes
//! grown by `j` stepovers, the outer wall last.
//!
//! Provenance: contour-parallel pocketing as level sets of the distance
//! to the boundary (Held, "On the Computational Geometry of Pocket
//! Machining", Springer LNCS 500, 1991), measured from one side; the
//! side split, the order and the proofs are ours. No vendor material
//! consulted.

use crate::geo2d::{Point2, Region, RingClip};

use super::face::GenError;
use super::level_order::{Piece, WALL, order};
use super::pass_start::{PassStart, StartSide};
use super::pocket::PocketParams;
use super::pocket_plan::plan_passes;
use super::pocket_strategy::PocketStrategy;
use super::ring::dist;

/// How far (mm) the area's own boundary may stand from the start side
/// and still be read as on it: a micron, well above Clipper's nanometre
/// lattice and below its 5 um arc flats.
const SIDE_EPS: f64 = 1e-3;

/// Pieces shorter than this (mm) are dropped: a stub where a wall meets
/// the start side, which the neighbouring pieces' discs already cover.
const MIN_PIECE: f64 = 1e-2;

/// The ring plan of one area for `side`, with the warnings it owes the
/// op page. Auto, a row strategy, and an Inside with no hole to start
/// from keep the planner's old plan byte for byte.
pub(super) fn ring_plan(
    region: &Region,
    center: &Region,
    p: &PocketParams,
    strategy: PocketStrategy,
    side: &StartSide,
) -> Result<(Vec<Vec<Point2>>, Vec<String>), GenError> {
    let rings = matches!(strategy, PocketStrategy::Waterline | PocketStrategy::OffsetRings | PocketStrategy::Adaptive);
    if side.start != PassStart::Auto && rings {
        let off = p.tool_diameter * 0.5 + p.stock_to_leave;
        if let Some(pieces) = level_pieces(region, center, off, p.tiling_step(), p.climb, side.start)? {
            let plan = order(pieces, center, p.tool_diameter * 0.5, side)?;
            return Ok((plan.passes, plan.notes));
        }
    }
    Ok((plan_passes(center, p.tiling_step(), p.climb, strategy)?, Vec::new()))
}

/// The pieces of every ring of `center` (the cutter-centre area of
/// `region` eroded by `off`), each walked so that the uncut side is on
/// the right for `climb` (the core-first planner's sense). `None` when
/// the start side does not exist (Inside on an area with no hole).
pub(super) fn level_pieces(
    region: &Region,
    center: &Region,
    off: f64,
    step: f64,
    climb: bool,
    start: PassStart,
) -> Result<Option<Vec<Piece>>, GenError> {
    let Some((lo, hi)) = center.bounds() else {
        return Err(GenError::Empty("empty pocket region".into()));
    };
    if !(step.is_finite() && step > 0.0 && off.is_finite()) {
        return Err(GenError::Invalid("the ring step must be positive and finite".into()));
    }
    let cap = (dist(lo, hi) / step + 10.0).max(16.0) as u32;
    let filled = Region::new(region.loops().iter().filter(|l| l.is_ccw()).cloned().collect())?;
    let mut out = Vec::new();
    let converged = match start {
        PassStart::Auto => return Ok(None),
        PassStart::Outside => {
            // The distance from the outer edge: the filled area eroded.
            let fc = filled.offset(-off)?;
            sides(center, &fc.offset(-SIDE_EPS)?, false, climb, &mut out)?;
            let mut done = false;
            for j in 1..=cap {
                let g = fc.offset(-f64::from(j) * step)?;
                if g.is_empty() {
                    done = true;
                    break;
                }
                // `g` lies on the left of its loops and is the uncut side.
                curves(center, &g, j, climb, &mut out)?;
            }
            done
        }
        PassStart::Inside => {
            let holes = filled.difference(region)?;
            if holes.area().abs() <= 1e-9 {
                return Ok(None);
            }
            // The distance from the holes: the holes' centre keep-out grown.
            let hc = holes.offset(off)?;
            sides(center, &hc.offset(SIDE_EPS)?, true, climb, &mut out)?;
            let mut done = false;
            for j in 1..=cap {
                let g = hc.offset(f64::from(j) * step)?;
                // `g` lies on the left of its loops and is the cut side.
                curves(center, &g, j, !climb, &mut out)?;
                if center.difference(&g)?.is_empty() {
                    done = true;
                    break;
                }
            }
            done
        }
    };
    if !converged {
        return Err(GenError::Invalid("pass start rings did not converge; check the stepover".into()));
    }
    share_ends(&mut out);
    Ok(Some(out))
}

/// The area's own boundary split by `start_side` (inside it for Inside,
/// outside it for Outside): ring 0 on the start side, the walls (level
/// [`WALL`]) elsewhere. The area lies on the left of its loops, and it
/// is the uncut side of ring 0 and the cut side of a wall.
fn sides(center: &Region, start_side: &Region, start_inside: bool, climb: bool, out: &mut Vec<Piece>) -> Result<(), GenError> {
    for l in center.loops() {
        let ring = walk(l.points(), climb);
        push(out, 0, &ring, start_side.clip_ring(&ring, start_inside)?);
        // The walls from the same walk, then turned: where a wall meets
        // ring 0 both carry the very same point.
        let back = walk(&ring, true);
        let wall = match start_side.clip_ring(&ring, !start_inside)? {
            RingClip::Pieces(ps) => RingClip::Pieces(ps.into_iter().rev().map(|p| walk(&p, true)).collect()),
            whole => whole,
        };
        push(out, WALL, &back, wall);
    }
    Ok(())
}

/// Each end of a ring piece that stands on the area's boundary becomes
/// a vertex of the boundary walk it stands on (ring 0 or a wall): the
/// two then share the point exactly, so the verifier's crossing law
/// reads a touch there, never a crossing.
fn share_ends(out: &mut [Piece]) {
    let ends: Vec<Point2> = out
        .iter()
        .filter(|p| !p.closed && p.level != 0 && p.level != WALL)
        .flat_map(|p| p.pts.first().into_iter().chain(p.pts.last()).copied().collect::<Vec<_>>())
        .collect();
    for p in out.iter_mut().filter(|p| p.level == 0 || p.level == WALL) {
        for &e in &ends {
            let at = p.pts.windows(2).position(|w| w[0] != e && w[1] != e && on_segment(e, w[0], w[1]));
            if let Some(i) = at {
                p.pts.insert(i + 1, e);
            }
        }
    }
}

/// Is `e` on the segment `a -> b`, strictly between its ends, within a
/// nanometre?
fn on_segment(e: Point2, a: Point2, b: Point2) -> bool {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let dd = dx * dx + dy * dy;
    if !(dd > 0.0 && dd.is_finite()) {
        return false;
    }
    let t = ((e.x - a.x) * dx + (e.y - a.y) * dy) / dd;
    t > 0.0 && t < 1.0 && dist(e, Point2::new(a.x + t * dx, a.y + t * dy)) <= 1e-6
}

/// Ring `level`: the loops of `g` clipped to `center`, reversed when
/// `reverse`.
fn curves(center: &Region, g: &Region, level: u32, reverse: bool, out: &mut Vec<Piece>) -> Result<(), GenError> {
    for l in g.loops() {
        let ring = walk(l.points(), reverse);
        push(out, level, &ring, center.clip_ring(&ring, true)?);
    }
    Ok(())
}

fn walk(pts: &[Point2], reverse: bool) -> Vec<Point2> {
    if reverse { pts.iter().rev().copied().collect() } else { pts.to_vec() }
}

fn push(out: &mut Vec<Piece>, level: u32, ring: &[Point2], clip: RingClip) {
    match clip {
        RingClip::Whole if ring.len() >= 2 => out.push(Piece { level, pts: ring.to_vec(), closed: true }),
        RingClip::Whole => {}
        RingClip::Pieces(ps) => {
            for pts in ps {
                let len: f64 = pts.windows(2).map(|w| dist(w[0], w[1])).sum();
                if pts.len() >= 2 && len >= MIN_PIECE {
                    out.push(Piece { level, pts, closed: false });
                }
            }
        }
    }
}
