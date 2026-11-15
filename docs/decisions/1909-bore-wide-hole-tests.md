## #1909 - Detect-holes UI tests follow the Bore ring plan

**What was decided.** The two failing tests in
`crates/ui/src/detect_holes_note_tests.rs` were stale, and the product was
correct. PR #1839 (#1566) changed the Bore law on purpose. Bore now clears a
wide hole with concentric rings. Detect holes skips a hole only when the ring
plan needs more than 4,096 rings, or when the tool has no floor band. The
tests now pin that law with a Ø10 bull-nose that has a 4.999 mm corner
(the catalog `BB-1000-R1` with its corner changed):

- A Ø74.3 through hole is stored. The corner reaches R3.001 at the floor
  plane, so the plan uses 14 rings.
- A Ø24 blind hole is skipped, and the status names it. Its flat floor band
  is R0.001, so the plan needs 8,750 rings. The limit is Ø16.554.
- A plate with only that blind hole is refused, and nothing is stored.

The test count stays at two. Each test keeps its stored-holes assertion and
its message assertion.

**Why.** The decision `docs/decisions/1566-bore-rings.md` states the new law.
The motion and app suites of #1839 pin it. The UI tests were the only
callers that still expected the old one-ring limit. No hole that the tool
cannot cut is accepted: the ring-budget refusal still stores nothing.

**Considered and dropped.**
- Restore the one-ring refusal in the product. #1566 removed it by design,
  and the ring plan clears the hole safely.
- A ball nose at a blind floor (the zero-band refusal). The Bore tool door
  refuses a ball nose when the operation is created, so the UI cannot reach
  that case. The app unit test `detect_refuses_a_blind_ball_floor_but_keeps_the_through_hole`
  pins it.
- A catalog flat end mill with a through hole past the ring budget. The
  smallest one (Ø2) needs a hole wider than Ø6555 mm. That fixture is not
  practical.
- Update the `detect_holes` row of the camcraft-mcp skill in this PR. That
  file is a process file. It is tracked in #1910.

**Cost to change later.** Low. The fixture builder takes a floor height for
each cell, so a later law can add a cell type.
