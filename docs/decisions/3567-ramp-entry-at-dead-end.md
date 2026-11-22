## #3567 - A later Outside pass enters by a ramp along the pass at a dead end

**Decided.** The maintainer set the entry in the question dialog on
2026-11-22 (the comments on #3567). At a dead end of a later 2D Offset pass
with "Pass starts at: Outside", the tool enters by a ramp along the pass.
The ramp lowers the tool while it travels along the pass. The cut engages in
a gradual way. The late-entry pin counts only plunges and helices into
stock. A ramp along the pass is not one of them. Climb milling stays. The
full-width rule stays. The "Auto" workaround stays until this fix merges.

The worker made these choices unattended:

1. **The dead end is read from the cleared column.** `pass_link` already
   asks `plunge_law::into_solid` at a later pass start. A start that a lead
   chord reached from cleared stock has a cleared column, so the test is
   false. A dead end has no such lead, so the test is true. No new per-pass
   flag is needed from `level_order`.
2. **The ramp goes before the helix and the plunge.** At such a start,
   `ramp_fallback` plans the existing zig-zag ramp along the pass at the
   entry ramp angle (3 degrees by default). The helix and the plunge remain
   the entry at every other start.
3. **The scope is Outside with Waterline or Offset rings.** The flag is set
   for `PassStart::Outside` and those two strategies, on the main pocket
   path and on the "Do not touch" path. Auto, Inside, Adaptive, Zigzag and
   rest machining keep their old entry law.
4. **A ramp that does not fit keeps the old law.** The pass can be too
   short for the ramp angle and the leg limit. Then the existing helix,
   plunge or named refusal stands. No refusal is weakened.
5. **The pin is narrowed, not removed.** `late_entry_cut` counts only a
   descending `MoveKind::Plunge` or a descending arc (a helix). A straight
   descending feed is the ramp and is not counted.

**Why.** Main posts a helix or a plunge at each later dead-end start of the
split band. The tool disc meets the crescent of uncut stock at the full tool
width. That is a tool-load fault: the cutter can break. The pin read
332.88 mm2 of stock cut by those later entries on main. With the ramp it
reads 0.00 mm2.

**Dropped.**
- A planar lead chord across one stepover (the held option A on
  `issue/3567-outside-later-entry`, commit `12c0fd125`): it feeds into the
  crescent at the full tool width and breaks the shape-mask law.
- The named fallback to the Auto order: it changes the band order and
  breaks the same shape-mask law.
- A per-pass flag from `level_order`: `into_solid` already names the dead
  end, so the flag adds plumbing with no gain.

**Cost to change later.** Low. The rule is one branch in
`machining2d/ladder_links.rs::pass_link` and one `Motion` flag set by the
callers. To make the ramp the entry for Inside too, set the flag for Inside
and run the Inside pins.
