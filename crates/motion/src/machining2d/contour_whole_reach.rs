//! Whole-model 2D contour (#2597): a ring of a deeper band whose tool sweep
//! reaches outside every sweep cut so far starts at Top Z, not at the level
//! where its outline appeared. The contour knows no stock outline, so the
//! material outside the part is taken to stand up to Top Z; a ring that
//! enters it first at a deep level would bury the tool (axial depth far over
//! the stepdown). Such a ring takes every flat level above its first level,
//! so each pass is at most one stepdown under the one before it. A ring that
//! cannot be proven clear at those levels refuses by name. Provenance: the
//! 2.5D Z-level rule of published machining practice (no pass deeper than
//! the stepdown into uncut material) with polygon offset and difference from
//! textbook computational geometry; no vendor material consulted.

use std::ops::Range;

use crate::geo2d::Region;
use crate::geometry::Triangle;

use super::contour::PreparedBand;
use super::contour_depth::{Ladder, ladder};
use super::contour_params::ContourParams;
use super::face::GenError;

const SAME_LEVEL: f64 = 1e-6;
/// Half the width (mm) under which a strip of the new sweep outside the cut
/// sweeps is offset noise (two slices of one wall tessellate apart by the
/// arc sag), not uncut material: the strip must stay non-empty after an
/// inward offset by this much, so it is wider than 0.05 mm.
const FRESH_HALF_WIDTH: f64 = 0.025;

/// The area the tool covers when its center walks every ring of `center`:
/// the ring lines grown by the tool radius on both sides.
pub(super) fn band_sweep(center: &Region, r: f64) -> Result<Region, GenError> {
    Ok(center.offset(r)?.difference(&center.offset(-r)?)?)
}

/// `lad` with the flat levels above its first level put in front of each ring
/// whose sweep reaches outside `cover` (the sweeps of every band cut so far).
/// `flat` is the operation's full level list from Top Z down.
pub(super) fn reach_up(
    band: &PreparedBand,
    lad: Ladder,
    cover: &Region,
    flat: &[f64],
    part: &[Triangle],
    bodies: &[Range<usize>],
    p: &ContourParams,
) -> Result<Ladder, GenError> {
    if cover.is_empty() {
        return Ok(lad);
    }
    let r = p.tool_diameter * 0.5;
    let mut per_ring = lad.per_ring.clone();
    for (li, l) in band.loops.iter().enumerate() {
        let Some(&first) = lad.ring(li).first() else { continue };
        // A level at Top Z cuts no material (the material is taken to stand
        // up to Top Z, not above it), so only the levels under it are added.
        let upper: Vec<f64> = flat
            .iter()
            .copied()
            .filter(|&z| z > first + SAME_LEVEL && z < p.top_z - SAME_LEVEL)
            .collect();
        let Some(ring) = band.plan.get(li) else { continue };
        if upper.is_empty() {
            continue;
        }
        let disk = Region::new(vec![l.clone()])?;
        let sweep = disk.offset(r)?.difference(&disk.offset(-r)?)?;
        if sweep.difference(cover)?.offset(-FRESH_HALF_WIDTH)?.is_empty() {
            continue;
        }
        let mut why = Vec::new();
        let up = ladder(std::slice::from_ref(ring), part, bodies, &upper, p.top_z, r, p.stock_to_leave, &mut why);
        let unread = up.unread.first().cloned().flatten();
        if up.ring(0).len() != upper.len() || unread.is_some() {
            let at = l.points().first().map_or((0.0, 0.0), |q| (q.x, q.y));
            let levels: Vec<String> = upper.iter().map(|z| format!("{z:.3}")).collect();
            why.extend(unread);
            return Err(GenError::Invalid(format!(
                "whole-model contour: the outline that grows at Z {first:.3} (ring at ({:.3}, {:.3})) \
                 has uncut material above it up to Top Z {:.3}, and the ring cannot be proven clear at \
                 the levels above ({}) - a first pass at Z {first:.3} would bury the tool; {}",
                at.0,
                at.1,
                p.top_z,
                levels.join(", "),
                why.join("; ")
            )));
        }
        let mut mine = upper;
        mine.extend(per_ring.get(li).map_or(&[][..], |v| v.as_slice()));
        if let Some(slot) = per_ring.get_mut(li) {
            *slot = mine;
        }
    }
    let mut zs: Vec<f64> = per_ring.iter().flatten().copied().collect();
    zs.sort_by(|a, b| b.total_cmp(a));
    zs.dedup_by(|a, b| a.to_bits() == b.to_bits());
    Ok(Ladder { zs, per_ring, unread: lad.unread })
}
