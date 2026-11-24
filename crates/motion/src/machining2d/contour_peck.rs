//! A contour dive below the levels above (#3678): the floor that the
//! posted cuts left under the full tool disc, and the peck that keeps
//! each feed down to one stepdown below it.
//!
//! Provenance: the peck (step) plunge of public G-code manuals (the G83
//! deep-hole cycle of RS-274 / ISO 6983: feed one step, retract, feed the
//! next) and the stepdown rule of machining handbooks; the floor reading
//! is this crate's own geometry. No vendor material consulted.

use crate::cl::{Move, MoveKind, PathBuilder};
use crate::geo2d::Point2;
use crate::geometry::Point3;

use super::contour_params::ContourParams;

/// Height slack: a cut on the dive's own plane counts as at its depth.
const Z_EPS: f64 = 1e-6;
/// Chord tolerance the floor reads arcs at (mm).
const CHORD_TOL: f64 = 0.001;
/// Rings and points per ring the disc is sampled at, out to the rim.
const RINGS: u32 = 6;
const PER_RING: u32 = 72;
/// At most this many pecks: a dive that needs more is a bad stepdown.
const MAX_PECKS: usize = 4096;

/// The highest floor under the tool disc of radius `r` at `xy`: at each
/// sample point the lowest Z of a posted cut whose swept disc holds it -
/// a feed at one height, or a straight plunge column - else `uncut`.
#[must_use]
pub(super) fn disc_floor(moves: &[Move], xy: Point2, r: f64, uncut: f64) -> f64 {
    if !(r.is_finite() && r > 0.0) {
        return uncut;
    }
    // Only the cuts within reach of the disc can hold a point of it.
    let near: Vec<(f64, Vec<(Point2, Point2)>)> = moves
        .iter()
        .filter(|m| m.kind != MoveKind::Rapid && m.to.z < uncut - Z_EPS)
        .filter_map(|m| {
            let flat = (m.from.z - m.to.z).abs() <= Z_EPS;
            let column = m.from.x == m.to.x && m.from.y == m.to.y && m.to.z < m.from.z;
            let chords = if flat {
                m.chords(CHORD_TOL)
            } else if column {
                let c = Point2::new(m.to.x, m.to.y);
                vec![(c, c)]
            } else {
                return None;
            };
            chords.iter().any(|&(a, b)| seg_dist(xy, a, b) <= 2.0 * r + Z_EPS).then_some((m.to.z, chords))
        })
        .collect();
    let floor = |q: Point2| {
        near.iter()
            .filter(|(_, ch)| ch.iter().any(|&(a, b)| seg_dist(q, a, b) <= r + Z_EPS))
            .map(|(z, _)| *z)
            .fold(uncut, f64::min)
    };
    let mut top = floor(xy);
    for ring in 1..=RINGS {
        let rr = r * f64::from(ring) / f64::from(RINGS) * 0.999;
        for k in 0..PER_RING {
            let t = std::f64::consts::TAU * f64::from(k) / f64::from(PER_RING);
            top = top.max(floor(Point2::new(xy.x + rr * t.cos(), xy.y + rr * t.sin())));
        }
    }
    top
}

/// The depths a dive at `xy` down to `z` stops at: one stepdown apart
/// below the disc floor, the last one `z`. A dive within one stepdown of
/// the floor (or with no `uncut` floor: a ring's first level) is `[z]`.
#[must_use]
pub(super) fn peck_depths(moves: &[Move], xy: Point2, z: f64, r: f64, step: f64, uncut: Option<f64>) -> Vec<f64> {
    let Some(uncut) = uncut else { return vec![z] };
    if !(step.is_finite() && step > 0.0 && z.is_finite()) {
        return vec![z];
    }
    let floor = disc_floor(moves, xy, r, uncut);
    let mut out = Vec::new();
    let mut d = floor - step;
    while d > z + Z_EPS && out.len() < MAX_PECKS {
        out.push(d);
        d -= step;
    }
    out.push(z);
    out
}

