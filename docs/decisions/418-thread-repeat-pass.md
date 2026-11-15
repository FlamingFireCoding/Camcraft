## #418 - threading: repeat each roughing depth ("spring at each depth")

- **Chosen:** a bool on the infeed plan, `ThreadInfeed.repeat_each_depth`
  (default off). When on, `ThreadInfeed::passes` follows every pass that
  stands above the root with an identical copy of itself, so each
  roughing depth is cut twice before the next one. Dropped: a count
  (2, 3, ... cuts per depth). The issue asks for "twice", and nothing in
  the research asks for more. If a count is ever needed, the bool
  becomes a `u32` with a DTO default of 1, and the absent key still
  reads as off.
- **The repeat is the same pass, so it cannot gouge.** It is pushed as
  the same `Pass { tip, shift }`, and the kernel lays it from the same
  lattice start (`c_first + shift`) with the same lead, so the repeat
  has the same tip, the same station and the same helix phase. Its
  moves are identical to the first cut's
  (`a_repeat_is_the_same_pass_on_every_method_and_the_judge_stays_silent`
  compares the move spans for every method and schedule). The verdict
  judges the laid passes, the repeats included, and stays silent.
- **Alternating infeed counts depths, not cuts.** The side flips once per
  depth, so a repeated depth keeps its side. If the flip counted cuts,
  the "repeat" would cut the other flank, which would make it a
  different pass.
- **What gets repeated:** every pass whose tip is above the root
  (`tip > root + HAIR`). The finishing pass and the spring passes at
  the root are not repeated: those passes already exist as spring
  passes and have their own count. With no finish allowance, the last
  rung is the root, so it is the finishing pass and is cut once
  (0.4, 0.4, 0.8, 0.8, 1.2).
- **G76 refuses by name.** A one-block cycle cuts each depth once, so
  the record carries a `repeat_each_depth` flag. This follows #417's
  `alternating` precedent: the stream writes `REP 1` only when the flag
  is set, the post property is `RepeatEachDepth`, and the project cache
  key is written only when set. `fanuc-lathe.cps` refuses
  "repeat each roughing depth is not a G76 cycle". The kernel does not
  refuse, because the G32 / G76 choice is a document post option, not
  an op parameter. The refusal therefore happens where that choice is
  known, the same way alternating infeed is refused.
- **The Tormach PathPilot lathe G76 refuses it too (takeover,
  2026-11-15).** Main's #612 added `tormach-lathe.cps` with a one-block
  LinuxCNC G76 after this PR was written. Its `g76Why` did not read
  `RepeatEachDepth`, so a repeat plan with G76 chosen posted a cycle
  that cuts each depth once: not the passes that were verified.
  - Chosen: refuse by name ("repeat each roughing depth - LinuxCNC's
    G76 cuts each depth once"), the same rule as Fanuc and as
    alternating infeed. Pinned by a new case in
    `lathe_post_tormach_thread::a_g76_record_the_cycle_cannot_state_exactly_is_refused_by_name`.
  - Dropped: post the G76 and ignore the repeat. The program then
    differs from the verified passes (#417's law), so it is not an
    option. G33 passes still post the repeat.
- **Defaults are byte-identical.** With the default off:
  - the kernel lays the same passes;
  - the stamp writes no extra byte (one byte, `0xD5`, only when the
    flag is set, inside the non-default branch);
  - the project's `infeed_plan` key and the record key are absent;
  - the stream writes no `REP` word.

  No golden changed. A #416 file with a non-default plan and no
  `repeat_each_depth` key loads with the repeat off (serde default).
- **UI:** one checkbox row, "Repeat each depth", on the Passes tab
  after "Spring passes". Its tooltip says G32 only, says G76 refuses
  it, and notes that tooling makers recommend against routine spring
  passes.
- **Cost to change later:** one field in the plan and in its DTO, one
  flag on the record. A dialect with a per-depth repeat would read the
  same flag. No public cycle we read has one.
