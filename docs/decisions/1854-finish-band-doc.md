## #1854 - Describe finish-band coverage accurately

**What was decided.** Correct the module documentation in
`crates/app/tests/shared_face_bodies_finish.rs` to describe each band
starting its own level ladder. Levels can shift by less than one stepdown
at a band boundary compared with the whole-span program. Links between
bands are not generated or verified.

**Why.** The previous claim that the bands run every level of the
whole-span program contradicted the decision recorded for #1715 in
PR #1846. The revised comment states that recorded limitation without
changing test or machining behavior. A temporary text assertion verifies
the documentation correction; restoring the upstream file makes it fail.
No permanent test code is needed for this prose correction.

**Dropped.** Changing band generation to reproduce the whole-span level
ladder would change behavior and exceeds this documentation-only issue.
Running the heavy finish lanes would exercise machining behavior rather
than establish whether the comment states the recorded limitation; the
quick gate and crate clippy check provide the requested validation.

**Cost to change later.** Documentation only. If band generation changes,
update this explanation alongside the tests that establish its coverage.
