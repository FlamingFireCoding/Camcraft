//! #3077: generate's seam to the live-tool first-traverse law (`verify::live_first_traverse`) - the cross drill on a lathe or a mill-turn, read from the turret's index position; our own reading of the RS-274 / ISO 6983 lathe rapid, no vendor material consulted.
//!
//! The lathe post (`fanuc-lathe.cps` `liveMove`) positions C, then sends
//! X and Z in one G00 to the first point of the section. The control
//! knows the turret only at its index position, the top of the X and Z
//! travel (decision #3077, the premise of #2987 and of the upright law).
//! The first move's two ends are judged: the post writes its end, and a
//! stream that states its start first makes that the first point.

use crate::cl::Toolpath;
use crate::fixture::FixtureBox;
use crate::geometry::{Point3, Triangle};
use crate::machining2d::GenError;
use crate::tools::Tool;
use crate::verify::{self, LiveHit, LiveSolids, LiveStart};

use super::material::Z_TOL;

/// What the cross drill's first traverse is judged against: the stock
/// and the parts by name (`solids`), a stock known only by its box, and
/// the setup's steel split by whether it turns (`fixture::workholding_for`).
pub(super) struct LiveWork<'a> {
    pub solids: &'a [(&'a str, &'a [Triangle])],
    pub stock_box: Option<(Point3, Point3)>,
    pub turning: &'a [FixtureBox],
    pub fixed: &'a [FixtureBox],
}

/// The tool's widest radius behind the tip: the drill, or a wider shank.
fn tool_radius(tool: &Tool) -> f64 {
    let shank = if tool.shank_d.is_finite() { tool.shank_d } else { 0.0 };
    0.5 * tool.diameter.max(shank)
}

fn unjudged(why: &str) -> GenError {
    GenError::Invalid(format!(
        "cross drill: the first rapid from the turret's index position cannot be judged ({why}) - refused, never posted unjudged"
    ))
}

/// Refuse by name a cross drill whose first rapid, from the turret's
/// index position about the spindle line through `origin` (the work
/// offset), can pass the stock, a part or the steel.
pub(super) fn judge(path: &Toolpath, origin: [f64; 3], tool: &Tool, work: &LiveWork<'_>) -> Result<(), GenError> {
    let Some(first) = path.moves.first() else {
        return Err(unjudged("the path has no first move"));
    };
    let r_tool = tool_radius(tool);
    let facets: Vec<&[Triangle]> = work.solids.iter().map(|(_, f)| *f).collect();
    let boxes: Vec<FixtureBox> = work.stock_box.map(|(min, max)| FixtureBox { min, max }).into_iter().collect();
    let solids = LiveSolids { work: &facets, work_boxes: &boxes, turning: work.turning, fixed: work.fixed };
    for at in [first.from, first.to] {
        let s = LiveStart { start: at, spindle: [origin[0], origin[1]], tool_radius: r_tool };
        let hit = verify::live_first_traverse(&s, &solids, Z_TOL)
            .map_err(|_| unjudged("its first point, the spindle line or the tool radius is not finite"))?;
        let Some(hit) = hit else { continue };
        let (what, reach) = match hit {
            LiveHit::Work { index, radius_mm } => (work.solids.get(index).map_or("the stock", |(n, _)| *n), radius_mm),
            LiveHit::Steel { turning: true, radius_mm, .. } => ("a fixture that turns with the part (a chuck)", radius_mm),
            LiveHit::Steel { turning: false, radius_mm, .. } => ("a fixed fixture (a tailstock or the machine table)", radius_mm),
        };
        let r = (at.x - origin[0]).hypot(at.y - origin[1]);
        let reach = if reach.is_finite() { format!("reaches radius {reach:.3}") } else { "has a point that cannot be read".into() };
        return Err(GenError::Invalid(format!(
            "cross drill: the first rapid comes from the turret's index position (the top of the X and Z travel) \
             to radius {r:.3} at Z{:.3}; the post sends X and Z in one rapid after C, so the tool can pass every \
             radius from {r:.3} out at Z{:.3} and above in the station's plane, and {what} {reach} there - move it clear of that band, \
             or drill these holes in a setup where nothing stands beyond them",
            at.z,
            at.z - r_tool
        )));
    }
    Ok(())
}
