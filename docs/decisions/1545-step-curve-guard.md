## #1545 - Bound STEP edge and swept-curve subdivision

**Decided:** Meter B-spline, NURBS and pcurve subdivision on shell edges,
and use the same guarded curve division for extrusion and revolution
generators. Keep truck's probe positions, subdivision order and healthy
results. Refuse non-finite probes, exhausted evaluation budgets and splits
that cannot resolve within floating-point precision or the recursion limit.
Each edge gets a fresh budget per pass. Swept generators share their face's
meter, and a failed division cannot keep a coarser mesh as a fallback.

**Why:** A zero-weight rational edge and swept generator each exceeded the
pin's 20-second limit in the old code. A discontinuous B-spline could instead
be accepted at a coarse tolerance. Inspect full-multiplicity knots crossed
by the requested range and refuse unequal one-sided endpoints. Equal joined
endpoints and ranges trimmed away from a jump remain legal. Name an edge
refusal by its position and shell; retain the existing named face refusal
for a swept surface.

Rebuild swept division adapters with their original extrusion vector or
revolution axis, transform and orientation. A refused revolution generator
temporarily collapses onto its axis origin, so truck cannot infer a bogus
radius and allocate an angular grid before the refusal is read. These
temporary points never become an accepted mesh.

**Considered and dropped:** A wall-clock timeout can leave kernel work
running after the caller returns. Changing the healthy subdivision algorithm
would risk changing normal imports. Rejecting every full-multiplicity knot
would also reject continuous composite curves. No dependency or process
change is needed.

**Cost to change later:** Replace the curve adapter and its meter wiring,
then rerun the refusal pins, mutation checks and exact comparisons with
truck's healthy curve and swept-surface divisions. Changing the budget also
needs measurements of healthy imports and bounded pathological inputs.
