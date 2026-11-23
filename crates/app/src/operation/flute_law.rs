//! #3525: a 2D milling operation that cuts deeper below the stock top than
//! its tool's flute length is named by that cause, as the FIRST warning and
//! the first note, before the move findings that it explains (the shank or
//! holder hits and the rapids next to them). The law itself is
//! `camcraft_sim::verify::flute_reach`; this file picks the operations and
//! places the finding. Provenance: the shop rule that the depth of cut
//! stays within the flute length (machine-tool handbooks); our own reading.

use crate::cl::Toolpath;
use crate::operation::OpParams;
use crate::tools::ToolGeom;
use crate::verify::{self, Warning};

/// The 2D milling kinds whose depth is a stack of levels down from the
/// stock: the pocket family and the 2D contour.
fn judged(params: &OpParams) -> bool {
    matches!(
        params,
        OpParams::Pocket { .. } | OpParams::Adaptive2D { .. } | OpParams::Offset2D { .. } | OpParams::Contour { .. }
    )
}

/// Put the flute-reach finding of `path` first in `warns` and its named
/// note first in the path's notes. Nothing for another kind, or when the
/// cut stays within the flute length.
pub(super) fn name_first(params: &OpParams, path: &mut Toolpath, warns: &mut Vec<Warning>, geom: &ToolGeom, tool_name: &str, stock_top_z: f64) {
    if !judged(params) {
        return;
    }
    let Some(w) = verify::flute_reach(path, geom, stock_top_z) else {
        return;
    };
    if let Some(note) = verify::flute_note(tool_name, &w) {
        path.notes.insert(0, note);
    }
    warns.insert(0, w);
}
