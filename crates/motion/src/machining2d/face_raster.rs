//! The Face raster's order under the user's start and exit picks (#3326,
//! design #1448): which edge row is cut first, which way the first row
//! runs, and so where the last row ends. The rows themselves are the
//! facer's own band and never change - a pick only orders them. An exit
//! that the row count cannot reach from the start corner is refused by
//! name; a pick is never moved without a note.
//!
//! Provenance: our own planning rule over the serpentine (boustrophedon)
//! raster of general machining practice - the row count's parity decides
//! the side where an alternating raster ends. No vendor material consulted.

use crate::geo2d::Point2;
use crate::geometry::Point3;
use crate::path_picks::{PathPicks, PickRole};

use super::blend::Turn;
use super::gen_error::GenError;

/// The order of the rows: the facer's automatic raster is `up` with the
/// first row running the climb direction.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Raster {
    /// The rows step from the low Y edge to the high one.
    up: bool,
    /// The first row runs from low X to high X.
    first_forward: bool,
    /// What the plan must say on the path (an exit met on its side only).
    pub note: Option<String>,
}

/// The raster of `rows` rows over the footprint box `lo..hi` at `cut_z`:
/// no pick is the automatic raster (`climb` sets the first row), to the
/// byte. A start pick takes the box corner nearest to it: its edge row is
/// cut first, away from that corner, and the rows step to the far edge.
/// An exit pick names the X side where the last row ends; with no start
/// pick the last row is also the edge row nearest the exit, so the tool
/// leaves at the exit's own corner. A pick at equal distance from two
/// sides takes the low one.
pub(super) fn plan(
    picks: &PathPicks,
    (lo, hi): (Point2, Point2),
    rows: usize,
    climb: bool,
    cut_z: f64,
) -> Result<Raster, GenError> {
    if let Some(why) = picks.refusal() {
        return Err(GenError::Invalid(format!("Face: {why}")));
    }
    let low_x = |p: Point2| (p.x - lo.x).abs() <= (p.x - hi.x).abs();
    let low_y = |p: Point2| (p.y - lo.y).abs() <= (p.y - hi.y).abs();
    // An odd count ends its last row the way the first one ran.
    let odd = rows % 2 == 1;
    let start = picks.first(PickRole::Start).map(|p| p.xy);
    let exit = picks.first(PickRole::Exit).map(|p| p.xy);
    let (up, first_forward) = match (start, exit) {
        (None, None) => (true, climb),
        (Some(s), _) => (low_y(s), low_x(s)),
        // The last row runs toward the exit's side, on the exit's edge.
        (None, Some(e)) => (!low_y(e), low_x(e) != odd),
    };
    let mut note = None;
    if let (Some(_), Some(e)) = (start, exit) {
        let ends_low_x = first_forward != odd;
        if low_x(e) != ends_low_x {
            let count = if rows == 1 { "1 row".to_owned() } else { format!("{rows} rows") };
            let side = if odd { "the side across from the start corner" } else { "the start corner's own side" };
            return Err(GenError::invalid_at(
                format!(
                    "Face: the exit point is nearest the other side - a raster of {count} from the start corner ends at {side}; clear the exit point, start at the other corner of the first row, or change the stepover (the row count)"
                ),
                Point3::new(e.x, e.y, cut_z),
            ));
        }
        if rows > 1 && low_y(e) == up {
            note = Some(
                "the exit point is nearest the first row: the raster ends on the exit point's side, at the last row".to_owned(),
            );
        }
    }
    Ok(Raster { up, first_forward, note })
}

impl Raster {
    /// Put the band's ascending rows and their reversal plan in cut order.
    pub fn order(&self, ys: &mut [f64], turns: &mut [Option<Turn>]) {
        if !self.up {
            ys.reverse();
            turns.reverse();
        }
    }

    /// Does row `i` (in cut order) run from low X to high X?
    pub fn forward(&self, i: usize) -> bool {
        self.first_forward == i.is_multiple_of(2)
    }

    /// Is the reversal after row `i` a left (counter-clockwise) turn? It
    /// bulges away from the footprint: left off a +X row that steps up.
    pub fn ccw(&self, i: usize) -> bool {
        self.forward(i) == self.up
    }

    /// `rho` along the step direction: where a corner arc's centre stands.
    pub fn step(&self, rho: f64) -> f64 {
        if self.up { rho } else { -rho }
    }

    /// Each row as (y, from, to) in cut order: the serpentine between
    /// `x_in` (low) and `x_out` (high), the first row entered `lead`
    /// farther out along its own direction.
    pub fn rows(&self, ys: &[f64], x_in: f64, x_out: f64, lead: f64) -> Vec<(f64, f64, f64)> {
        ys.iter()
            .enumerate()
            .map(|(i, &y)| {
                let (from, to) = if self.forward(i) { (x_in, x_out) } else { (x_out, x_in) };
                let led = if self.forward(i) { x_in - lead } else { x_out + lead };
                (y, if i == 0 { led } else { from }, to)
            })
            .collect()
    }
}
