//! #2901: generate's seam to the first-traverse law (`verify::first_traverse_low`): which operations answer to it and which steel they read; our own reading of the shop rule "never traverse through uncut stock or steel", no vendor material consulted.

use crate::cl::{TiltedFrame, Toolpath};
use crate::fixture::{FixtureBox, MACHINE_TABLE};
use crate::geometry::Point3;
use crate::machine::AxisLimits;
use crate::verify::{self, Warning};

use super::OpParams;
use super::kinds::Section;
use super::material::Z_TOL;

/// What the first traverse must ride over: the start stock's top in the
/// path's frame, and the setup's steel in the world (`fixture::
/// named_boxes_for`: the tagged fixtures' boxes, then the machine table),
/// `names` beside `fixtures` in the same order. `rotaries` is the mounted
/// machine's rotary inventory (`None`: no machine is mounted).
pub(super) struct Over<'a> {
    pub stock_top: f64,
    pub fixtures: &'a [FixtureBox],
    pub names: &'a [String],
    pub rotaries: Option<&'a [AxisLimits]>,
}

/// The rotary inventory of `job.machine_rotaries` as [`Over::rotaries`].
pub(super) fn rotaries_of(m: Option<&(String, Vec<AxisLimits>)>) -> Option<&[AxisLimits]> {
    m.map(|(_, axes)| axes.as_slice())
}

/// Leads the note of a 3+2 plane whose first traverse the machine table
/// could not judge (#2987).
const TABLE_UNJUDGED_NOTE: &str = "First traverse: the machine table is not judged on this tilted plane";

/// Whether the operation's first traverse is a move on a plane in its
/// own frame on a mill: the planar and 3-D milling kinds and the
/// vertical hole kinds, upright or 3+2. The multi-axis, rotary,
/// turning, plasma and EDM kinds read their clearance on other
/// surfaces or axes; the rapid laws judge their moves (decision #2901).
fn answers(params: &OpParams, mill: bool, rotary: bool) -> bool {
    if !mill || rotary {
        return false;
    }
    match params.kind().info().section {
        Section::Mill2D | Section::Mill3D => true,
        Section::Drilling => !matches!(params, OpParams::Drill { cross, .. } if !cross.is_empty()),
        _ => false,
    }
}

/// Whether the machine reaches a 3+2 pose only by turning the WORK
/// (#2987): a tilting rotary that turns the part, and no tilting rotary
/// that may ride the head (a head's own, or one an old file never
/// stated). The table then stays under the work in the machine frame,
/// where the tool stands upright; the stock top over it is the higher
/// top, and the index-swing clause judges the work against the table.
fn work_tilts(rotaries: &[AxisLimits]) -> bool {
    let tilts = |a: &&AxisLimits| a.rotary && a.axis.is_none_or(|d| d[0].abs() > 1e-9 || d[1].abs() > 1e-9);
    let mut tilting = rotaries.iter().filter(tilts).peekable();
    tilting.peek().is_some() && tilting.all(|a| a.turns_part == Some(true))
}

/// The table where the tool can meet it (#2987): its box clipped to the
/// machine's Z travel, which starts at the table top - the top face over
/// the stated X/Y travel. The slab under it (`MAX_FIXTURE_DIM` deep) is
/// out of the tool's reach, and along a tilted axis it reads a far edge.
fn table_top_face(b: &FixtureBox) -> FixtureBox {
    FixtureBox { min: Point3::new(b.min.x, b.min.y, b.max.z), max: b.max }
}

/// How the machine table meets a 3+2 plane at local height `start_h`.
enum TableRead {
    /// The move along the tool axis to the plane can carry the tool under
    /// the table top: the table's top face is steel the plane must clear.
    Counts(FixtureBox),
    /// No move along the tool axis to the plane goes under the table top.
    Clear,
    /// The tool axis points down toward the table and the move to the
    /// plane can go under its top: the verdict cannot name a height to
    /// raise to, so a note names the table as not judged.
    Unjudged,
}

/// The table on a 3+2 plane (#2987). The control does not know where the
/// tool stands, so the worst start is on the table top face itself. The
/// move along the tool axis from there to the plane changes the world Z
/// by `(start_h - h) * axis_z` (`h`: that start's local height). The
/// table counts only when that move can drop the tool more than `Z_TOL`
/// under the top. Then the in-plane traverse runs between two points
/// over the table top (a convex half-space), so it cannot go under it.
/// A horizontal axis (B 90) never changes the Z: the table never counts,
/// although its far edge reads high along that axis.
fn table_read(b: &FixtureBox, f: &TiltedFrame, start_h: f64) -> TableRead {
    let face = table_top_face(b);
    let up = f.axis[2];
    if up > 0.0 {
        let high = verify::box_top(&face, |p| f.z_local(p));
        if (high - start_h) * up > Z_TOL { TableRead::Counts(face) } else { TableRead::Clear }
    } else {
        let low = -verify::box_top(&face, |p| -f.z_local(p));
        if (start_h - low) * -up > Z_TOL { TableRead::Unjudged } else { TableRead::Clear }
    }
}

/// Add the first-traverse verdict to `warns` when `path` (in `frame`'s
/// local heights when it is tilted) starts under the stock or any steel.
/// The note it returns says what a 3+2 plane could not judge.
///
/// A 3+2 plane reads the tagged fixtures along its tool axis (they ride
/// the table with the part). It reads the machine table's top face over
/// the stated travel when the head tilts (#2987), only where the move
/// along the tool axis can carry the tool under the table top
/// (`table_read`). When only the work tilts, the table stays
/// under the work (`work_tilts`). A mounted machine that states no travel
/// gives no table box, and the note names that. Upright, the table's top
/// is under the stock and never the higher top.
pub(super) fn judge(path: &Toolpath, params: &OpParams, (mill, rotary): (bool, bool), frame: Option<&TiltedFrame>, over: &Over<'_>, warns: &mut Vec<Warning>) -> Option<String> {
    if !answers(params, mill, rotary) {
        return None;
    }
    let mut note = None;
    let w = match frame {
        Some(f) => {
            let work_turns = over.rotaries.is_some_and(work_tilts);
            let is_table = |i: usize| over.names.get(i).is_some_and(|n| n == MACHINE_TABLE);
            let start_h = path.start().map(|p| p.z).filter(|z| z.is_finite());
            let mut steel: Vec<FixtureBox> = Vec::with_capacity(over.fixtures.len());
            for (i, b) in over.fixtures.iter().enumerate() {
                if !is_table(i) {
                    steel.push(*b);
                    continue;
                }
                match start_h.filter(|_| !work_turns).map(|h| table_read(b, f, h)) {
                    Some(TableRead::Counts(face)) => steel.push(face),
                    Some(TableRead::Unjudged) => {
                        note = Some(format!("{TABLE_UNJUDGED_NOTE}: the tool axis points down toward the table, so the move along it to the clearance plane can go under the table top"));
                    }
                    Some(TableRead::Clear) | None => {}
                }
            }
            let has_table = (0..over.fixtures.len()).any(is_table);
            if over.rotaries.is_some() && !work_turns && !has_table {
                note = Some(format!("{TABLE_UNJUDGED_NOTE}: the machine does not state all of the X, Y and Z travel, so the first traverse is judged against the stock and the fixtures only"));
            }
            verify::first_traverse_low(path, over.stock_top, &steel, |p| f.z_local(p), Z_TOL)
        }
        None => verify::first_traverse_low(path, over.stock_top, over.fixtures, |p| p.z, Z_TOL),
    };
    if let Some(w) = w
        && !warns.contains(&w)
    {
        warns.push(w);
    }
    note
}
