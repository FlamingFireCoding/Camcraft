## #2473 - Tormach post: a move with no A word in an indexed operation

**Decided.** `assets/posts/tormach.cps` (`indexFor`) refuses by name a rapid
or linear record that has no A word while the operation holds an A that is
not 0 (modulo 360). The refused move is not written. A record with no A word
in an upright operation, or after the stream stated its turn to A0, posts as
before.

**Why.** The post read a record with no A word as tool angle 0. After PR
#2386 it then wrote a `G0` / `G1` with an `A0` word in the middle of the
operation, which swings the part under the tool. The stream does not mean
that: `post::dump` (`crates/post/src/post/mod.rs`, the `(None, None)` arm)
gives a record with no angle when the CL move states no axis, and its comment
says such a move "keeps the modal pose". A real turn back to upright gets an
explicit `A0` (the `(None, Some(_)) if tilted` arm). So a record with no A
word after a non-zero A is never a stated turn. Before #2386 the post refused
this record, so the refusal restores the old safe output.

**Dropped.** Keep the section angle (post the move in the held frame). That
matches the "keeps the modal pose" reading, but the `.cnc` parser reads a
record with no angle word as upright, so the two readings disagree. A
refusal is correct under the two readings; a kept angle is wrong under one.
No generator was found that emits such a record (the CL path attaches the
modal axis to each move), so the refusal costs no legal program that is
known.

**Cost to change.** Small: one branch in `indexFor`. If a producer is found
that emits such records on purpose, change the branch to keep `aSection` and
update `tormach_no_a_word.rs`.
