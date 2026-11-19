//! The checks on a pcurve the kernel made (#917): its interval mapped onto
//! ours within the edge's reach, and its lift onto the surface within the
//! edge's tolerance. Each refuses by name with how far.
//! Provenance: the maintainer's Solidcraft `SmlibTrims` checks
//! (`crates/smlib/src/pcurves.rs`, reused with his permission). No HSMWorks
//! material was consulted.

use std::cmp::Ordering;

use solidcore::eval::{CurveEval, SurfaceEval};
use solidcore_ops::pcurve::{TrimEdge, TrimFace};
use solidcore_record::brep::Curve2;

/// Samples of a made pcurve's lift checked against its 3D edge, ends
/// included (one more than this).
const LIFT_SAMPLES: usize = 32;

/// A pcurve the kernel made on its edge's interval `theirs`, on ours. The
/// kernel may end an edge at a parameter of its own (a knot its vertex lies
/// at, the end of the part on the face): where each end it chose is within
/// `reach` of ours on the edge's curve (that end's vertex tolerance, or the
/// edge's tolerance as the kernel held it where larger), the pcurve is mapped
/// onto our interval by the affine change of parameter (exact for a
/// B-spline: its knots move). Farther is the kernel trimming the edge,
/// refused with how far.
pub(super) fn on_our_interval(
    pc: Curve2,
    ours: &TrimEdge<'_>,
    theirs: [f64; 2],
    reach: [f64; 2],
) -> Result<Curve2, String> {
    let [a, bb] = ours.interval;
    let [c, d] = theirs;
    if a == c && bb == d {
        return Ok(pc);
    }
    let curve = edge_eval(ours)?;
    let at = |t: f64| {
        curve
            .point(t)
            .ok_or_else(|| format!("the edge's curve has no point at {t}"))
    };
    let shift = [at(a)?.distance(at(c)?), at(bb)?.distance(at(d)?)];
    if !(above(reach[0], shift[0]) || reach[0] == shift[0])
        || !(above(reach[1], shift[1]) || reach[1] == shift[1])
        || !above(d, c)
    {
        return Err(format!(
            "the kernel moved an edge's interval from {:?} to {theirs:?}, its ends {shift:?} mm along its curve (tolerance {reach:?} mm)",
            ours.interval
        ));
    }
    let map = |x: f64| {
        if x == c {
            a
        } else if x == d {
            bb
        } else {
            a + (x - c) * ((bb - a) / (d - c))
        }
    };
    Ok(match pc {
        Curve2::Spline { knots, points } => Curve2::Spline {
            knots: knots.into_iter().map(map).collect(),
            points,
        },
        Curve2::RationalSpline { knots, homogeneous } => Curve2::RationalSpline {
            knots: knots.into_iter().map(map).collect(),
            homogeneous,
        },
    })
}

/// The lift check: `pc` taken onto the face's surface lies within
/// `tolerance` of its 3D edge at every sample of the edge's interval, ends
/// included. A made pcurve that misses is refused with how far - never a
/// face trimmed along a curve that is not its edge.
pub(super) fn lifts_onto(
    face: &TrimFace<'_>,
    ours: &TrimEdge<'_>,
    pc: &Curve2,
    tolerance: f64,
) -> Result<(), String> {
    let surface = solidcore_convert::surface(face.nurbs.unwrap_or(face.surface))
        .map_err(|e| e.to_string())?;
    let on = SurfaceEval::new(&surface).map_err(|e| e.to_string())?;
    let curve = edge_eval(ours)?;
    let [t0, t1] = ours.interval;
    let mut gap = 0.0_f64;
    for i in 0..=LIFT_SAMPLES {
        let t = if i == LIFT_SAMPLES {
            t1
        } else {
            t0 + (t1 - t0) * (i as f64 / LIFT_SAMPLES as f64)
        };
        let q = solidcore_ops::pcurve::eval(pc, t)
            .ok_or_else(|| format!("the made pcurve has no point at {t}"))?;
        let s = on
            .point(q)
            .ok_or_else(|| format!("the surface has no point at {q:?}"))?;
        let p = curve
            .point(t)
            .ok_or_else(|| format!("the edge's curve has no point at {t}"))?;
        gap = gap.max(s.distance(p));
    }
    if !(above(tolerance, gap) || tolerance == gap) {
        return Err(format!(
            "a made pcurve lies {gap} mm off its edge on the surface (tolerance {tolerance} mm)"
        ));
    }
    Ok(())
}

/// `x > y`, false where either is NaN (so `!above` refuses a NaN).
pub(super) fn above(x: f64, y: f64) -> bool {
    x.partial_cmp(&y) == Some(Ordering::Greater)
}

/// The evaluator of the curve an edge is parameterised on.
fn edge_eval(ours: &TrimEdge<'_>) -> Result<CurveEval, String> {
    let curve =
        solidcore_convert::curve(ours.nurbs.unwrap_or(ours.curve)).map_err(|e| e.to_string())?;
    CurveEval::new(&curve).map_err(|e| e.to_string())
}
