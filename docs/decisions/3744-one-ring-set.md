## #3744 - 2D Offset "Pass starts at": one ring set round a part

**Decided.** An area round a part (`StartSide::around_part`, the 2D Offset
Outside band) has ONE ring set for Auto, Inside and Outside: the level
curves of the distance from the part (the area's holes) at
`off + k * stepover`, clipped to the cutter-centre area. The pieces that the
stock edge cuts are open pieces of the same set.

- Inside walks the set from the part out (the old Inside order, byte for
  byte).
- Outside walks the same pieces mirrored (`ring_side::mirror`): each piece
  runs the other way (the cut and uncut sides swap, so climb stays climb),
  the outer edge of the area is ring 0, ring `j` of `top` is ring
  `top + 1 - j`, and the part wall is cut last.
- Auto picks one side in one place (`ring_side::resolve`). Auto is Outside
  when the stock footprint is known and no outer edge of the cutter-centre
  area has its disc wholly in the stock (the same law as the "wall first"
  warning: an Outside start there owes no full-width cut). Else Auto is
  Inside. The resolved side then drives the rings, the links
  (`cleared_links`) and the entry, so Auto is exactly the path of Inside or
  of Outside. The 2D Offset op now sends the stock footprint for Auto round
  a part too (`generate.rs`), so that the rule can read it.

**Why.** The maintainer's rule (#3741): "Pass starts at" changes only where
the tool starts and the ring order. Before, the three values called three
planners: Outside offset the stock box (the part cut each ring into pieces
that ended on the part wall, 14 of 14 runs on the dog bone), Inside offset
the part, and Auto offset both edges (`waterline_passes`).

**Scope kept small.**
- Only `around_part` areas change. A pocket (no part to follow), the 3-D
  levels (`level_ring_plan`, `around_part: false`) and the Adaptive plan keep
  their old plans byte for byte. #3743 reviews them.
- A part with no hole (stock nested in a ring part) keeps its old rings:
  core first for Inside, from its own edge for Outside. Those are offsets of
  the same edge already.
- The completeness guard stays: a plan that leaves more than 0.1 mm2 of the
  cutter-centre area unswept still takes the old Auto plan with the named
  `StrategyFallback` note. Dropped for now: clean-up rings of the same set
  for a leftover. The guard is the Prime Directive's net, and the dog-bone
  pins assert that it does not fire. #3743 owns the replacement.
- The #3567 dead-end ramp marks go away round a part (issue step 6). The
  Outside pieces of the one set end on no part wall, so Outside enters as
  Inside does. On the fig8 Auto pass (`flute_reach`), two dead-end ramps
  crossed in XY at the same depth, and the crossing law blocked the post.
  The marks stay for a pocket (`around_part: false`); #3748 retires them.
  The entry (#3746) and the links (#3747) are not changed.
- The ring ends are snapped onto the boundary walk within 0.1 um
  (`ring_side::share_ends`). The closing chord of a closed walk is
  included. Before, the test was 1 nm, and a ring end that the clip put
  3 nm off its chord was not shared. On the block of `shoulder_adaptive`,
  the verifier then read a crossing at that end. Inside had the same gap.
  The ends of an open walk never move, because ring 0 and a wall share
  them.
- When Auto resolves, the "Pass start" notes name the side that was cut
  ("Inside" or "Outside"), not "Auto". Dropped: an extra note for Auto,
  because each note shows as a warning line on the op page.

**Cost to change later.** The rule lives in `ring_side::resolve`; a
different Auto rule changes one function and its pin
(`auto_picks_one_side_of_the_one_ring_set`).

## #3788 (c) - the planner names each full-width cut by the flanks law

**Decided.** `level_order::order` judges a full-width cut by the law of the
full-width meter of the op tests (`level_flanks`): both flank points, 0.9
tool radii either side of the walk, stand in material (the stock footprint
inside the area that the disc may sweep) that no earlier cut and no part of
the walk's own disc more than 2.5 radii back reached. A piece with such a
sample counts in the narrow note, which also names a merge of two rings now.

**Why.** The review of #3790 found the 495 waist cuts of fig8 Outside
silent: the old law (no cut within 1.9 radii of the centre) passed them.
The coordinator's decision (maintainer away): every full-width cut is named
by the meter's own law. The pins are `fig8_outside_offset_cuts_no_move_at_full_width`
(the note names the meter's first place) and
`fig8_inside_offset_names_each_full_width_cut`.

**Dropped.** A note for each place: the note names the count and the first
place, as before.
