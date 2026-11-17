## #2308 - a stock-source chamfer's bevel is no credit over a part

**Decided.** Two changes, both in the post's verification gate.

1. The program sim credits no edge-break record of a 2D Chamfer whose
   source names the setup's stock (the stock itself, or picks off it):
   `stock_sim::breaks::breaks_stock_edge` drops those records in
   `plan_program`. A part that stands under the stock's edge is then
   judged sharp, and the chamfer's cut into it is a sim gouge. When the sim
   finds a gouge, the gate also adds the live re-judge line of #1628
   (`kept_moved`), so the refusal names the operation and the part.
2. When the sim verdict is clean and every op is covered (fresh, stamped,
   with a path), the gate also runs `kept_moved` (the proposal in the
   issue). A cut too shallow for the sim's lattice (0.02 to 0.1 mm into a
   part added after generation) reads clean in the sim. The live re-judge
   finds it and refuses by name.

**Why.** The cause of #2308 was the #303 credit together with the skipped
re-judge. The credit took the stock edge's declared bevel as part of the
finished part over every column under it, also over a part added after
generation. So the sim found zero gouge cells, the re-judge did not run,
and the post went out. The op's own law (#1631) says that a stock-source
chamfer must not enter any part, so the credit could only forgive a cut
that this law forbids. A part known at generation is convicted on the op,
so for it change 1 adds nothing. The PR review then found that a shallow
cut still posted with change 1 only, because the lattice does not see it;
change 2 closes that.

**Dropped.**
- Change 2 alone (the first proposal). It leaves the sim's verdict wrong
  for a deep cut. With change 1, the sim and the re-judge agree.
- Change 1 alone (the first head of PR #2312). The review showed that a
  cut of 0.02 to 0.1 mm into an added part still posted. The risk named
  then - that the re-judge and the generation disagree and refuse a legal
  program - did not show in any of the suites in the PR (keep law, kept
  parts, stock-source holes and cross drill, setup carry and align,
  edge-break suites, golden CL).
- Drop the credit for each break. A part-source chamfer's bevel is on the
  part's own edge and is part of the finished part.
- Fix Deburr here. A Deburr can also take the stock as source, and a
  stock-source Deburr can cut a part that is known at generation and still
  post. That defect is also on `main`. It needs its own part law
  (`keep_law` has no stock-source Deburr kind), so it is issue #2313, not
  part of this change.

**Cost to change.** Small: one predicate in `stock_sim_breaks.rs` and two
arms of `posting_sim::judge_clause`.
