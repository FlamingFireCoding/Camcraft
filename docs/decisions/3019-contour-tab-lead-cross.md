## #3019 - Leads at a closed ring's seam leave on the air side

**Decided.** On a closed contour ring, the lead tangents at the seam are
compared with each other. If the ring bends to the air side at the seam,
the lead-out tangent is mirrored across the ring's start tangent, and the
lead-in tangent is mirrored across the ring's end tangent. Then each lead
leaves the seam on the air side of the whole ring. A smooth seam, a seam
that bends away from the air, and an open walk (tab windows, chains, an
exit point) keep their tangents unchanged. The change is in
`crates/motion/src/machining2d/lead_fit.rs` (`seam_tangents`).

**Why.** The seam of a round-cornered outside ring falls on a corner arc.
The fitted arc's end tangent is a hair (about 0.05 degree) off the ring's
first chord, toward the material. The 8 mm linear lead-out along that
tangent went back over the ring's first two moves and crossed the second
move about 0.25 mm past the seam. The crossing law convicted it on each
level. The lead-in has the same fault on a conventional cut. The cutter
there re-cuts the wall that it cut before, so the crossing was nearly
harmless. But the law is correct: a lead must not pass back across the
cut, and the fix makes the path pass the law. The law is not relaxed.

**Dropped.**
- Relax the crossing law for lead moves. That hides real lead crossings.
- Check the lead against the walk and shrink it when it crosses. The
  crossing is 0.25 mm from the seam, so each halving down to 0.5 mm still
  crosses; the lead would be refused on most round-cornered plates.
- Use the start tangent for the lead-out without a mirror. The lead then
  runs exactly along the ring's first chord, and the crossing reading of
  the chord's end point depends on float rounding.

**Cost to change.** Local to one function. Only closed rings with an
air-side bend at the seam change; their lead directions turn by the bend
angle (a fraction of a degree on fitted arcs).
