## #1716 - Lighter debug fixtures for four HSM suites: a larger adaptive load

**What was decided.** The four debug suites `whole_model_pocket`,
`pocket_overhead`, `climb_direction` and `adaptive_pinch` (camcraft-app)
run their adaptive (HSM) plans at an optimal load of 0.4 (40% of D) in
place of the default 0.1464. The geometry of each fixture does not change.
`whole_model_pocket` gets one new `#[ignore]`d release-lane twin,
`a_whole_part_adaptive_at_the_default_load_keeps_the_full_tool_radius_off_the_part`,
that keeps the disc-law check at the default load.

**Why.** The time is in the HSM plan at the default load, not in the
fixture size: one 2D Adaptive plan on the 30 mm block took about 130 s in
debug and takes about 1.5 s at 0.4. The laws these tests pin (island
region, full tool radius off the part, the chord law between pieces, the
cut direction, the refusal under a closed floor) do not read the ring
step. A mutation check of each law at the new load fails each changed
test (recorded in the PR body).

**Considered and dropped.**
- A smaller block or stock: the band around the part must stay wider than
  the tool radius plus 1 mm, or the "stock is cleared" check has nothing
  to read; and the size was not the cost.
- One cached generation shared by the three adaptive tests of
  `whole_model_pocket`: libtest runs them in parallel, so the wall time
  stays at one plan (130 s).
- Twins at the default load for the other three suites: their Adaptive
  cases are one of several strategies or kinds in each test, and the
  default load stays covered by the `whole_model_pocket` twin and the
  release lane of `adaptive_pinch`.

**Cost to change later.** Set the load back in four test helpers.
