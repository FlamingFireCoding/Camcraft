//! The chain-set law of the Rotary Flow kernel (#2829, the third
//! #2804 slice): the picked guide chains read as ONE ordered set,
//! never as loose pairs. The law states three things. A single chain
//! machines the full revolution: the kernel morphs it to its own
//! next lap, one whole turn on in the unwrap's own theta, and the
//! developed band between a chain and its lap is the full-revolution
//! band a single-start screw or an auger rides. A set of three or
//! more machines one band per adjacent pair, and the SHARED
//! boundary chain is cut once - the band above already walked it, so
//! the band below starts at its first interior pass. And a set that
//! cannot tile is refused by name: chains that meet or cross, and a
//! band that doubles back through the chain before it.
//!
//! Provenance: the closest points of two segments from Ericson,
//! "Real-Time Collision Detection" 5.1 (the guard's own citation,
//! spelled here so the refusal can name the meet's own spot); the
//! lap morph, the developed-direction law and the shared-boundary
//! pass are this codebase's own, written from published 4-axis
//! flow-line practice. No vendor material consulted.

use crate::geometry::Point3;
use crate::machining2d::GenError;

use super::rotary_flow_unwrap::{lerp_at, Unwrapped, TAU};

/// Most flow passes one band may take: a band that needs more is a
/// wrong pick or a wrong stepover, refused by name.
pub(super) const MAX_PASSES: usize = 4096;
/// Most segment pairs the meet law walks before it refuses by name:
/// past it the proof would outlive the generation.
const MEET_PAIR_CAP: usize = 1 << 22;

/// The track of the chain's own next lap: the same walk advanced one
/// whole turn in theta, every other coordinate held. The morph
/// behind the single-chain case - the development between a chain
/// and its lap is exactly one full revolution wide.
pub(super) fn lap_of(a: &Unwrapped) -> Unwrapped {
    Unwrapped {
        th: a.th.iter().map(|t| t + TAU).collect(),
        v: a.v.clone(),
        rho: a.rho.clone(),
        u: a.u.clone(),
    }
}

/// `p` clamped onto the segment `a -> b` (the guard's own clamp
/// law, the pt_seg2 core spelled for the point it lands on).
fn clamp_to(p: Point3, a: Point3, b: Point3) -> Point3 {
    let ab = b - a;
    let l = ab.dot(ab);
    let t = if l > 0.0 {
        ((p - a).dot(ab) / l).clamp(0.0, 1.0)
    } else {
        0.0
    };
    Point3::new(a.x + ab.x * t, a.y + ab.y * t, a.z + ab.z * t)
}

/// The closest pair of the segments `p1 -> q1` and `p2 -> q2`: the
/// distance, and the point of the FIRST segment that reaches it.
/// Ericson 5.1's clamped solve over the four endpoint candidates -
/// the guard's own `seg_seg` law, spelled here so a meet can name
/// its own spot. The candidates alone cover the parallel and
/// collinear cases the interior solve's denominator cannot.
fn seg_seg_at(p1: Point3, q1: Point3, p2: Point3, q2: Point3) -> (f64, Point3) {
    let mut best2 = f64::INFINITY;
    let mut wit = p1;
    // Each endpoint of one segment against the other segment; the
    // witness stays on the FIRST chain.
    for (p, w) in [(p1, p1), (q1, q1)] {
        let c = clamp_to(p, p2, q2);
        let d = p - c;
        let dd = d.dot(d);
        if dd < best2 {
            best2 = dd;
            wit = w;
        }
    }
    for p in [p2, q2] {
        let c = clamp_to(p, p1, q1);
        let d = p - c;
        let dd = d.dot(d);
        if dd < best2 {
            best2 = dd;
            wit = c;
        }
    }
    // The interior solve, when the segments are not near-parallel.
    let (d1, d2, r) = (q1 - p1, q2 - p2, p1 - p2);
    let (a, e, f) = (d1.dot(d1), d2.dot(d2), d2.dot(r));
    let (b, c) = (d1.dot(d2), d1.dot(r));
    let den = a * e - b * b;
    if a > 1e-30 && e > 1e-30 && den > 1e-18 * a * e {
        let s = ((b * f - c * e) / den).clamp(0.0, 1.0);
        let t = ((b * s + f) / e).clamp(0.0, 1.0);
        let s = ((b * t - c) / a).clamp(0.0, 1.0);
        let x = Point3::new(p1.x + d1.x * s, p1.y + d1.y * s, p1.z + d1.z * s);
        let y = Point3::new(p2.x + d2.x * t, p2.y + d2.y * t, p2.z + d2.z * t);
        let d = x - y;
        let dd = d.dot(d);
        if dd < best2 {
            best2 = dd;
            wit = x;
        }
    }
    (best2.sqrt(), wit)
}

