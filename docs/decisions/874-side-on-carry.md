## #874 - Carry machined stock into a setup turned side-on

**What was decided.** The stock one setup leaves is carried through the
sixteen side-on quarter turns (the old X or Y on the new tool axis) by a
conservative read of the column body (`Body::carry_side`,
`crates/sim/src/stock_sim/carry_side.rs`). The eight lattice turns keep
their exact re-index. Every turn off the quarter turns keeps the named
refusal (`CARRY_TURN_REFUSAL`), with shorter words.

The read, per new column:

- The level axis that stays level is re-indexed, centre onto centre.
- The old heights become a level axis in slabs of one cell. The slabs are
  centred on the old Z range. An old column feeds every slab that its span
  reaches into. A face inside a slab counts the slab whole.
- The old level axis becomes the height. The new top and floor are the
  hull of the old column centres that hold material in the slab. Each end
  moves out to the next old centre that holds no material there, or to the
  lattice edge. A cavity between the ends is filled.

The carried body is never smaller than the body the columns hold. It is at
most one grid cell larger on a turned face. The raw faces of a box stock at
the lattice edge are read exactly.

**A gouge cut before the turn is kept.** The hull can hide a gouge that an
earlier setup cut (a face cut too deep becomes a wall that the new tool axis
does not see). The program walk judges the body before a side-on turn and
folds those gouges into the final verdict, the same fold a block left for a
new block gets (#320). A gouge that both verdicts see is counted in both.

**The floor is not judged after a side-on turn.** The new floor is a face
the earlier setup held level, known to one cell. The gouges of the earlier
setup are already kept by the fold above. A later flip sets the flag again.

**The verdict says so.** The gate's note line names the first side-on carry
and what the verdict reads (`side_on_note`).

**The field engine refuses a side-on turn by name**
(`FIELD_SIDE_CARRY_REFUSAL`). Its column read assumes the cuts came along
the tool axis. The shipped engine is the column engine.

**Why.** The maintainer's rule on the issue: "a re-sample that never leaves
the carried stock smaller than the real one, so rest machining cannot run
into uncut material". A setup that stands the part on its side is the
ordinary second setup. The refusal blocked it.

**Considered and dropped.**

- An exact carry on the field engine (re-index the 3-D bricks, read the
  columns along the new axis). It is exact to the field's own resolution,
  but the cut records (`cut_ups`, `cut_downs`) and the column read
  (`column_span`) assume cuts along Z. It is a new read law for the field
  engine, and the field engine does not ship (#589). A follow-up issue
  tracks it.
- The cell edge as the turned face (the middle estimate). It can be half a
  cell short of a wall the tool flank cut. Rejected: the stock must never
  be smaller than the real one.
- Sampling the old heights at slab centres only. It can drop up to half a
  cell at a faced face, and a plate thinner than a cell. Rejected for the
  same reason. The centred slabs keep both end centres inside the range, so
  a fixture at an end face meets no column centre outside the real stock.
- Crediting the earlier setup's edge breaks and marks in the verdict taken
  before the turn. The fold for a new block (#320) does not credit them
  either. A follow-up issue tracks both.

**Known limits (stated, not hidden).**

- A wall the tool flank cut, or a step, is read up to one cell outside the
  real face after the turn. The verdict can show rest of up to one cell on a
  finished face. The carried-fixture law (#321) can refuse a fixture that
  touches such a face. The workaround is the setup's own stock.
- A cavity seen from the side (a pocket of the earlier setup) is filled.
  Rest machining in the turned setup can cut air there. It never meets
  uncut material.
- Rest that the earlier setup left is not carried into the final verdict's
  rest numbers when the turn hides it. Gouges are.

**Cost to change later.** The exact field carry replaces `carry_side`
behind `CarriedStock::carry_through`; the app callers hold a `CarryTurn`
and do not change. The digest text of a lattice turn is unchanged (the
`Debug` of `CarryTurn` prints the inner turn), so no shipped stamp moves.
