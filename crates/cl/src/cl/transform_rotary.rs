//! Turn a path about the machine's A (X) or B (Y) axis (#3011).
//!
//! Provenance: right-handed rotations about X and Y, the RS-274 A / B
//! sense that `pose.rs` composes (A tips +Z toward -Y, B toward +X).
//! Our own model; no vendor material consulted.
//!
//! The tool turns with the work: every move states the turned axis
//! (`Move::axis`, normalized by `pose.rs`), as a [`super::super::TiltedFrame`]
//! path does. An arc does not keep its plane meaningfully under a tilt,
//! so every arc is flattened to chords at `chord_tol` with exact
//! endpoints. The move count then changes, so the records that index
//! moves cannot follow: a canned cycle and a plasma statement are
//! refused, a compensated span is dropped with a note (its cutter-centre
//! moves still post, compensation in computer), as the tilted frame does.

use crate::geometry::Point3;

use super::super::path::Toolpath;
use super::{checked, map_breaks, map_path, refuse_records, sin_cos_deg, Rigid, TransformError};

/// The rotary axis to turn about: A turns about X, B about Y.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RotaryAxis {
    A,
    B,
}

/// Turn the path by `angle_deg` about the `axis` line through `origin`.
/// A whole turn is the identity and keeps every record; any other angle
/// flattens the arcs at `chord_tol` (mm, positive) and states the
/// turned tool axis on each move.
pub fn rotate_about_axis(
    tp: &Toolpath,
    axis: RotaryAxis,
    origin: Point3,
    angle_deg: f64,
    chord_tol: f64,
) -> Result<Toolpath, TransformError> {
    if !angle_deg.is_finite() || !origin.is_finite() {
        return Err(TransformError::BadInput("the rotary transform"));
    }
    if !chord_tol.is_finite() || chord_tol <= 0.0 {
        return Err(TransformError::BadInput("the chord tolerance"));
    }
    refuse_records(tp)?;
    let (s, c) = sin_cos_deg(angle_deg);
    let m = match axis {
        RotaryAxis::A => [[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]],
        RotaryAxis::B => [[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]],
    };
    let o = [origin.x, origin.y, origin.z];
    let r = Rigid { m, t: Rigid::pivot_offset(&m, o, [0.0; 3]) };
    if r.keeps_up() {
        return map_path(tp, &r);
    }
    if !tp.cycles.is_empty() {
        return Err(TransformError::CannedCycleTilt);
    }
    if tp.plasma.is_some() {
        return Err(TransformError::PlasmaCut);
    }
    let mut moves = Vec::with_capacity(tp.moves.len());
    for m in &tp.moves {
        if m.arc.is_none() {
            moves.push(r.straight(m, m.from, m.to)?);
            continue;
        }
        let chords = m.segments3(chord_tol);
        let last = chords.len().saturating_sub(1);
        for (i, (a, b)) in chords.into_iter().enumerate() {
            // The run's own endpoints, never a re-derived Z: the chain
            // into and out of the arc stays exact.
            let a = if i == 0 { m.from } else { a };
            let b = if i == last { m.to } else { b };
            let chord = r.straight(m, a, b)?;
            if chord.from != chord.to {
                moves.push(chord);
            }
        }
    }
    let mut notes = tp.notes.clone();
    if !tp.comps.is_empty() {
        notes.push(
            "compensation: control compensation does not ride a tilt - this path posts its cutter-centre moves (in computer)"
                .into(),
        );
    }
    let breaks = map_breaks(tp, &r, &mut notes);
    checked(Toolpath { moves, notes, breaks, ..Toolpath::default() })
}

