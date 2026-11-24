//! The warnings of a start-side pass whose entry is not the selected one
//! or plunges into uncut stock (#3884, 2D "Pass starts at" Inside or
//! Outside): each stand-in entry that the ladder names, and a selected
//! Plunge at a pass start in material, become one advisory operation
//! warning each, never a note only. Provenance: our own rule, from the
//! maintainer's words on #3748 ("it uses a safe entry and names it in a
//! warning") and the Prime Directive; no vendor material consulted.

use crate::approach::{AIR_ENTRY_REFUSAL, OUTSIDE_AIR_REFUSAL};
use crate::cl::{MoveKind, PathBuilder, Toolpath};
use crate::entry::Entry3D;
use crate::geo2d::Point2;
use crate::verify::Warning;

use super::GenError;
use super::descent::{Descent, Lvl};
use super::levels::Motion;
use super::pass_start::PassStart;

/// The ladder's marker note: a start-side pass took at least one stand-in
/// entry. The notes before it name each one ([`is_entry_stand_in`]).
pub const ENTRY_STAND_IN_NOTE: &str =
    "entry: Pass starts at - the selected entry did not fit at one or more pass starts; each stand-in entry is named";

/// The ladder's note of a start-side pass whose selected Plunge enters a
/// pass start that no earlier cut cleared: it plunges into uncut stock.
pub const PLUNGE_INTO_STOCK_NOTE: &str =
    "entry: Pass starts at - the selected Plunge entry plunges into uncut stock at a pass start that no earlier cut cleared";

/// #3884: the one Outside-air check of every Outside entry path - the
/// plan (no air room at all), the first entry of a level (no air lead
/// fits, `air_entry::stand_in`) and the re-entry after a lift
/// (`descent_dive::entry_restart`). A selected Arc on an Outside ring pass
/// goes down in air outside the stock (`in_air`), or it is refused by name
/// with `why`: never an automatic helix, ramp or dive. A selected Helix,
/// Ramp or Plunge runs, with its one warning.
pub(super) fn outside_air_check(outside: bool, entry: Entry3D, in_air: bool, why: &str) -> Result<(), String> {
    if outside && entry == Entry3D::Arc && !in_air {
        return Err(format!("{OUTSIDE_AIR_REFUSAL} ({why})"));
    }
    Ok(())
}

/// #3884: the selected Arc on an Outside pass re-enters a pass start in
/// material in air outside the stock, or the op is refused by name - never
/// a ramp or a helix. `None`: not that case.
pub(super) fn outside_arc_restart(b: &PathBuilder, l: &Lvl, e: Point2, z: f64, plane: f64, m: &Motion) -> Option<Descent> {
    let air = m.air.as_ref().filter(|a| a.outside() && m.entry == Entry3D::Arc)?;
    let site = format!("the pass start ({:.3}, {:.3}) at Z {z:.3} is in material", e.x, e.y);
    if let Some(a) = super::air_entry::restart(l, m).filter(|d| super::descent_dive::moves_clear(b, d, l, z, plane, m)) {
        return Some(Descent::Named(Box::new(a), format!("entry: {site} - entered from the air outside the stock box instead")));
    }
    let why = format!("{site} and no air lead reaches it");
    Some(Descent::Refused(match outside_air_check(air.outside(), m.entry, false, &why) {
        Err(r) => r,
        Ok(()) => format!("{AIR_ENTRY_REFUSAL} - {why}"),
    }))
}

/// The plan's call of [`outside_air_check`]: an Outside ring pass
/// (`rings`: the resolved strategy walks rings, an Adaptive op's ring
/// fallback too) with no air room - no stock known, not round a part, or
/// the fixtures fill it.
pub(super) fn refuse_outside_arc(start: PassStart, entry: Entry3D, rings: bool, air: bool) -> Result<(), GenError> {
    outside_air_check(start == PassStart::Outside && rings, entry, air, "the pass has no air room outside the stock").map_err(GenError::Invalid)
}

/// The tails of the notes that name a stand-in entry: the entry that the
/// ladder used where the selected one did not fit.
const STAND_IN_TAILS: [&str; 6] = [
    " - ramp along the pass instead",
    " - helix entry instead",
    " - ramp along the first cut instead",
    "plunged into material at the plunge feed",
    "axial dive on the floor instead at (",
    "the floor has no dive room either; dived at the pass start",
];

/// Whether `note` names a stand-in entry.
#[must_use]
pub fn is_entry_stand_in(note: &str) -> bool {
    STAND_IN_TAILS.iter().any(|t| note.contains(t))
}

/// How many stand-in entries the notes name.
#[must_use]
pub fn entry_stand_ins(notes: &[String]) -> usize {
    notes.iter().filter(|n| is_entry_stand_in(n)).count()
}

/// Mark a stand-in entry `note` of a start-side ladder (`m.cleared_links`,
/// the start-side plan's link law of #3316).
/// `None`: the substituted axial dive, a stand-in always.
pub(super) fn mark_stand_in(b: &mut PathBuilder, m: &Motion, note: Option<&str>) {
    if m.cleared_links && note.is_none_or(is_entry_stand_in) {
        b.note_once(ENTRY_STAND_IN_NOTE);
    }
}

/// A start-side pass link that plunges at `e` into uncut stock at `z`: the
/// selected Plunge is named once; any other selected entry that comes to
/// this plunge (no certified area to plan it in) is a named stand-in.
pub(super) fn plunge_into_stock(b: &mut PathBuilder, m: &Motion, e: Point2, z: f64) {
    if m.entry == Entry3D::Plunge {
        b.note_once(PLUNGE_INTO_STOCK_NOTE);
        return;
    }
    let note = format!(
        "entry: the pass start ({:.3}, {:.3}) at Z {z:.3} is in material - plunged into material at the plunge feed",
        e.x, e.y
    );
    b.note_once(&note);
    mark_stand_in(b, m, Some(&note));
}

/// The start-side entry warnings of `path`: one `EntryStandIn` that counts
/// the named stand-ins, and one `PlungeIntoStock` for a selected Plunge
/// (none where `advised`: the op already names the selected Plunge in its
/// `EntryNotRecommended` warning, so the Plunge is named once).
#[must_use]
pub fn entry_warnings(path: &Toolpath, advised: bool) -> Vec<Warning> {
    let at = path.moves.iter().position(|m| m.kind != MoveKind::Rapid).unwrap_or(0);
    let mut out = Vec::new();
    if path.notes.iter().any(|n| n == ENTRY_STAND_IN_NOTE) {
        out.push(Warning::EntryStandIn { at, entries: entry_stand_ins(&path.notes).max(1) });
    }
    if !advised && path.notes.iter().any(|n| n == PLUNGE_INTO_STOCK_NOTE) {
        out.push(Warning::PlungeIntoStock { at });
    }
    out
}
