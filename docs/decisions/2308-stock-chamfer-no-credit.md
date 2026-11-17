## #2308 - a stock-source chamfer's bevel is no credit over a part

**Decided.** The program sim credits no edge-break record of a 2D Chamfer
whose source names the setup's stock (the stock itself, or picks off it):
`stock_sim::breaks::breaks_stock_edge` drops those records in
`plan_program`. A part that stands under the stock's edge is then judged
sharp, and the chamfer's cut into it is a sim gouge. When the sim finds a
gouge, the gate also adds the live re-judge line of #1628 (`kept_moved`),
so the refusal names the operation and the part.

**Why.** The cause of #2308 was the #303 credit, not only the skipped
re-judge. The credit took the stock edge's declared bevel as part of the
finished part over every column under it, also over a part added after
generation. So the sim found zero gouge cells and the post went out. The
op's own law (#1631) says that a stock-source chamfer must not enter any
part. So the credit could only forgive a cut that this law forbids. A part
known at generation is convicted on the op, so for it the change adds
nothing. Programs that posted before and do not cut a part post the same.

**Dropped.**
- Run `kept_moved` in the clean-sim arm too (the proposal in the issue).
  It leaves the sim's verdict wrong. It also adds a full part-law re-judge
  of each stock-kept op to each post on aligned stock, where no suite has
  pinned that the re-judge and the generation agree. It can thus refuse a
  legal program on a disagreement of two judges.
- Drop the credit for each break (deburr, part-source chamfer). Those
  bevels are on the part's own edge and are part of the finished part.

**Cost to change.** Small: one predicate in `stock_sim_breaks.rs` and one
line in `posting_sim::judge_clause`.
