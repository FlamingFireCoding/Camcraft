//! The control-side wire offset law (issue #4076): one rule, in the
//! prelude every post script sees, for a wire EDM post that writes the
//! control's own wire offset (G41 / G42 with an offset register, G40).
//!
//! Provenance: original. The statements of the owner of a Sodick AQ600L
//! (LN2W control) on #4076, 2026-11-25: the offset engages on the lead-in
//! (or on its own line immediately before it), the lead-in is at least
//! the wire radius long (the same rule as a mill cutter radius), and G40
//! goes on a lead-out move; the owner states that other wire EDM makes
//! use the same scheme with small changes. RS-274 / ISO 6983 for G40 /
//! G41 / G42. The decision is `docs/decisions/4076-ln2w-wire-offset.md`.
//!
//! The helper keeps the law only. The words (the G41 / G42 line, the
//! register number and format, the condition that goes with it) are each
//! dialect's own, in its post script.
//!
//! The use, from a post script:
//! - `var st = wireOffsetNew();` once.
//! - `onRadiusCompensation(side)`: `why = wireOffsetRecord(st, side,
//!   properties.CompensationRadius, properties.CompensationType)`; a
//!   non-empty `why` is the refusal.
//! - each motion record: `d = wireOffsetMove(st, straight, sx, sy, x, y)`.
//!   `d.error` is the refusal; `d.engage` (`"left"` / `"right"`) says the
//!   offset engages on this move (its lead-in); `d.cancel` says G40 goes
//!   on this move (its lead-out).
//! - the section's end: a non-empty `wireOffsetEnd(st)` is the refusal (the
//!   offset is still on). It resets the state.

/// Appended to [`super::js::PRELUDE`]: plain JS, no host calls.
pub const WIRE_OFFSET_JS: &str = r#"
// #4076 (wire_offset.rs): the control-side wire offset law.
function wireOffsetNew() { return { pending: "", side: "", radius: 0, wear: false, active: false }; }
function _woNum(v) { return typeof v === "number" && isFinite(v); }
function _woMm(v) { return Number(v).toFixed(3).replace(/0+$/, "").replace(/\.$/, ""); }
// A COMP record: "left" / "right" engages on the next move, "off" cancels
// on the next move. Returns "" or the reason the record is refused.
function wireOffsetRecord(st, side, radius, type) {
  if (side === "left" || side === "right") {
    if (!_woNum(radius) || (type !== "wear" && radius <= 0)) {
      return "the wire offset states no usable radius (" + String(radius) + ") - refused";
    }
    if (st.active) {
      return "the wire offset changes to " + side + " while it is on - cancel it (G40) on a straight lead-out move first";
    }
    st.pending = side;
    st.side = side;
    st.radius = radius;
    st.wear = type === "wear";
    return "";
  }
  st.pending = st.active ? "off" : "";
  return "";
}
// A motion record from (sx, sy) to (x, y); `straight` is false for an arc.
function wireOffsetMove(st, straight, sx, sy, x, y) {
  var d = { engage: "", cancel: false, error: "" };
  if (st.pending === "left" || st.pending === "right") {
    if (!straight) {
      d.error = "the lead-in that engages the wire offset is an arc - the offset engages on a straight lead-in";
      return d;
    }
    if (!_woNum(sx) || !_woNum(sy) || !_woNum(x) || !_woNum(y)) {
      d.error = "the lead-in that engages the wire offset states no start point - refused";
      return d;
    }
    var len = Math.sqrt((x - sx) * (x - sx) + (y - sy) * (y - sy));
    var need = Math.abs(st.radius);
    if (len + 1e-6 < need) {
      d.error = "the lead-in is " + _woMm(len) + " mm long, shorter than the wire offset radius " + _woMm(need) + " mm - make the lead-in at least the radius long";
      return d;
    }
    st.active = true;
    st.pending = "";
    d.engage = st.side;
    return d;
  }
  if (st.pending === "off") {
    if (!straight) {
      d.error = "the lead-out that cancels the wire offset is an arc - G40 goes on a straight lead-out move";
      return d;
    }
    st.active = false;
    st.pending = "";
    d.cancel = true;
  }
  return d;
}
// The section's end. Returns "" or the reason it is refused; resets the state.
function wireOffsetEnd(st) {
  var on = st.active;
  st.pending = "";
  st.side = "";
  st.radius = 0;
  st.wear = false;
  st.active = false;
  return on ? "the section ends with the wire offset on - cancel it (G40) on a straight lead-out move" : "";
}
"#;
