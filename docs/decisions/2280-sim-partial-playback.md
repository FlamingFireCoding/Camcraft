## #2280 - The stock playback stops before the first operation with no toolpath

**Decided.** The stock playback (the transport's traced stock) plays the
program in posting order up to the first operation that has no toolpath.
It stops there. It does not skip that operation to cut the ones after it.
A separate line under the transport bar names the operation: "Stock
simulation stops before 2D Contour1: it has no toolpath. Generate it to
see the rest." The stop is not added to the status line, so the notes of
a clean operation do not get the error mark. The remaining playback
refusal reads "stock simulation refused: ... - the stock shows uncut" in
place of "traced playback refused: ... - the sheet stands".

**Why.** The issue's expected behaviour. A stock after a skipped
operation is wrong, so a skip is a silent wrong answer; a stop is not.
The post and the Deviate verdict keep `posting::program_ops` and
`stock_sim::plan_program` unchanged: they still refuse the program by
name. Only the playback uses the new `program_ops_until_ungenerated` /
`plan_program_until_ungenerated` walk.

**Details.**
- Only an operation with no toolpath (`OpStatus::Never`) stops the
  playback. A stale operation, a tool problem or a spindle problem before
  the stop still refuses the playback, as before: those operations have a
  toolpath that the playback would cut wrong.
- When the first operation has no toolpath, the playback shows the uncut
  stock of that operation's setup and the stop line.
- On the timeline, an operation at or after the stop holds the stock as
  the plan left it (`PlayIndex::position_at`). An operation that is not
  in the plan and is not after the stop is still a refusal.

**Dropped.** Skip the operation and continue (the issue forbids it: the
stock after the skip is wrong). Keep the stop as a clause on the status
line (fault 2 of the issue: it reads as an error at the end of other
notes).

**Cost to change.** Small: the stop list lives in `PlayIndex::rest`; the
line is drawn by `play_precalc::transport_row`.
