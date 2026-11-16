## #1599 - STEP import: a pass-wide evaluation budget for runaway faces

**What was decided.** All face meters and edge-curve meters of one shell's
tessellation pass share one `Pass` budget (`step_meter.rs`). Each meter
counts its first refusal in the pass one time: a cap run out, a division
that did not converge (`exhaust`), or a boundary-search fault. When
`PASS_TRIPS` (3) meters have tripped, the pass stops. After the stop, each
meter answers its fixed point, and each face in flight ends within one
sweep. The pass is then refused. The refusal names the first face (or edge)
that really tripped. It adds "the tessellation stopped after 3 faces or
edges ran out of evaluations; more of the model may be broken".

**Why.** Each runaway face spends a whole cap (150 M evaluations, 10-15 s)
before it trips. A file with N runaway faces cost N caps divided by the
core count. Now a pass spends at most 3 caps plus less than one cap for
each worker thread, for any face count. The pin measures this bound on a
shell with 31 runaway faces: 6.2 M evaluations before the change, at most
2.2 M after.

- The limit counts trips, not the summed evaluations of all faces. A
  healthy model's total evaluations in a pass were never measured, and a
  limit on that total could refuse a large healthy model. A trip happens
  only on a face that spent its whole cap, so healthy models are not
  affected.
- 3 trips keeps each pass with one or two trips exactly as before. #1568's
  case (a proven plane and a jump that trip in the same pass) is still
  judged face by face.
- A stopped pass is never "continuous". The faces that the stop cut short
  were never judged. One of them can be a jump, and a coarser mesh would
  smear that jump. Thus a refinement pass with 3 or more proven faces that
  trip is now refused, where before it kept the coarser mesh. The heaviest
  healthy face measured needs 28 M evaluations (cap 150 M), so 3 healthy
  trips in one pass is not expected.
- A meter that the stop cut short is not a trip. Its `exhaust` and its
  search faults are not recorded, so the refusal never names a healthy face
  as broken.
- Edge curves (#1545's `CurveGuard`) share the same pass, as the issue
  asks.

**Considered and dropped.**
- Stop the pass at the first trip that is not continuous. That refusal is
  certain, but the named face would then depend on thread timing for a
  shell with one or two runaways. The present refusals are deterministic,
  and the trip limit keeps them so below 3 trips.
- A limit on the summed evaluations of the pass: dropped for the reason
  above.
- A limit that scales with the face count: the time would then grow with
  the face count, which the issue forbids.

**Known limit.** Above the trip limit, the face that the refusal names can
change from run to run (the threads decide which faces trip first). The
model is refused on each run.

**Cost to change later.** `PASS_TRIPS` is one constant. The pins read it,
except the end-to-end pin's message text ("stopped after 3").