/// Do the two chains meet or cross, within the corner-lattice dust?
/// Refused by name at the meet's own spot: a band whose boundary
/// chains touch pinches to a point there, and one whose chains cross
/// cuts its ground twice. Adjacent picks only - the pairs the kernel
/// blends.
pub(super) fn meet_or_cross(
    a: &[Point3],
    b: &[Point3],
    k: usize,
    eps2: f64,
) -> Result<(), GenError> {
    let (na, nb) = (a.len().saturating_sub(1), b.len().saturating_sub(1));
    if na == 0 || nb == 0 {
        return Ok(()); // clean_chain already refused the short chain
    }
    if na * nb > MEET_PAIR_CAP {
        return Err(GenError::Invalid(format!(
            "Rotary Flow: guide chains {} and {} hold too many points to prove that they do \
             not meet - simplify the model edges, or pick shorter chains",
            k + 1,
            k + 2
        )));
    }
    for (i, sa) in a.windows(2).enumerate() {
        for sb in b.windows(2) {
            let (d, w) = seg_seg_at(sa[0], sa[1], sb[0], sb[1]);
            if d * d <= eps2 {
                return Err(GenError::invalid_at(
                    format!(
                        "Rotary Flow: guide chains {} and {} meet or cross each other at \
                         segment {} of chain {} - pick edge chains that bound the band without \
                         meeting",
                        k + 1,
                        k + 2,
                        i + 1,
                        k + 1
                    ),
                    w,
                ));
            }
        }
    }
    Ok(())
}

/// The pair's developed offsets at the matched places of the same
/// grid the width reads: the fixed samples and both tracks' own
/// knots. Each offset is `(R_ref * dtheta, dv)`, the vector the flow
/// passes divide.
fn offsets(a: &Unwrapped, b: &Unwrapped, r_ref: f64, samples: usize) -> Vec<(f64, f64)> {
    let n = samples.max(2);
    let fixed = (0..n).map(|i| i as f64 / (n - 1) as f64);
    fixed
        .chain(a.u.iter().copied())
        .chain(b.u.iter().copied())
        .map(|x| {
            (
                r_ref * (lerp_at(&b.u, &b.th, x) - lerp_at(&a.u, &a.th, x)),
                lerp_at(&b.u, &b.v, x) - lerp_at(&a.u, &a.v, x),
            )
        })
        .collect()
}

/// The pair's band shape: the mean direction the band advances in
/// the development, and the pass count its width divides into.
/// The mean direction feeds the set law: consecutive bands whose
/// mean offsets oppose each other double back through the chain
/// they share. A pair whose offset merely pinches through zero at
/// one place (a lens band) is legal - the #2824 box band pins it -
/// so the direction is read as a mean, never as a sign law.
pub(super) fn band_shape(
    a: &Unwrapped,
    b: &Unwrapped,
    r_ref: f64,
    width: f64,
    step: f64,
    samples: usize,
) -> Result<((f64, f64), usize), GenError> {
    let os = offsets(a, b, r_ref, samples);
    let mut dir = (0.0f64, 0.0f64);
    for o in &os {
        dir.0 += o.0;
        dir.1 += o.1;
    }
    let n = os.len().max(1) as f64;
    Ok(((dir.0 / n, dir.1 / n), pass_count(width, step)?))
}

/// Consecutive bands must not double back: the mean offset of the
/// band below, read against the band above's, opposes it. The set is
/// out of band order, or the middle chain is the wrong edge - never
/// machined as ground the band above already cut.
pub(super) fn refuse_backtrack(prev: (f64, f64), now: (f64, f64), k: usize) -> Result<(), GenError> {
    if prev.0 * now.0 + prev.1 * now.1 < 0.0 {
        return Err(GenError::Invalid(format!(
            "Rotary Flow: the bands between chains {}-{} and {}-{} advance in opposite \
             directions - pick the guide chains in band order along the surface",
            k,
            k + 1,
            k + 1,
            k + 2
        )));
    }
    Ok(())
}

