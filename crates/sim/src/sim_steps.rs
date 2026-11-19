//! Operation steps on the simulation timeline (#2608): where each
//! operation starts, and the start that a "next operation" or "previous
//! operation" step goes to. Pure lookups over [`Script`], so the
//! transport bar and the stand-alone simulator (#2587) step the same way.
//! Provenance: our own timeline (`sim::Script`); nothing from vendor
//! binaries.
//!
//! An operation starts at its first segment. That is the connector rapid
//! when there is one: the script gives the rapid between two programs to
//! the operation that it arrives for (its tool is in the spindle). At
//! that time the earlier operations are complete, and the operation has
//! cut nothing. An operation with no moves is not in the script, so it is
//! never a stop.

use crate::sim::Script;

/// Two starts closer than this are the same position (seconds). A
/// step lands on a start exactly, so the tolerance only absorbs the
/// rounding of a playhead that a slider or a clock set.
const SAME_T: f64 = 1e-9;

impl Script {
    /// Each operation's start, in program order: (index into
    /// [`Script::ops`], seconds). One entry for each run of segments of
    /// one operation.
    #[must_use]
    pub fn op_starts(&self) -> Vec<(usize, f64)> {
        let mut out: Vec<(usize, f64)> = Vec::new();
        for (i, s) in self.segs.iter().enumerate() {
            if out.last().is_some_and(|&(op, _)| op == s.op) {
                continue;
            }
            let start = if i == 0 { 0.0 } else { self.cum.get(i - 1).copied().unwrap_or(0.0) };
            out.push((s.op, start));
        }
        out
    }

    /// The start of the first operation after the playhead `t`, or
    /// `None` in the last operation (and on an empty script).
    #[must_use]
    pub fn next_op_start(&self, t: f64) -> Option<f64> {
        let t = if t.is_finite() { t } else { self.total };
        self.op_starts().into_iter().map(|(_, s)| s).find(|&s| s > t + SAME_T)
    }

    /// The start of the operation under the playhead `t`, or - when the
    /// playhead already stands at that start - the start of the one
    /// before it. `None` at the start of the first operation (and on an
    /// empty script).
    #[must_use]
    pub fn prev_op_start(&self, t: f64) -> Option<f64> {
        let t = if t.is_finite() { t } else { self.total };
        self.op_starts().into_iter().rev().map(|(_, s)| s).find(|&s| s < t - SAME_T)
    }

    /// The name of the operation that starts at `start` (a value that
    /// `next_op_start` or `prev_op_start` gave): the step tips name it.
    #[must_use]
    pub fn op_starting_at(&self, start: f64) -> Option<&str> {
        let (op, _) = self.op_starts().into_iter().find(|&(_, s)| (s - start).abs() <= SAME_T)?;
        self.ops.get(op).map(String::as_str)
    }
}
