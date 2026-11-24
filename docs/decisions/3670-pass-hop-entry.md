## #3670 - A 3D Pocket pass restart in a level takes the entry type of the operation

**Decided.** A 3D Pocket pass that starts again inside a level after a hop
(the zigzag row on the far side of an island) is judged by the
centre-cutting plunge law (`plunge_law::into_solid`) for every tool, not
only for a tool that does not cut at its centre. Where the straight plunge
would go into uncut stock, a centre-cutting tool now takes, in this order:

1. The entry type of the operation (helix, ramp, arc), through the same
   `descend` that the level entry uses.
2. The ramp (zig-zag) along the pass.
3. The straight plunge, as before, with a note that names it.

Steps 1 and 2 run on a copy of the path. A copy is kept only if it is
clean: it posts no plunge into uncut stock, and it crosses no move that is
already posted (the verifier's Crossing law; on a lifted offset shell a
descent can cross the shell's legs).

A tool that does not cut at its centre keeps its M11.3 fallback. An
operation whose entry is Plunge keeps the straight plunge. A plunge over a
column that is already cleared stays a plunge.

**Why.** The user sets a helix entry to avoid a full-width plunge into
stock. The old code honoured it only on the first descent of each level.
The trial copy is necessary: where the helix does not fit at the pass start
(a 2.2 mm stub between the dome and the stock edge in the catalog case),
`descend` puts the helix far away at the widest room and then dives again
at the pass start. That gives a helix and also the plunge, which is worse
than the plunge alone.

**Dropped.**
- Use the level transition (`links.arm(entry)`) for the restart. A restart
  is a new entry into stock at the same plane, not a move between levels,
  so the entry type is the correct choice.
- Change the pass order so that each row starts where a helix fits. That
  changes the zigzag walk and every 3D Pocket program.
- Refuse the operation when no helix or ramp fits for a centre-cutting
  tool. A centre-cutting tool can plunge. The note tells the user.
- The two "Also" items of the issue (a plunge over a cleared column still
  feeds from the hop plane; 16 moves shorter than the post resolution post
  as zero-length lines). They are a different site and go to the
  follow-up issue #3852.

**Cost to change.** Small. The rule is in one function,
`machining3d::reentry::reenter`, and one call site in `ladder.rs`. Each 3D
Pocket program with a helix, ramp or arc entry and pass hops changes when
this rule changes.
