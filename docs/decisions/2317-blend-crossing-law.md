## #2317 - The crossing law on walks with corner arcs

**Decided.** Two changes, both only for corner arcs (`blend_turns`).

1. *One corner arc is refused, not the pass.* The pocket family keeps the
   walks as fitted (`base`) and rounds them again each round
   (`machining2d/blend_cross.rs`). The crossing law reads the posted path.
   A corner arc that crosses another feed bans its own corner: that corner
   stays sharp and the fillet note counts it ("an arc there would cross
   another feed"). A fitted arc that crosses goes back to the chords of the
   plan through `arc_cross::refuse`, on the fitted walks, which the law can
   map. The loop runs `CROSS_ROUNDS` (3) rounds. After that no corner is
   rounded and the classic law (`post_crossing_free`) holds on the fitted
   walks. The path that returns is a path the law read with no crossing, or
   the answer of the classic law.
2. *A tangent touch of a corner arc is not a crossing*
   (`machining2d/arc_touch.rs`). A pair (corner arc chord, other chord `q`)
   is not counted when `q`, over the reach of the bracket of the arc along
   the tangent at one end of the arc, stays within `d` of that tangent line
   and `d + dr <= TOUCH = 1e-5 mm`. `dr` is the difference of the two end
   radii of the arc (the lattice step of the center).

**Proof bound of the touch.** A circle lies on one side of each of its
tangents. The bracket of an arc is the inscribed fan and the circumscribed
hull; the last hull edge lies on the tangent at the end. Thus each bracket
point `X` holds `(X - E) . n <= dr` (`E` the end, `n` the unit radius
there). Outside the reach along the tangent there is no bracket point.
Inside the reach `q` is within `d` of the tangent line. Thus the true arc
passes `q` by `d + dr <= 1e-5 mm` at most. The allowance can hide no
crossing deeper than 0.01 micrometer, which is ten steps of the 1e-6 mm
lattice of the CL data. A pair that cannot be measured (no finite radius,
an empty clip, a NaN) is a crossing. A chord that goes through the arc at
a real angle leaves the 1e-5 mm band inside the reach and is a crossing.

**Why.** PR #2273 sent the rounded walks of Waterline and Offset rings
through `post_crossing_free`. That law maps a crossing arc onto the
vertices of the pass. A rounded walk has new vertices (the tangent
points), so the map failed and the pass went back to plain chords: all
corner arcs and all fitted arcs of the pass were lost on one hit
(`corner_arc_radius` red on main). Two causes gave hits:

- real crossings at the full tool radius (fixture: Waterline, no radius,
  4 corners);
- false hits: the center of a corner arc is on the 1e-6 mm lattice, so the
  bracket goes about 7e-8 mm past the line of its own leg, where the next
  ring or a link runs. The strict predicate reads a proper crossing.

Readings on the fixture of `corner_arc_radius` (20 x 12 pocket with an
island, 8 mm tool): without the touch allowance 1 Waterline corner and 2
Offset rings corners at 1 mm lose their arc for a touch; with it, none.
At the tool radius 4 Waterline corners stay sharp for a real crossing and
22 of 45 keep their arc. `verify::crossed_pairs` is empty in all cases.

**Dropped.**

- *A tolerance in the general predicate `crosses`.* It would apply to each
  arc pair, also Adaptive (HSM) and the 3D ladder, and it can hide a
  through-crossing at a shared vertex. No proof bound is available for the
  general case. The callers without corner arcs pass an empty blend set and
  get the same result as before, bit for bit.
- *The law only on the fitted walks, no check on the rounded walks* (held
  commit `51c4d8767` on `issue/2317-waterline-corner-arcs`). The corner
  arcs would be back, but the real crossings at the tool radius would be
  posted. That is against the Prime Directive.
- *No allowance, refusal for each corner only.* Safe, but corners lose
  their arc for a 7e-8 mm lattice effect and the note gives a false cause.
- *A map of the rounded walk onto its own vertices in `arc_proof::judge`.*
  The tangent points are not vertices of the plan, so a refused arc has no
  chords to go back to.

**What changes in output.** Pocket operations with corner arcs on, strategy
Waterline, Offset rings (also as the fallback of Adaptive) and each
strategy with `hold_crossings`: the corner arcs are back, less the corners
that would cross. With corner arcs off nothing changes. Zigzag and Auto
without `hold_crossings` post as before (rounded one time, no law), which
is the state before this change.

**Cost to change later.** `TOUCH` is one constant with its tests in
`arc_touch.rs`. The loop is one function; the classic law is its fallback
and is not changed. The trace of `fillet_corners_except` (which arc is a
corner arc, which fitted arc was trimmed) is the only new coupling between
`fillet.rs` and the law.

**Not exercised.** The word format of each post script was not read. The
bound is stated against the CL lattice, not against a posted word.
