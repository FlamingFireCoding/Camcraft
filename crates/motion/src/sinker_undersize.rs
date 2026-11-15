//! Sinker EDM rough and finish with undersize (#543): which pass a burn
//! is, the size law that ties an electrode to its condition and to the
//! finished cavity, and the order law between a rough and its finish.
//!
//! Provenance: general die-sink EDM practice from public machine-tool
//! handbooks - an electrode is made smaller than the finished cavity by
//! a per-side undersize; the cavity it burns is the electrode grown by
//! its condition's spark gap on every side; a rough electrode carries
//! the larger undersize and a finish electrode the smaller, so the
//! finish takes off what the rough left. The refusals and the warning
//! are our own design (#290 D1-D10). No vendor material.
//!
//! The law, per axis, for an electrode `e` wide burning a cavity `c`
//! wide under a condition of undersize `u`, spark gap `g` and XY orbit radius `r`:
//! - the electrode is the one the condition calls for: `e = c - 2u`;
//! - it burns `e + 2(g + r)`, which never exceeds `c` (more would burn past
//!   the finished wall: a gouge);
//! - a finish burns exactly `c` - less leaves the finished cavity
//!   under size, refused by name;
//! - the orbit moves only in XY: the tip stops `u - r` above the finished
//!   floor, so a rough leaves `u - r - g` on the floor as on the walls.

use crate::machining2d::GenError;
use crate::sinker_orbit::{Orbit, OrbitKind};

/// Which pass of a cavity a burn is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SinkerPass {
    /// One electrode, no undersize law (#541): the tip stops one spark
    /// gap above the stated floor.
    #[default]
    Single,
    /// The rough electrode: undersized for its condition, it leaves
    /// stock for the finish on the walls and floor.
    Rough,
    /// The finish electrode: it burns the cavity to its finished size.
    Finish,
}

impl SinkerPass {
    /// The pass in the words a refusal or a page names.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            SinkerPass::Single => "single",
            SinkerPass::Rough => "rough",
            SinkerPass::Finish => "finish",
        }
    }
}

/// Slack under which two sizes count as equal, mm: a micron, below any
/// electrode's stated tolerance.
pub const SIZE_SLACK: f64 = 1e-3;

/// One undersized burn's numbers, mm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Undersize {
    pub pass: SinkerPass,
    /// The finished cavity's X and Y size.
    pub cavity: (f64, f64),
    /// The electrode face's X and Y size.
    pub electrode: (f64, f64),
    /// The condition's spark gap per side.
    pub gap: f64,
    /// The condition's electrode undersize per side.
    pub undersize: f64,
    /// The lateral sweep; absent for a straight plunge.
    pub orbit: Option<Orbit>,
}

/// Judge one undersized burn, by name: the cavity it burns against the
/// finished size, and the electrode against its condition. On success,
/// the stand-off above the finished floor (undersize minus XY orbit radius).
/// A `Single` pass is not judged here and stands one gap off.
///
/// # Errors
/// A garbled number; a burn wider than the finished cavity (it would
/// burn past the wall); a finish that leaves the cavity under size; an
/// electrode that is not the size its condition calls for.
pub fn judge(u: &Undersize, condition: &str) -> Result<f64, GenError> {
    if u.pass == SinkerPass::Single {
        return Ok(u.gap);
    }
    let pass = u.pass.label();
    if let Some(o) = &u.orbit {
        o.check()?;
    }
    let radius = u.orbit.map_or(0.0, |o| o.radius);
    let nums = [u.cavity.0, u.cavity.1, u.electrode.0, u.electrode.1, u.gap, u.undersize];
    if !nums.iter().all(|v| v.is_finite()) {
        return Err(GenError::Invalid(format!("sinker {pass}: a size, gap or undersize is not a finite number")));
    }
    if !(u.cavity.0 > 0.0 && u.cavity.1 > 0.0) {
        return Err(GenError::Invalid(format!(
            "sinker {pass}: the finished cavity size {} x {} is not stated - a {pass} burn is judged against it",
            u.cavity.0, u.cavity.1
        )));
    }
    if u.undersize < 0.0 || u.gap < 0.0 {
        return Err(GenError::Invalid(format!(
            "sinker {pass}: condition {condition:?} states a negative gap or undersize"
        )));
    }
    for (axis, c, e) in [("X", u.cavity.0, u.electrode.0), ("Y", u.cavity.1, u.electrode.1)] {
        let burned = e + 2.0 * (u.gap + radius);
        if burned > c + SIZE_SLACK {
            return Err(GenError::Invalid(format!(
                "sinker {pass}: the electrode's {e:.3} in {axis} plus 2 x (spark gap {:.3} of condition {condition:?} + orbit radius {radius:.3}) burns {burned:.3}, wider than the {c:.3} cavity - the wall would be burned {:.3} past size per side",
                u.gap,
                (burned - c) * 0.5
            )));
        }
        if u.pass == SinkerPass::Finish && burned < c - SIZE_SLACK {
            return Err(GenError::Invalid(format!(
                "sinker finish leaves the cavity under size: the electrode's {e:.3} in {axis} plus 2 x (spark gap {:.3} of condition {condition:?} + orbit radius {radius:.3}) burns {burned:.3}, short of the {c:.3} cavity by {:.3} per side",
                u.gap,
                (c - burned) * 0.5
            )));
        }
        let wanted = c - 2.0 * u.undersize;
        if (e - wanted).abs() > SIZE_SLACK {
            return Err(GenError::Invalid(format!(
                "sinker {pass}: condition {condition:?} undersizes its electrode {:.3} per side, so under the {c:.3} cavity it is {wanted:.3} in {axis} - the electrode is {e:.3}",
                u.undersize
            )));
        }
    }
    // Circular and planetary paths reach all four axes. A vector pattern
    // does so only in multiples of four; radius alone cannot certify its finish.
    if u.pass == SinkerPass::Finish
        && u.orbit.is_some_and(|o| o.kind == OrbitKind::Vector && !o.vectors.is_multiple_of(4))
    {
        return Err(GenError::Invalid(
            "sinker finish: the vector orbit leaves the cavity under size on a wall - use a multiple of four directions to reach all four walls".into(),
        ));
    }
    Ok(u.undersize - radius)
}

