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
    /// The Z stroke of the mounted machine, mm: the tool-change height
    /// over the table top (`None`: unstated, read as 0 - a start on the
    /// table top, the stricter premise).
    pub z_stroke: Option<f64>,
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
    /// the table top: a point on the tool axis at the lowest plane height
    /// that clears it (`box_top` reads that height) is steel to clear.
    Counts(FixtureBox),
    /// No move along the tool axis to the plane goes under the table top.
    Clear,
    /// The tool axis points down toward the table and the move to the
    /// plane can go under its top: the verdict cannot name a height to
    /// raise to, so a note names the table as not judged.
    Unjudged,
}

/// The table on a 3+2 plane (#2987), decision premise (b): the control
/// does not know the tool's X/Y, but the tool starts at the tool-change
/// height, the top of the Z travel (`stroke` over the table top) - the
/// premise of the upright law too. The worst start is any point of the
/// travel footprint at that height. The move along the tool axis from a
/// start at local height `h` to the plane changes the world Z by
/// `(start_h - h) * axis_z`; the table counts only when that move can end
/// more than `Z_TOL` under the table top. Then the in-plane traverse
/// runs between two points over the table top (a convex half-space), so
/// it cannot go under it. A horizontal axis (B 90) never changes the Z.
/// NOT covered: a program that starts with the tool low near the table
/// (after a manual jog, for example).
fn table_read(b: &FixtureBox, f: &TiltedFrame, start_h: f64, stroke: f64) -> TableRead {
    let face = table_top_face(b);
    let z_top = face.max.z + stroke;
    let starts = FixtureBox { min: Point3::new(face.min.x, face.min.y, z_top), max: Point3::new(face.max.x, face.max.y, z_top) };
    let up = f.axis[2];
    if up > 0.0 {
        let high = verify::box_top(&starts, |p| f.z_local(p));
        if (high - start_h) * up - stroke <= Z_TOL {
            return TableRead::Clear;
        }
        // The lowest plane that a move from every start reaches over the
        // table top, as a point on the tool axis through the origin.
        let clear = high - stroke / up;
        let (o, a) = (f.origin, f.axis);
        let p = Point3::new(o.x + clear * a[0], o.y + clear * a[1], o.z + clear * a[2]);
        TableRead::Counts(FixtureBox { min: p, max: p })
    } else {
        let low = -verify::box_top(&starts, |p| -f.z_local(p));
        if (start_h - low) * -up - stroke > Z_TOL { TableRead::Unjudged } else { TableRead::Clear }
    }
}

/// Add the first-traverse verdict to `warns` when `path` (in `frame`'s
/// local heights when it is tilted) starts under the stock or any steel.
/// The note it returns says what a 3+2 plane could not judge.
///
/// A 3+2 plane reads the tagged fixtures along its tool axis (they ride
/// the table with the part). It reads the machine table's top face over
/// the stated travel when the head tilts (#2987), only where the move
/// along the tool axis from the tool-change height can carry the tool
/// under the table top (`table_read`). When only the work tilts, the table stays
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
            let stroke = over.z_stroke.filter(|s| s.is_finite() && *s >= 0.0).unwrap_or(0.0);
            let mut steel: Vec<FixtureBox> = Vec::with_capacity(over.fixtures.len());
            for (i, b) in over.fixtures.iter().enumerate() {
                if !is_table(i) {
                    steel.push(*b);
                    continue;
                }
                match start_h.filter(|_| !work_turns).map(|h| table_read(b, f, h, stroke)) {
                    Some(TableRead::Counts(face)) => steel.push(face),
                    Some(TableRead::Unjudged) => {
                        note = Some(format!("{TABLE_UNJUDGED_NOTE}: the tool axis points down toward the table, so the move along it from the tool-change height (the Z travel top) to the clearance plane can go under the table top"));
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The `multiaxis/indexed` catalog case's Tilted pocket (#2987): A30
    /// B30 on `tilthead.mch` (X +-600, Y +-500, Z -500..500: the table
    /// top is Z -500, the tool-change height Z 500, the stroke 1000),
    /// tool axis (sqrt3/4, -0.5, 0.75), plane at 52.120 along it. The
    /// worst start, (600, -500, 500), reads 600 sqrt3/4 + 250 + 375 =
    /// 884.808; the move from it to the plane ends at Z 500 - (884.808 -
    /// 52.120) * 0.75 = -124.5, 375.5 mm over the table top: Clear.
    /// A plane under 884.808 - 1000 / 0.75 = -448.526 would end under it.
    #[test]
    fn the_catalog_tilted_pocket_clears_the_table_from_the_tool_change_height() {
        let axis = [3f64.sqrt() / 4.0, -0.5, 0.75];
        let f = TiltedFrame { origin: Point3::new(0.0, 0.0, 0.0), u: [1.0, 0.0, 0.0], v: [0.0, 1.0, 0.0], axis };
        let table = FixtureBox { min: Point3::new(-600.0, -500.0, -1.0e6), max: Point3::new(600.0, 500.0, -500.0) };
        assert!(matches!(table_read(&table, &f, 52.12, 1000.0), TableRead::Clear));
        let clear = 600.0 * axis[0] + 250.0 + 375.0 - 1000.0 / 0.75;
        assert!((clear + 448.526).abs() < 1e-3, "{clear}");
        assert!(matches!(table_read(&table, &f, clear + 0.01, 1000.0), TableRead::Clear));
        match table_read(&table, &f, clear - 0.01, 1000.0) {
            TableRead::Counts(b) => {
                let top = verify::box_top(&b, |p| f.z_local(p));
                assert!((top - clear).abs() < 1e-6, "the height to clear along the axis: {top}, want {clear}");
            }
            _ => panic!("a plane under the clear height must count the table"),
        }
        // With no stroke (a start on the table top, the stricter reading)
        // the same plane at 52.120 counts: the move ends 62.0 mm under it.
        assert!(matches!(table_read(&table, &f, 52.12, 0.0), TableRead::Counts(_)));
    }
}
