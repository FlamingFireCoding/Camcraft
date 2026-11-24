//! The warnings of a start-side pass whose entry is not the selected one
//! or plunges into uncut stock (#3884, 2D "Pass starts at" Inside or
//! Outside): each stand-in entry that the ladder names, and a selected
//! Plunge at a pass start in material, become one advisory operation
//! warning each, never a note only. Provenance: our own rule, from the
//! maintainer's words on #3748 ("it uses a safe entry and names it in a
//! warning") and the Prime Directive; no vendor material consulted.

use crate::cl::{MoveKind, PathBuilder, Toolpath};
use crate::entry::Entry3D;
use crate::geo2d::Point2;
use crate::verify::Warning;

use super::GenError;
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

/// #3884 (the maintainer's rule on #3741: an Outside pass goes down in air
/// outside the stock, never by a helix or a ramp): the refusal of a
/// selected Arc on an Outside ring pass with no air room.
pub const OUTSIDE_AIR_REFUSAL: &str =
    "entry refused: no air room for an Outside entry: set a stock, or choose an Inside start or a Helix, Ramp or Plunge entry";

/// Refuse the selected Arc on an Outside ring pass (`rings`: the resolved
/// strategy walks rings) that has no air room - no stock known, not round
/// a part, or the fixtures fill it. A selected Helix, Ramp or Plunge runs,
/// with its one warning.
pub(super) fn refuse_outside_arc(start: PassStart, entry: Entry3D, rings: bool, air: bool) -> Result<(), GenError> {
    if start == PassStart::Outside && entry == Entry3D::Arc && rings && !air {
        return Err(GenError::Invalid(OUTSIDE_AIR_REFUSAL.to_owned()));
    }
    Ok(())
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