/// The order law between a rough and the finish after it: the finish
/// electrode's undersize must be smaller than the rough's, or the finish
/// takes nothing off the walls the rough left. `true` when the pair is
/// inverted (or equal) - a warning, not a refusal: the finish's own size
/// law already holds the finished cavity to size.
#[must_use]
pub fn inverted(finish_undersize: f64, rough_undersize: f64) -> bool {
    finish_undersize >= rough_undersize - SIZE_SLACK
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair(pass: SinkerPass, e: f64, gap: f64, undersize: f64) -> Undersize {
        Undersize { pass, cavity: (20.0, 12.0), electrode: (e, e - 8.0), gap, undersize, orbit: None }
    }

    #[test]
    fn sinker_undersize_judges_rough_and_finish() {
        // Rough: 0.3 under per side, 0.1 gap - burns 19.6, leaves 0.2.
        assert_eq!(judge(&pair(SinkerPass::Rough, 19.4, 0.1, 0.3), "R").expect("rough"), 0.3);
        // Finish: undersize equals the gap - burns exactly 20.
        assert_eq!(judge(&pair(SinkerPass::Finish, 19.96, 0.02, 0.02), "F").expect("finish"), 0.02);
        // A finish whose condition undersizes more than it gaps: under size.
        let e = judge(&pair(SinkerPass::Finish, 19.9, 0.02, 0.05), "F").expect_err("under size");
        assert!(e.to_string().contains("leaves the cavity under size"), "{e}");
        // A rough whose gap outburns its undersize: past the wall.
        let e = judge(&pair(SinkerPass::Rough, 19.8, 0.2, 0.1), "R").expect_err("overburn");
        assert!(e.to_string().contains("past size"), "{e}");
        // The wrong electrode for the condition.
        let e = judge(&pair(SinkerPass::Rough, 19.0, 0.1, 0.3), "R").expect_err("wrong electrode");
        assert!(e.to_string().contains("undersizes its electrode"), "{e}");
        // No cavity stated.
        let none = Undersize { cavity: (0.0, 12.0), ..pair(SinkerPass::Rough, 19.4, 0.1, 0.3) };
        assert!(judge(&none, "R").is_err());
        assert!(inverted(0.3, 0.3) && inverted(0.4, 0.3) && !inverted(0.02, 0.3));
    }

    #[test]
    fn sinker_undersize_counts_orbit_width_and_keeps_the_floor() {
        let orbit = |radius| Some(Orbit { kind: OrbitKind::Circular, radius, vectors: 4, turns: 1 });
        let rough = Undersize { orbit: orbit(0.05), ..pair(SinkerPass::Rough, 19.4, 0.1, 0.3) };
        assert!((judge(&rough, "R").expect("rough") - 0.25).abs() < 1e-9);
        let finish = Undersize { orbit: orbit(0.03), ..pair(SinkerPass::Finish, 19.9, 0.02, 0.05) };
        assert!((judge(&finish, "F").expect("finish") - 0.02).abs() < 1e-9);
        let overburn = Undersize { orbit: orbit(0.21), ..rough };
        let err = judge(&overburn, "R").expect_err("orbit overburn").to_string();
        assert!(err.contains("orbit") && err.contains("past size"), "{err}");
    }

    #[test]
    fn sinker_undersize_checks_orbit_before_judging_it() {
        let circle = Orbit { kind: OrbitKind::Circular, radius: 0.05, vectors: 4, turns: 1 };
        for orbit in [
            Orbit { radius: f64::NAN, ..circle },
            Orbit { radius: f64::INFINITY, ..circle },
            Orbit { radius: -0.1, ..circle },
            Orbit { kind: OrbitKind::Vector, vectors: 0, ..circle },
            Orbit { kind: OrbitKind::Planetary, turns: 99, ..circle },
        ] {
            let rough = Undersize { orbit: Some(orbit), ..pair(SinkerPass::Rough, 19.4, 0.1, 0.3) };
            let err = judge(&rough, "R").expect_err("bad orbit").to_string();
            assert!(err.contains("orbit"), "{err}");
        }
    }
}
