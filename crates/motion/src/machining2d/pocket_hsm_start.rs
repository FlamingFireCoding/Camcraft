//! "Pass starts at" for the planar Adaptive pocket (#3355): how the HSM
//! plan honours the shared `PassStart` of #3316. Provenance: our own law
//! from general machining practice (an adaptive cut starts from open
//! air or grows from its own bore, never with a full-width slot along a
//! wall) and the air-side plan of #1950 (Held 1991); no vendor material
//! consulted.

use crate::geo2d::Region;

use super::face::GenError;
use super::adaptive_method::AdaptiveMethod;
use super::pass_start::PassStart;
use super::pocket::{PocketParams, PocketStrategy, plan_passes_from};
use super::pocket_hsm::{Planned, adaptive_ordered};

/// The Adaptive pocket's passes under `start`. `stock` is the stock
/// outline in the area's own frame (`None`: not known here).
///
/// - Auto: [`super::pocket_hsm::adaptive_passes`] with no stock, the plan
///   every earlier file cut, byte for byte.
/// - Outside: each piece is cut from the air past the stock edge inward
///   and ends on its holes' walls (the #1950 air plan). Where a piece
///   has no air side, the Offset rings cut the area from its outer
///   boundary in, and the note says why.
/// - Inside: an area without holes is the growth plan itself - it
///   starts at its bore in the core and walks out to the outer wall. An
///   area with holes cannot start on a hole wall without a full-width
///   slot, so the Offset rings cut it from its holes out, said.
///
/// The trochoid fallback is tried only under Auto: its loops follow the
/// medial line and have no ring order.
pub(crate) fn adaptive_passes_from(center: &Region, p: &PocketParams, what: &str, stock: Option<&Region>, start: PassStart) -> Result<Planned, GenError> {
    match start {
        PassStart::Auto => adaptive_ordered(center, p, what, true, None, start),
        PassStart::Outside => match stock.filter(|s| !s.is_empty()) {
            Some(st) => adaptive_ordered(center, p, what, false, Some(st), start),
            None => rings(center, p, what, start, "has no stock outline to start from in the air"),
        },
        PassStart::Inside if center.loops().iter().any(|l| l.area() < 0.0) => {
            rings(center, p, what, start, "cannot start on a hole wall without a full-width slot")
        }
        PassStart::Inside => adaptive_ordered(center, p, what, false, None, start),
    }
}

/// Why the Trochoidal method refuses `start` (#3355), or `None`. Its
/// loops follow the area's medial line and have no ring order, so Inside
/// and Outside are refused by name, never ignored.
#[must_use]
pub fn trochoidal_pass_start_refusal(method: AdaptiveMethod, start: PassStart) -> Option<String> {
    (method == AdaptiveMethod::Trochoidal && start != PassStart::Auto).then(|| {
        format!(
            "Pass starts at {} is not available with the Trochoidal method - its loops follow the medial line and have no ring order; use Auto or Spiral",
            start.label()
        )
    })
}

/// The Offset rings in the order `start` asks, with the note that names
/// the switch (an order is never changed in silence).
fn rings(center: &Region, p: &PocketParams, what: &str, start: PassStart, why: &str) -> Result<Planned, GenError> {
    let s = PocketStrategy::OffsetRings;
    let step = crate::load::load_step(p.tool_diameter, p.optimal_load);
    let note = format!(
        "adaptive: Pass starts at {} - the HSM plan {why}; Offset rings cut {what} in that order instead",
        start.label()
    );
    Ok((plan_passes_from(center, step, p.climb, s, start)?, vec![note], s, None, Vec::new()))
}

#[cfg(test)]
#[path = "pocket_hsm_start_tests.rs"]
mod tests;
