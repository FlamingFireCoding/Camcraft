//! A pass restart inside a 3-D ladder level (#3670): the op's own entry,
//! the ramp along the pass, or a named plunge, read by the centre-cutting
//! plunge law (`plunge_law`).
//!
//! Provenance: standard CNC entry motions (helix, ramp, plunge) per public
//! G-code manuals and machining handbooks; the cleared-column test is this
//! crate's own geometry. No vendor material.

use super::descent::{Ask, descend, fallback, mat_top, push_zig3, zig_fallback};
use super::level_ctx::LevelCtx;
use super::params::Entry3D;

use crate::cl::{MoveKind, PathBuilder};
use crate::geo2d::Point2;
use crate::plunge_law;

/// A pass start inside a level, reached by a hop (#3670): a straight
/// plunge stands only over a cleared column, or when the user's entry is
/// itself the plunge on a centre-cutting tool. Into solid, a tool that is
/// not centre-cutting takes [`fallback`] (M11.3). Any other tool takes
/// the op's own `entry` (helix, ramp, arc) where that descent posts no
/// plunge into solid itself, else the ramp along the pass, else the
/// plunge, named. `None`: plunge as before; `Some(dive start)`: posted.
pub(super) fn reenter(b: &mut PathBuilder, lvl: &LevelCtx, a: &Ask, entry: Entry3D) -> Option<Option<Point2>> {
    let (r, top) = (lvl.p.tool_diameter * 0.5, mat_top(lvl, a));
    if !plunge_law::into_solid(b, a.entry, lvl.tip(a.entry, 0.0), top, r) {
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
                    && plunge_law::into_solid_at(&mv[..i], Point2::new(m.to.x, m.to.y), m.to.z, top, r)
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