/// The pass count a band's developed width needs at `step`: one pass
/// at each edge, one per `step` in between; the ceiling is the
/// runaway guard, refused by name.
pub(super) fn pass_count(width: f64, step: f64) -> Result<usize, GenError> {
    if !(step.is_finite() && step > 0.0) {
        return Err(GenError::Invalid(
            "Rotary Flow: the scallop law collapsed to a zero stepover - check the tolerance"
                .into(),
        ));
    }
    // CEILING: the last span may be short, never long - a pass a
    // stepover and a half wide is a broken promise.
    let spans = (width / step).ceil();
    if !spans.is_finite() || spans < 0.0 {
        return Err(GenError::Invalid(format!(
            "Rotary Flow: the band's developed width {width:.3} mm does not divide at a \
             {step:.3} mm stepover"
        )));
    }
    if spans + 1.0 > MAX_PASSES as f64 {
        return Err(GenError::Invalid(format!(
            "Rotary Flow: the band needs {} passes at a {step:.3} mm stepover - more than the \
             {MAX_PASSES} pass ceiling; widen the stepover, or pick a narrower band",
            spans + 1.0
        )));
    }
    Ok((spans as usize + 1).max(2))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p3(x: f64, y: f64, z: f64) -> Point3 {
        Point3::new(x, y, z)
    }

    #[test]
    fn the_lap_advances_one_whole_turn_and_holds_everything_else() {
        let a = Unwrapped {
            th: vec![0.0, 1.0, 2.5],
            v: vec![1.0, 2.0, 3.0],
            rho: vec![8.0, 8.5, 9.0],
            u: vec![0.0, 0.4, 1.0],
        };
        let lap = lap_of(&a);
        assert_eq!(lap.th, vec![TAU, 1.0 + TAU, 2.5 + TAU]);
        assert_eq!(lap.v, a.v);
        assert_eq!(lap.rho, a.rho);
        assert_eq!(lap.u, a.u);
    }

    #[test]
    fn segments_that_cross_answer_zero_at_the_crossing() {
        // Two segments crossing at the origin: the closest pair is
        // the crossing itself, on the first segment.
        let (d, w) = seg_seg_at(p3(0.0, -1.0, 0.0), p3(0.0, 1.0, 0.0), p3(-1.0, 0.0, 0.0), p3(1.0, 0.0, 0.0));
        assert!(d < 1e-12, "{d}");
        assert!(w.x.abs() < 1e-12 && w.y.abs() < 1e-12, "{w:?}");
        // Parallel neighbours keep their gap.
        let (d, _) = seg_seg_at(p3(0.0, 0.0, 0.0), p3(1.0, 0.0, 0.0), p3(0.0, 1.0, 0.0), p3(1.0, 1.0, 0.0));
        assert!((d - 1.0).abs() < 1e-12, "{d}");
        // Collinear neighbours that overlap: the endpoint candidates
        // must witness the overlap, not the gap past its ends.
        let (d, w) = seg_seg_at(p3(0.0, 0.0, 0.0), p3(10.0, 0.0, 0.0), p3(3.0, 0.0, 0.0), p3(7.0, 0.0, 0.0));
        assert!(d < 1e-12, "{d}");
        assert!(w.x >= 3.0 && w.x <= 7.0, "the witness {w:?} is off the overlap");
    }

    #[test]
    fn the_meet_law_refuses_at_the_meet_and_passes_clean_neighbours() {
        let a = [p3(0.0, 0.0, 0.0), p3(10.0, 0.0, 0.0)];
        // A chain that meets the first one at its middle.
        let meet = [p3(5.0, -1.0, 0.0), p3(5.0, 1.0, 0.0)];
        let e = meet_or_cross(&a, &meet, 0, 1e-12).unwrap_err();
        assert!(format!("{e}").contains("meet or cross"), "{e}");
        assert!(e.location().is_some_and(|q| (q.x - 5.0).abs() < 1e-9), "{e:?}");
        // A clean neighbour passes.
        let clean = [p3(0.0, 3.0, 0.0), p3(10.0, 3.0, 0.0)];
        assert!(meet_or_cross(&a, &clean, 0, 1e-12).is_ok());
    }

    #[test]
    fn the_shape_law_reads_a_mean_direction_and_divides_the_width() {
        // A lens band (the offset pinches through zero at the
        // middle) stays legal - the #2824 box band pins it - so the
        // direction is read as a mean: here it cancels to zero and
        // never refuses anything.
        let a = Unwrapped {
            th: vec![0.0, 1.0],
            v: vec![0.0, 0.0],
            rho: vec![8.0, 8.0],
            u: vec![0.0, 1.0],
        };
        let lens = Unwrapped {
            th: vec![-0.3, 1.3],
            v: vec![0.0, 0.0],
            rho: vec![8.0, 8.0],
            u: vec![0.0, 1.0],
        };
        let (dir, count) = band_shape(&a, &lens, 8.0, 10.0, 1.0, 17).unwrap();
        assert!(dir.0.abs() < 1e-9, "{dir:?}");
        assert_eq!(count, 11); // a 10 mm band at a 1 mm stepover
        // A band that advances one way reads its direction.
        let on_side = Unwrapped {
            th: vec![0.3, 1.3],
            v: vec![0.0, 0.0],
            rho: vec![8.0, 8.0],
            u: vec![0.0, 1.0],
        };
        let (dir, _) = band_shape(&a, &on_side, 8.0, 10.0, 1.0, 17).unwrap();
        assert!((dir.0 - 8.0 * 0.3).abs() < 1e-9, "{dir:?}");
    }

    #[test]
    fn the_backtrack_law_refuses_bands_that_double_back() {
        let e = refuse_backtrack((0.0, 2.0), (0.0, -1.0), 1).unwrap_err();
        let msg = format!("{e}");
        assert!(
            msg.contains("advance in opposite directions") && msg.contains("chains 1-2 and 2-3"),
            "{msg}"
        );
        assert!(refuse_backtrack((0.0, 2.0), (0.0, 8.0), 1).is_ok());
        // A corner turn is not a doubling back: the offsets stand
        // perpendicular, the bands tile around the corner.
        assert!(refuse_backtrack((2.0, 0.0), (0.0, 2.0), 1).is_ok());
    }
}
