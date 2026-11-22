//! #3452: the part-off kerf's side and its gouge law - the parting plane is the finished part's end face, the whole kerf of the blade stands on the waste side (the bar that stays in the chuck), and a parting move whose band enters the finished part is refused by name. Machine-tool handbook cutoff practice (the part keeps its length, the kerf comes out of the bar) and ISO 841's lathe Z (positive away from the workholding) for a plane that the model runs on past; our own reading, no vendor material consulted.

use crate::cl::{MoveKind, Toolpath};
use crate::geometry::Triangle;

use super::profile;
use super::revolve;
use super::stock::StockEnv;

/// How far a kerf may stand past the end face and still be "beside"
/// it (mm): one micrometre. A float hair, never a real cut.
pub const KERF_TOL: f64 = 1e-6;

/// Which side of the parting plane the scrap is on, along the part's
/// axis: `Below` is toward the lower stations (the lathe's chuck at
/// -Z), `Above` toward the higher ones.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WasteSide {
    Below,
    Above,
}

impl WasteSide {
    /// The kerf band `(lo, hi)` a blade of width `w` takes beside the
    /// plane at station `s`: its whole width on this side, one flank
    /// on the plane.
    pub fn kerf(self, s: f64, w: f64) -> (f64, f64) {
        match self {
            WasteSide::Below => (s - w, s),
            WasteSide::Above => (s, s + w),
        }
    }

    /// The blade's CL station (the middle of its tip edge) for that
    /// kerf.
    pub fn centre(self, s: f64, w: f64) -> f64 {
        match self {
            WasteSide::Below => s - w * 0.5,
            WasteSide::Above => s + w * 0.5,
        }
    }

    /// The part of a band `(lo, hi)` that stands on the finished
    /// part's side of the plane at `s`.
    fn into_part(self, s: f64, lo: f64, hi: f64) -> (f64, f64) {
        match self {
            WasteSide::Below => (lo.max(s), hi),
            WasteSide::Above => (lo, hi.min(s)),
        }
    }
}

/// The scrap side for a parting plane at station `s` on a model whose
/// silhouette spans the stations `d`:
/// - a plane at the model's low end (within `tol`) keeps the model
///   above it - the scrap is below;
/// - a plane at the model's high end keeps the model below it - the
///   scrap is above;
/// - a plane the model runs on past on both sides (a model that
///   carries its own bar) parts toward the side the setup's bar runs
///   on further past the model, and with no bar to read, toward the
///   lathe's chuck (ISO 841: +Z points away from the workholding).
pub fn waste_side(d: (f64, f64), s: f64, tol: f64, stock: Option<StockEnv>) -> WasteSide {
    let tol = tol.max(KERF_TOL);
    let at_lo = s <= d.0 + tol;
    let at_hi = s >= d.1 - tol;
    match (at_lo, at_hi) {
        (true, false) => WasteSide::Below,
        (false, true) => WasteSide::Above,
        _ => {
            let runs = stock.map(|e| {
                let lo = if e.s_bot.is_finite() { (d.0 - e.s_bot).max(0.0) } else { 0.0 };
                let hi = if e.s_top.is_finite() { (e.s_top - d.1).max(0.0) } else { 0.0 };
                (lo, hi)
            });
            match runs {
                Some((lo, hi)) if hi > lo + tol => WasteSide::Above,
                _ => WasteSide::Below,
            }
        }
    }
}

/// The stations the bar spans: the model's own, widened by the setup
/// stock's ends where they read.
pub fn bar_span(d: (f64, f64), stock: Option<StockEnv>) -> (f64, f64) {
    let (mut lo, mut hi) = d;
    if let Some(e) = stock {
        if e.s_bot.is_finite() {
            lo = lo.min(e.s_bot);
        }
        if e.s_top.is_finite() {
            hi = hi.max(e.s_top);
        }
    }
    (lo, hi)
}

/// The generation verdict for a part-off path (#3452): `Some(reason)`
/// when a cutting move's blade band takes material of the finished
/// part - the model on the part's side of the parting plane at
/// `parting_z` - or when the path cannot be judged at all (a judge
/// that cannot measure never reports silence). The band is the
/// blade's own width, never wider.
pub fn part_kerf_refusal(
    facets: &[Triangle],
    path: &Toolpath,
    width: f64,
    parting_z: f64,
    tol: f64,
    stock: Option<StockEnv>,
) -> Option<String> {
    if facets.is_empty() || !width.is_finite() || width <= 0.0 || !parting_z.is_finite() || !tol.is_finite() || tol <= 0.0 {
        return Some("the part-off kerf cannot be judged: no model, or a blade width, plane or tolerance that is not a size".into());
    }
    let f = match profile::Frame::about(facets) {
        Ok(f) => f,
        Err(e) => return Some(format!("the part-off kerf cannot be judged: {e}")),
    };
    let pts = match profile::silhouette(facets, &f, tol * 0.25) {
        Ok(p) if !p.is_empty() => p,
        Ok(_) => return Some("the part-off kerf cannot be judged: the model has no silhouette".into()),
        Err(e) => return Some(format!("the part-off kerf cannot be judged: {e}")),
    };
    let sg = revolve::segs(&pts);
    let d = (pts[0].0, pts[pts.len() - 1].0);
    let s_p = f.station_of(parting_z);
    let side = waste_side(d, s_p, tol, stock);
    let half = width * 0.5;
    for (i, m) in path.moves.iter().enumerate() {
        if m.kind == MoveKind::Rapid {
            continue;
        }
        let (sa, ra) = f.sr(m.from);
        let (sb, rb) = f.sr(m.to);
        let (lo, hi) = (sa.min(sb) - half, sa.max(sb) + half);
        let (olo, ohi) = side.into_part(s_p, lo, hi);
        if ohi - olo <= KERF_TOL {
            continue;
        }
        let top = revolve::span_max(&sg, olo, ohi);
        if top > ra.min(rb) + KERF_TOL {
            return Some(format!(
                "the part-off cuts into the finished part: move {i} puts the {width:.3} mm blade at \
                 Z{:.3} to Z{:.3}, {:.3} mm past the part's end face at Z{parting_z:.3} - the whole \
                 kerf must stand on the bar's side of that face",
                f.height_of(lo),
                f.height_of(hi),
                ohi - olo,
            ));
        }
    }
    None
}
