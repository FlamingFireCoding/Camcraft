## #4085 - DIN/ISO Heidenhain post on a manual tool change machine; the manual change law

**What was decided.**

1. The DIN/ISO post (`assets/posts/heidenhain.cps`) writes the manual
   change sequence when the machine states `tooling.tool_change =
   "manual-retract-top"` (today only the Fehlmann Picomax 55, iTNC 530):

   ```
   N.. M5*                       (end of the previous section, as before)
   N.. G00 G90 Z+0 M92*          (retract, under the OLD tool call)
   N.. ; CHECK DRIVES OFF BEFORE TOUCHING TOOL*
   N.. ; CHECK QUILL AT PHYSICAL CHANGE POSITION*
   N.. ; ABORT IF EITHER CHECK FAILS*
   N.. ; THEN INSTALL TOOL 3 FLAT-6*
   N.. M6*                       (the stop: MP 7440 bit 0 = 0 halts at M06)
   N.. ; T3 FLAT-6 - ...*
   N.. T3 G17 S3500*             (the new length becomes active here)
   ```

   At the first tool the tool in the spindle is unknown: the checks and
   `M6` only, no retract (the same as the Klartext post's first `M0`). A
   repeated tool number is no change and writes no stop.
2. The engine refuses a script on a machine with a manual tool change
   (`manual` or `manual-retract-top`) unless the script declares that mode
   in its manifest: `manualToolChange = ["manual-retract-top"]`
   (`crates/post/src/post/manual_change.rs`). The refusal names the post
   and the machine, before any hook runs. The Klartext post and the
   DIN/ISO post declare `manual-retract-top`. No shipped post declares
   plain `manual`.

**Why this order (which tool length is active).** The Picomax manual
writes TOOL CALL, then `L Z+0 R0 FMAX M92`, then the M6 park move. MP
7480.0 = 1: TOOL CALL only gives the T number to the PLC and does not
stop. We put the T block after the stop instead. Then the retract runs
under the old tool call, so the length in force is the length of the tool
that is physically in the spindle. The retract target is machine-referenced
(M92 Z+0 = MP 950.2 = MP 4210.0, the PLC tool change Z, -0.05 from the
machine datum, the top of traverse range 1). If the control ignores the
tool length for M92 moves, the quill goes to the change height. If it
applied a length, the target is above the range-1 limit and the control
stops with a limit error. In both cases the move goes only up. The first
move with the new tool comes after `M6` and after the new T block, so
the new length is active only when the operator has installed that tool.
The section end already wrote `M5` and `M9` before the retract.

**Why `manual-retract-top` only.** That mode states that the change is at
the physical upper change position, which M92 Z+0 reaches on the Picomax
(machine parameters). Plain `manual` states no change position, so the
DIN/ISO post does not claim it, and the engine law refuses the pair by
name. The operator checks and the M6 stop catch a machine where the M92
datum is not the change height.

**Considered and dropped.**
- Refuse the DIN/ISO post on every manual change machine: safe, but it
  drops the DIN/ISO drilling cycles for the Picomax, which the Klartext
  post does not have. The sequence above is safe, so the refusal is kept
  only for plain `manual`.
- TOOL CALL before the retract (the manual's own order): the new length
  is active while the old tool moves. The move is upward in both length
  interpretations, but the order above needs no such argument.
- `M140 MB MAX` (the Klartext post's retract): its DIN/ISO block form is
  not confirmed from a source for NC 340490-03. `G00 G90 Z+0 M92` is the
  form that the issue and the review name.
- An XY park move (the manual's `X-50 Y-50`): the park position is
  job-specific and the machine file does not state one.
- A law keyed on the `toolChangeMode` text in the script source: a
  script can read the field and still write no stop. A declaration is
  explicit, and reviewers can check it.

**Cost to change later.** A new post that handles a manual change adds
one manifest line and its own sequence. A machine-file change position
(issue suggestion "M92 change position") can replace `Z+0 M92` with the
stated value in one function (`manualChange`). Not verified on the real
control (NC 340490-03): the DIN/ISO post is verified only on the
606425-04 programming station (#3095).
