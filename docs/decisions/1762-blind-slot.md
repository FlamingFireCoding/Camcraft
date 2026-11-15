## #1762 - Close a blind slot against the stock outline

**Decided:** A single welded U-shaped open run can define a blind slot for
the 2D pocket family. The run must close with its endpoint chord into a
convex cap. Its outward end tangents must be parallel. The mouth must be
perpendicular to those tangents. The existing level and selection checks
still apply.

Use `sides_of_chain` to extend the two end tangents and split the stock
outline. Keep the side that contains the cap's interior seed. Require one
region that contains the mouth and overlaps the cap. Apply the same rule
to the stock outline grown by the requested extension. This moves the open
side without moving the three selected walls. Keep the two-wall band law,
the gouge check, and the collision check.

**Why:** The cap identifies the slot side without a stock-dependent guess.
The existing plane split already extends open chains along their end
tangents. Reuse keeps the boundary and extension rules consistent.

**Considered and dropped:** Do not select the smaller side of the stock.
Stock size could change that choice. Do not accept arbitrary concave runs
or converging or diverging end tangents. Those shapes need a separate rule
and refusal tests. Do not add another plane-split implementation when the
existing helper provides the required construction.

**Cost to change later:** Extend the supported-shape checks and add tests
for each accepted shape and each ambiguous case. The plane split and the
safety checks can stay in place.
