//! Planar pocket pass emission shared by [`super::pocket`] and [`super::rest2d`] (#1816).
//!
//! Provenance: extracted from `pocket.rs` (`pocket_over`) - arc fitting
//! (`fit_arcs` / `fit_proved`), corner filleting (`fillet_corners`), level
//! ladder (`run_level_walks`), crossing-free arc enforcement
//! (`post_crossing_free`), and approach refusal checks. Clean-room, no vendor
//! material consulted.

use crate::cl::{MoveKind, PathBuilder, Toolpath};
use crate::geo2d::{Point2, Region};
use crate::geometry::Point3;

use super::adaptive::Return;
use super::arc_cross::post_crossing_free;
use super::arc_proof::{ProofTally, fit_proved};
use super::arcs::{ARC_FIT_TOL, Seg, fit_arcs};
use super::face::GenError;
use super::fillet::{Legal, fillet_corners};
use super::fillet_tally::{FilletTally, corner_arc_radius, fillet_floor};
use super::levels::{Cert, Motion, cut_levels_under, run_level_walks};
use super::pocket::{PocketParams, PocketStrategy};
use super::pocket_hsm::EntryRoom;
use super::predrilled::PreHole;

/// Pre-planned XY cutter-center passes and entry/link certification areas.
pub(super) struct PocketWalks<'a> {
    pub(super) center: &'a Region,
    pub(super) links_center: &'a Region,
    pub(super) strategy: PocketStrategy,
    pub(super) passes: Vec<Vec<Point2>>,
    pub(super) room: Option<EntryRoom>,
    pub(super) joins: Vec<Option<Return>>,
    pub(super) hold_crossings: bool,
    pub(super) holes: &'a [PreHole],
}

/// Fit, optionally fillet, descend, link, and post `walks` across all Z levels.
pub(super) fn emit_pocket_passes(
    mut b: PathBuilder,
    p: &PocketParams,
    walks: PocketWalks<'_>,
) -> Result<Toolpath, GenError> {
    let PocketWalks {
        center,
        links_center,
        strategy,
        passes,
        room,
        joins,
        hold_crossings,
        holes,
    } = walks;
    // The horizontal arc entry (M11.3) lands on the first pass: a closed
    // first ring starts mid its longest edge, where an arc has room.
    let passes = if p.links.approach.arc_xy && room.is_none() {
        super::descent_xy::reseat_first(passes)
    } else {
        passes
    };
    // #1449: a proved hole re-seats the plan on itself (not the HSM plan's own bore).
    let passes = super::predrilled::reseat(
        passes,
        holes,
        center,
        p.links.approach.predrilled && room.is_none(),
    );
    let zs = cut_levels_under(p.top_z, p.links.approach.material_top, p.bottom_z, p.stepdown)?;
    let fit = match strategy {
        PocketStrategy::Zigzag | PocketStrategy::Auto => None,
        PocketStrategy::Waterline | PocketStrategy::OffsetRings => Some(ARC_FIT_TOL),
        PocketStrategy::Adaptive => Some(ARC_FIT_TOL * 0.25),
    };
    let mut proof = ProofTally::default();
    let mut fit_pass = |pass: &[Point2], tol: f64| -> Vec<Seg> {
        if strategy == PocketStrategy::Adaptive {
            fit_proved(pass, tol, center, &mut proof)
        } else {
            fit_arcs(pass, tol)
        }
    };
    let helix_r = room.as_ref().map_or(p.helix_radius, |e| e.helix_r);
    let motion = Motion::new(p.clearance_z, p.retract_z, p.feed, p.plunge_feed)
        .with_entry(p.entry, helix_r)
        .with_arc_radius(p.tool_diameter)
        .with_links(p.links, helix_r)
        .with_top(p.top_z)
        .with_xy_arc(p.links.approach.arc_xy)
        .with_microlift(joins)
        .with_holes(holes);
    let runs: Option<Vec<Vec<Seg>>> = if p.blend_turns {
        let legal = Legal { region: center, inside: true, air_left: None };
        let rho = corner_arc_radius(p.blend_radius, p.tool_diameter).map_err(GenError::Invalid)?;
        let mut tally = FilletTally::default();
        let runs = passes
            .iter()
            .map(|pass| {
                let Some((first, rest)) = pass.split_first() else {
                    return Vec::new();
                };
                let walk = match fit {
                    Some(tol) => fit_pass(pass, tol),
                    None => vec![Seg::Line(rest.to_vec())],
                };
                let (walk, t) = fillet_corners(&walk, *first, rho, ARC_FIT_TOL, &legal);
                tally.add(t);
                walk
            })
            .collect();
        for n in tally.notes("pass", fillet_floor(ARC_FIT_TOL)) {
            b.note(n);
        }
        Some(runs)
    } else {
        fit.map(|tol| passes.iter().map(|p| fit_pass(p, tol)).collect())
    };
    let cert = match &room {
        Some(e) => Cert::Stadium { a: e.seed, b: e.seed, h: e.clear, tess: &e.tess },
        None => Cert::Region(center),
    };
    let post = |b: &mut PathBuilder, runs: Option<&[Vec<Seg>]>| {
        run_level_walks(
            b,
            &passes,
            &zs,
            &motion,
            runs,
            Some(&cert),
            Some(&Cert::Region(links_center)),
        );
        let cur = b.at();
        b.push(MoveKind::Rapid, Point3::new(cur.x, cur.y, p.clearance_z), None);
    };
    let mut path = match runs {
        Some(runs) if strategy == PocketStrategy::Adaptive || hold_crossings => {
            post_crossing_free(b, &passes, runs, &mut proof, post)
        }
        runs => {
            post(&mut b, runs.as_deref());
            b.finish()
        }
    };
    if let Some(n) = proof.note() {
        path.notes.push(n);
    }
    if let Some(why) = crate::approach::refusal(&path) {
        return Err(GenError::Invalid(why).with_ghost(&path));
    }
    if !path.all_finite() {
        return Err(GenError::Invalid("path left the finite horizon".into()));
    }
    Ok(path)
}
