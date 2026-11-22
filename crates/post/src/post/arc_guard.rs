//! The coincident arc guard (issue #3442): one rule, in the prelude every
//! post script sees, for an arc record whose end point, as the script
//! writes it, is its start point.
//!
//! Provenance: original. RS-274 / ISO 6983 circular interpolation: an
//! arc block whose end point equals the current position, with a centre
//! off that point, is a full circle (the reading of the Fanuc-family,
//! LinuxCNC and grbl controls' published programming manuals, behaviour
//! only). The output resolution is the script's own word format, so the
//! comparison is made on the words as written, never on the stream's
//! millimetres.
//!
//! The rule (decision `docs/decisions/3442-coincident-arc-blocks.md`):
//! - the written in-plane ends differ: the arc posts as before;
//! - they coincide and the record turns more than half a circle (or its
//!   ends are within 1e-5 mm in the stream: a full turn stated as one
//!   record), or it states no centre (the script refuses it by name):
//!   the full circle is the arc within the output resolution, and the
//!   arc posts as before;
//! - they coincide on a short arc: the tool moves less than the output
//!   resolution in the plane, so the record posts as a straight move
//!   when the plane's normal axis (a helix) or a rotary still moves, and
//!   as no block at all when nothing changes as written.

/// Appended to [`super::js::PRELUDE`]: plain JS, no host calls.
pub const ARC_GUARD_JS: &str = r#"
// #3442 (arc_guard.rs): what an arc record may become in the output.
// `word(axis, value)` is the script's own text for a position on "X", "Y"
// or "Z" (origin, unit and precision applied; "" for an axis it never
// writes). Returns "arc", "linear" (post the record as a straight move)
// or "none" (write no motion block).
function _arcNum(v) { return typeof v === "number" && isFinite(v); }
function arcShapeOf(p) {
  var c = p.IJKIncrement || {};
  return {
    plane: p.CircularPlane || "XY",
    ccw: p.CircularDirection !== "CW",
    start: { X: p.StartX, Y: p.StartY, Z: p.StartZ },
    end: { X: p.X, Y: p.Y, Z: p.Z },
    center: { X: c.I, Y: c.J, Z: c.K },
    rotary: _arcNum(p.A) || _arcNum(p.B) || _arcNum(p.C)
  };
}
// The record's own sweep (radians, 0..2pi) in the frame (u, v).
function _arcSweep(a, u, v) {
  var s = a.start, e = a.end, c = a.center;
  var a0 = Math.atan2(s[v] - c[v], s[u] - c[u]), a1 = Math.atan2(e[v] - c[v], e[u] - c[u]);
  var sweep = a.ccw ? a1 - a0 : a0 - a1;
  if (sweep < 0) sweep += 2 * Math.PI;
  return sweep;
}
// The number of a written word ("X10.011", "X+10.011", "10.") or NaN.
function _arcWordNum(t) {
  var m = /[-+]?(\d+\.?\d*|\.\d+)/.exec(String(t));
  return m ? parseFloat(m[0]) : NaN;
}
// #3474: the written ends differ, but the written end is on the written
// start's ray from the written centre (the same angle), or one of them is
// on the written centre. A control that reads the sweep from the angles
// sees none, or a full turn. On a short record (half a circle or less)
// the tool moves under the output step across the radius only: a straight
// move. The test uses the cross product's zero and the dot product's sign
// only, so a mirrored or scaled word (a lathe diameter) reads the same.
function sameAngleShort(a, u, v, word) {
  var s = a.start, e = a.end, c = a.center;
  if (!_arcNum(c[u]) || !_arcNum(c[v])) return false;
  var cu = _arcWordNum(word(u, c[u])), cv = _arcWordNum(word(v, c[v]));
  var su = _arcWordNum(word(u, s[u])) - cu, sv = _arcWordNum(word(v, s[v])) - cv;
  var eu = _arcWordNum(word(u, e[u])) - cu, ev = _arcWordNum(word(v, e[v])) - cv;
  if (!isFinite(su) || !isFinite(sv) || !isFinite(eu) || !isFinite(ev)) return false;
  var scale = Math.sqrt((su * su + sv * sv) * (eu * eu + ev * ev));
  var onRay = scale === 0 || (Math.abs(su * ev - sv * eu) <= 1e-9 * scale && su * eu + sv * ev > 0);
  // A record whose own ends are on one ray (a zero sweep in the stream)
  // states a full turn, as emitArcWithRadius reads it: the arc stays.
  var sweep = _arcSweep(a, u, v);
  return onRay && sweep > 1e-9 && sweep <= Math.PI;
}
function arcFateOf(a, word) {
  // The right-handed in-plane frame (u, v), u x v = +normal: CCW (seen
  // from the positive normal) is the ordinary atan2 sense in each plane.
  var ax = a.plane === "XZ" ? ["Z", "X", "Y"] : (a.plane === "YZ" ? ["Y", "Z", "X"] : ["X", "Y", "Z"]);
  var u = ax[0], v = ax[1], h = ax[2], s = a.start, e = a.end, c = a.center;
  if (!_arcNum(s[u]) || !_arcNum(s[v]) || !_arcNum(e[u]) || !_arcNum(e[v])) return "arc";
  function same(k, p, q) { return String(word(k, p)) === String(word(k, q)); }
  if (!same(u, s[u], e[u]) || !same(v, s[v], e[v])) return sameAngleShort(a, u, v, word) ? "linear" : "arc";
  // The written ends coincide. With no centre the record is not an arc
  // this rule can read: the script's own code refuses it by name.
  if (!_arcNum(c[u]) || !_arcNum(c[v])) return "arc";
  // A centre that stands off the start as written makes the block a
  // full circle: right for a full turn. Ends within 1e-5 mm in the
  // stream (the band emitArcWithRadius uses) are a full turn stated as
  // one record, whatever float noise puts between them.
  var round = !(same(u, c[u], s[u]) && same(v, c[v], s[v]));
  if (round) {
    var full = Math.abs(e[u] - s[u]) < 1e-5 && Math.abs(e[v] - s[v]) < 1e-5;
    if (!full) full = _arcSweep(a, u, v) > Math.PI;
    if (full) return "arc";
  }
  // A short arc under the output resolution: no arc block.
  if (_arcNum(e[h]) && !(_arcNum(s[h]) && same(h, s[h], e[h]))) return "linear";
  if (a.rotary) return "linear";
  return "none";
}
function arcFate(word) { return arcFateOf(arcShapeOf(properties), word); }
"#;
