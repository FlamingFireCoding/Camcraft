//! The flute-reach law (#3525): a cut deeper below the stock top than the
//! tool's flute length puts the tool body above the flutes into the cut -
//! the shop rule that the depth of cut stays within the flute length
//! (machine-tool handbooks, end mill selection). Our own reading.
//!
//! The law names the CAUSE that the collision laws report one move at a
//! time: the operation asks for a depth that the flutes cannot reach. It
//! reads the path's lowest cutting tip against the stock top (the stock's
//! highest point, the conservative reading for a box or flat stock) and
//! compares that depth with the flute length.
//!
//! - The shank (or, past the shank, the holder) is wider than the cutter:
//!   the body meets the wall that the flutes leave. The finding holds the
//!   post, like the shank and holder findings that it explains.
//! - The body is not wider (a reduced or same-diameter shank): it rubs
//!   the wall below the flute. That is a risk, not a certain collision, so
//!   the finding is advisory; the collision laws still judge each move.

use super::types::{FixturePart, ToolGeom, Warning};
use crate::cl::{MoveKind, Toolpath};

/// The slack under which a depth equal to the flute length is no finding,
/// and under which a body radius equal to the cutter's is "not wider" (mm).
const SLACK: f64 = 1e-3;

/// The flute-reach finding of `path` against a stock whose top stands at
/// `stock_top_z`, or `None` when every cutting tip stays within the flute
/// length (or the tool or the top is not finite: there is nothing to read).
#[must_use]
pub fn flute_reach(path: &Toolpath, tool: &ToolGeom, stock_top_z: f64) -> Option<Warning> {
    let flute = tool.flute_len;
    if !(flute.is_finite() && flute > 0.0 && stock_top_z.is_finite() && tool.radius.is_finite()) {
        return None;
    }
    let reach_z = stock_top_z - flute;
    let mut first: Option<usize> = None;
    let mut low = f64::INFINITY;
    for (i, m) in path.moves.iter().enumerate() {
        if m.kind == MoveKind::Rapid {
            continue;
        }
        let z = m.from.z.min(m.to.z);
        if !z.is_finite() {
            continue;
        }
        if z < reach_z - SLACK && first.is_none() {
            first = Some(i);
        }
        low = low.min(z);
    }
    let at = first?;
    let depth_mm = stock_top_z - low;
    let wider = |r: f64| r.is_finite() && r > tool.radius + SLACK;
    let into_holder = depth_mm > flute + tool.shank_len.max(0.0) + SLACK;
    let wider_body = if wider(tool.shank_radius) {
        Some(FixturePart::Shank)
    } else if into_holder && wider(tool.holder_radius) {
        Some(FixturePart::Holder)
    } else {
        None
    };
    Some(Warning::FluteShort { at, flute_mm: flute, depth_mm, wider: wider_body })
}

/// The finding's note with the tool's name (the warning itself is `Copy`
/// and carries no name): `None` for any other warning.
#[must_use]
pub fn flute_note(tool_name: &str, w: &Warning) -> Option<String> {
    let Warning::FluteShort { flute_mm, depth_mm, wider, .. } = w else {
        return None;
    };
    let cause = match wider {
        Some(FixturePart::Holder) => "the holder, wider than the cutter, is in the cut".to_string(),
        Some(_) => "the shank, wider than the cutter, is in the cut".to_string(),
        None => "the shank is in the cut and can rub the walls (the post is not held)".to_string(),
    };
    Some(format!(
        "{tool_name} has a flute length of {flute_mm:.3} mm; this operation cuts {depth_mm:.3} mm below the stock top; {cause} - use a tool with a flute length of at least {depth_mm:.3} mm, or cut less deep"
    ))
}
