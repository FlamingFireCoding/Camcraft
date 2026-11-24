//! A pass restart inside a 3-D ladder level (#3670): the op's own entry,
//! the ramp along the pass, or a named plunge, with a full-disc clearance
//! proof separate from the non-centre-cutting plunge law (`plunge_law`).
//!
//! Provenance: standard CNC entry motions (helix, ramp, plunge) per public
//! G-code manuals and machining handbooks; the cleared-column test is this
//! crate's own geometry. No vendor material.

use super::descent::{Ask, descend, fallback, mat_top, push_zig3, zig_fallback};
use super::level_ctx::LevelCtx;
use super::params::Entry3D;

use crate::cl::{Move, MoveKind, PathBuilder};
use crate::geo2d::Point2;

/// A straight cut by this same tool clears its full disc at every point
/// of its centre line. Require the restart axis on that line, rather
/// than merely near it (the core law permits an offset of half a radius).
/// This sufficient proof can miss clearance made by several cuts. That
/// costs an entry, never a plunge into stock. Arcs do not prove a line:
/// their tessellation chords can lie outside the actual cleared sweep.
fn disc_into_solid(moves: &[Move], xy: Point2, z: f64, top: f64) -> bool {
    if ![xy.x, xy.y, z, top].iter().all(|v| v.is_finite()) {
        return true;
    }
    if z >= top - 1e-6 {
        return false;
    }
    !moves.iter().rev().any(|m| {
        if m.kind == MoveKind::Rapid
            || m.arc.is_some()
            || !m.from.z.is_finite()
            || !m.to.z.is_finite()
            || m.from.z.max(m.to.z) > z + 1e-6
        {
            return false;
        }
        let (ax, ay) = (xy.x - m.from.x, xy.y - m.from.y);
        let (dx, dy) = (m.to.x - m.from.x, m.to.y - m.from.y);
        let len2 = dx * dx + dy * dy;
        let along = ax * dx + ay * dy;
        // No XY slack: an offset centre line does not contain this disc.
        // Explicit endpoint equality also covers a stationary plunge.
        (xy.x == m.from.x && xy.y == m.from.y)
            || (xy.x == m.to.x && xy.y == m.to.y)
            || (len2.is_finite() && len2 > 0.0 && along >= 0.0 && along <= len2 && ax * dy == ay * dx)
    })
}

/// A pass start inside a level, reached by a hop (#3670): a straight
/// plunge stands only over a cleared column, or when the user's entry is
/// itself the plunge on a centre-cutting tool. Into solid, a tool that is
/// not centre-cutting takes [`fallback`] (M11.3). Any other tool takes
/// the op's own `entry` (helix, ramp, arc) where that descent posts no
/// plunge into solid itself, else the ramp along the pass, else the
/// plunge, named. `None`: plunge as before; `Some(dive start)`: posted.
pub(super) fn reenter(b: &mut PathBuilder, lvl: &LevelCtx, a: &Ask, entry: Entry3D) -> Option<Option<Point2>> {
    let top = mat_top(lvl, a);
    if !disc_into_solid(b.moves(), a.entry, lvl.tip(a.entry, 0.0), top) {
        return None;
    }
    if !lvl.p.links.approach.center_cutting {
        return Some(fallback(b, lvl, a, "a pass start inside the level", false));
    }
    if entry == Entry3D::Plunge {
        return None;
    }
    // Each candidate runs on a copy and is kept only if it is clean: no
    // plunge into solid (a helix that fits only far off the pass start
    // dives again there) and no crossing of a move already posted (the
    // verifier's Crossing law, which a lifted shell's legs can meet).
    let n0 = b.moves().len();
    let clean = |t: &PathBuilder| {
        let mv = t.moves();
        !crate::machining2d::moves_cross_prior(&mv[n0..], &mv[..n0])
            && !(n0..mv.len()).any(|i| {
                let m = &mv[i];
                m.kind == MoveKind::Plunge
                    && disc_into_solid(&mv[..i], Point2::new(m.to.x, m.to.y), m.to.z, top)
            })
    };
    let mut trial = b.clone();
    let dove = descend(&mut trial, lvl, &Ask { arm: entry, ..*a });
    if clean(&trial) {
        *b = trial;
        return Some(dove);
    }
    if let Ok((pre, n, zf)) = zig_fallback(lvl, a) {
        let mut trial = b.clone();
        push_zig3(&mut trial, lvl, a, &pre, n, zf);
        if clean(&trial) {
            *b = trial;
            b.note_once("entry: a pass start inside the level has no room for the entry - ramp along the pass instead");
            return Some(None);
        }
    }
    b.note_once("entry: a pass start inside the level has no room for the entry or a ramp - plunged there");
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point3;

    #[test]
    fn cleared_core_does_not_prove_the_outer_disc_clear() {
        let cut = Move::new(MoveKind::Feed, Point3::new(-5.0, 0.0, 0.0), Point3::new(5.0, 0.0, 0.0), Some(100.0));
        let restart = Point2::new(0.0, 1.0);
        assert!(!crate::plunge_law::into_solid_at(&[cut], restart, 0.0, 2.0, 3.0), "the core law still permits this offset");
        // (0, 3.5) lies 2.5 mm from the restart axis, but 3.5 mm
        // from the earlier cut: the outer disc contains uncut stock.
        assert!(disc_into_solid(&[cut], restart, 0.0, 2.0));
        assert!(!disc_into_solid(&[cut], Point2::new(0.0, 0.0), 0.0, 2.0));
    }

    #[test]
    fn an_arc_chord_does_not_prove_a_cleared_disc() {
        let arc = Move::new(MoveKind::Feed, Point3::new(-5.0, 0.0, 0.0), Point3::new(5.0, 0.0, 0.0), Some(100.0))
            .with_arc(Point2::new(0.0, 0.0), true);
        assert!(disc_into_solid(&[arc], Point2::new(0.0, 0.0), 0.0, 2.0));
    }
}