/// The floor with no cut under the disc, for a dive on level `rzi` of a
/// ring whose levels are `rz`: the material's top, or one stepdown over
/// the ring's first level where that is lower (a shelf band). `None` on
/// the ring's first level: that dive never pecks.
#[must_use]
pub(super) fn uncut_floor(p: &ContourParams, rz: &[f64], rzi: usize) -> Option<f64> {
    let first = *rz.first()?;
    (rzi > 0).then(|| p.entry_top().min(first + p.stepdown))
}

/// Where the tool waits between pecks: the plunge height over the
/// material, held under the retract plane (`feed_down`'s own rapid plane).
#[must_use]
fn retract_plane(p: &ContourParams, hop_z: f64) -> f64 {
    let top = p.entry_top();
    (top + p.links.approach.plunge_height.max(0.0)).min(hop_z.max(top))
}

/// The dive the tool stands over at `xy`: the first feed down by `first`
/// (the caller's plunge to the depth it is given), then the pecks to `z`.
#[allow(clippy::too_many_arguments)]
pub(super) fn dive(
    b: &mut PathBuilder,
    p: &ContourParams,
    xy: Point2,
    z: f64,
    uncut: Option<f64>,
    hop_z: f64,
    feed: f64,
    first: impl FnOnce(&mut PathBuilder, f64),
) {
    let depths = peck_depths(b.moves(), xy, z, p.tool_diameter * 0.5, p.stepdown, uncut);
    first(b, depths.first().copied().unwrap_or(z));
    push_pecks(b, xy, &depths, retract_plane(p, hop_z), p.links.approach.plunge_height, feed);
}

/// The pecks after the first feed down (`depths[0]`, already posted): up
/// to `retract` - air over the material - by rapid, down by rapid to the
/// plunge height over the last depth, then the feed one stepdown deeper.
/// A pecked dive is said in one note for the path.
fn push_pecks(b: &mut PathBuilder, xy: Point2, depths: &[f64], retract: f64, h: f64, feed: f64) {
    let Some((&first, rest)) = depths.split_first() else { return };
    let mut last = first;
    for &d in rest {
        b.push(MoveKind::Rapid, Point3::new(xy.x, xy.y, retract), None);
        let back = last + h.max(0.0);
        if h > 0.0 && back < retract {
            b.push(MoveKind::Rapid, Point3::new(xy.x, xy.y, back), None);
        }
        b.push(MoveKind::Plunge, Point3::new(xy.x, xy.y, d), Some(feed));
        last = d;
    }
    if !rest.is_empty() {
        const SAID: &str = "entry: a dive more than one stepdown below the floor that the levels above left under the tool disc";
        if !b.notes().iter().any(|n| n.starts_with(SAID)) {
            b.note(format!(
                "{SAID} is pecked one stepdown at a time (first at ({:.3}, {:.3}) to Z {:.3}, {} pecks)",
                xy.x,
                xy.y,
                depths[depths.len() - 1],
                depths.len()
            ));
        }
    }
}

/// Distance from `p` to the segment `a -> b`.
fn seg_dist(p: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let dd = dx * dx + dy * dy;
    let t = if dd > 0.0 && dd.is_finite() { (((p.x - a.x) * dx + (p.y - a.y) * dy) / dd).clamp(0.0, 1.0) } else { 0.0 };
    (p.x - a.x - t * dx).hypot(p.y - a.y - t * dy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dive_within_one_stepdown_or_on_a_first_level_does_not_peck() {
        assert_eq!(peck_depths(&[], Point2::new(0.0, 0.0), -4.5, 3.0, 4.5, Some(0.0)), vec![-4.5]);
        assert_eq!(peck_depths(&[], Point2::new(0.0, 0.0), -18.0, 3.0, 4.5, None), vec![-18.0]);
    }

    #[test]
    fn a_dive_into_uncut_stock_pecks_one_stepdown_at_a_time() {
        let d = peck_depths(&[], Point2::new(0.0, 0.0), -18.0, 3.0, 4.5, Some(0.0));
        assert_eq!(d, vec![-4.5, -9.0, -13.5, -18.0]);
    }
}
