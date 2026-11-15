//! Side Auto for picked contour rings (#1933): the side of the rings
//! that holds the material, read from the part's section at the rings'
//! own level - never from the full XY silhouette, which a blind bore's
//! floor fills. Provenance: the 2.5D profiling rule of published
//! machining practice (the cutter rides the air side of the wall it
//! trims); our own design notes. No vendor material consulted.

use std::ops::Range;

use crate::geo2d::Region;
use crate::geometry::Triangle;
use crate::geometry::slice::material_above_of;

use super::face::GenError;

/// How far from the rings' level each probe section is taken, mm: just
/// below the highest ring point and just above the lowest one. Far
/// under any wall a cutter trims, far over the slice's own nudge.
pub const AUTO_PROBE_DZ: f64 = 0.01;
/// Width of the band just outside the rings that stands for the other
/// side of the wall, mm.
pub const AUTO_BAND: f64 = 0.5;
/// A side "holds material" when at least this fraction of its probe
/// area is part section (the old fill law's threshold).
const HOLDS: f64 = 0.5;

/// What Side Auto resolves to for a set of picked rings.
#[derive(Debug, Clone, PartialEq)]
pub enum AutoSide {
    /// The material is inside the rings: the cutter goes outside.
    Outside,
    /// The material is outside the rings: the cutter goes inside.
    Inside,
    /// The section cannot tell (material on both sides, or on neither,
    /// or the probes disagree): the sentence asks for a named side.
    Undecided(String),
}

/// One probe's verdict: `Some(true)` = Inside, `Some(false)` = Outside.
fn probe(rings: &Region, band: &Region, facets: &[Triangle], bodies: &[Range<usize>], z: f64) -> Result<Option<bool>, GenError> {
    let Ok(sec) = material_above_of(facets, bodies, z) else {
        return Ok(None);
    };
    let (whole, ring_band) = (rings.area().abs(), band.area().abs());
    if whole <= 0.0 || ring_band <= 0.0 {
        return Ok(None);
    }
    let inside = rings.intersect(&sec)?.area().abs() / whole >= HOLDS;
    let outside = band.intersect(&sec)?.area().abs() / ring_band >= HOLDS;
    Ok(match (inside, outside) {
        (true, false) => Some(false),
        (false, true) => Some(true),
        _ => None,
    })
}

/// Side Auto for `rings`, picked between Z `z_lo` and `z_hi` (one level
/// for a flat chain or face; a wall patch spans its height). The part
/// section is read just below the top of that span, then just above its
/// bottom: the first probe that finds material on exactly one side
/// decides. A blind bore rim reads air inside and metal outside at its
/// mouth (Inside), as a through hole does; a boss or an outer wall reads
/// metal inside and air outside (Outside). A floor rim reads metal on
/// both sides below it and decides from above. When no probe decides, or
/// the two probes disagree, the answer is [`AutoSide::Undecided`] with
/// the sentence to show - never a guess.
pub fn auto_side(
    rings: &Region,
    facets: &[Triangle],
    bodies: &[Range<usize>],
    z_lo: f64,
    z_hi: f64,
) -> Result<AutoSide, GenError> {
    if facets.is_empty() || rings.is_empty() || !z_lo.is_finite() || !z_hi.is_finite() {
        // No part to read: the ring bounds what it bounds - Outside, the
        // side a contour of a sketch outline always took.
        return Ok(AutoSide::Outside);
    }
    let band = rings.offset(AUTO_BAND)?.difference(rings)?;
    let (below, above) = (z_hi - AUTO_PROBE_DZ, z_lo + AUTO_PROBE_DZ);
    let first = probe(rings, &band, facets, bodies, below)?;
    let second = probe(rings, &band, facets, bodies, above)?;
    let undecided = |why: &str| {
        AutoSide::Undecided(format!(
            "Side Auto cannot tell which side of the picked ring holds the material at Z {z_hi:.3}: {why} - set Side to Inside or Outside"
        ))
    };
    Ok(match (first, second) {
        (Some(a), Some(b)) if a != b => undecided("the part stands inside the ring on one side of that level and outside it on the other"),
        (Some(true), _) | (None, Some(true)) => AutoSide::Inside,
        (Some(false), _) | (None, Some(false)) => AutoSide::Outside,
        (None, None) => undecided("the part stands on both sides of the ring there, or on neither"),
    })
}
