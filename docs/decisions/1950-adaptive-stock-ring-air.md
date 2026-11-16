## #1950 - Adaptive 3D Pocket cuts the stock ring from the air

**What was decided.**

1. **The wide band.** A whole-model 3D Pocket with the Adaptive strategy
   (not shell, not rest roughing) grows its footprint `stock_band(r, true)`
   = `2r + OPEN_SLACK` past the stock outline, in place of `r - OPEN_SLACK`.
   Each level's stock ring is then planned from the air side inward
   (`adaptive::plan_from_air`), on the material clipped to the stock. The
   growth spiral planned that ring from one bore and ran past its budgets
   (45 retract hops, the level refused to Offset rings).
2. **The joins.** The air plan joins each pass to the one before it with
   the existing `LinkLaw` (straight chord, return over the retract plane,
   lead whose graze joins the pass). Each join is walked again on the
   stock, and its readings go into the worst load. A pass with no proved
   join keeps its retract hop. D plate reading: 45 hops -> 3 (budget 8).
3. **The air entry.** Where the first pass of an air plan starts with the
   tool's disc clear of the stock outline by 0.005 mm (`machining2d::in_air`;
   the stock outline is a prism, so the column is air at every height),
   the level is entered by an axial plunge there. No helix or ramp bores
   into the stock. Where it cannot be proved, the old entry law runs. A tool
   that is not centre-cutting keeps its old entry law too: the plunge law
   reads the cleared material of the path, not the stock outline, and
   would refuse a plunge in the air.
4. **Fixtures in the air band: a refusal by name, not a smaller band.** When
   a tagged fixture box meets the air band (the footprint less the stock
   outline) in XY and stands above the operation's bottom, generation
   refuses the operation by name. The message names the fixture and says
   what to do: move the fixture out of the band, or select the Offset
   rings strategy. The machine table is not judged by this law (it stands
   under the whole stock); the post gate judges it on every move. The post
   gate (`posting::verification_gate`, "strikes a fixture") is not changed
   and stays the second line, for steel moved after generation. So the air
   entry plunge in the band is clear of the tagged fixtures: a fixture in
   the band refuses the op before any move is posted.
5. **No pass crosses an earlier one (round E).** The round D reading through
   `adaptive_passes` was a self-crossing at (-8.12, -35.61). Cause, measured
   by the crossing pairs before and after the joins: the bare passes had 40
   crossing pairs, and the joins added none. The pairs were the spiral and
   the first open runs (a lead-in starts one load width back, on the last
   front's path, and steps past it), each open run and the run of the next
   front, and open runs and the wall laps (a run that meets the part wall
   cuts across the laps). Fix: `adaptive::air_uncross`. Each new pass drops
   the chords that cross a kept pass, by the plan's own law
   (`guard::crosses`), and goes on as a new piece. Each piece is read on the
   stock as before, so the rest and the load readings stay true. A late join
   (`LinkLaw::late`) that grows a pass by its grazing lead is kept only when
   the grown pass crosses no other pass. If not, the next law runs, or the
   retract hop. The crossing check, the load check and the budgets are not
   changed.
6. **Round E reading: over the hop budget.** Through `adaptive_passes`, the
   D plate level now has no crossing, but it has 23 retract hops (budget 8).
   So the level still goes to the Offset rings, by name. 22 of the 23 hops
   are late joins that were refused because their lead crosses an earlier
   pass. The round C reading of 3 hops (and the round D reading of 8) came
   from joins whose leads crossed. The budget was not raised.
7. **Round F reading: each open run needs its hop.** A reading probe
   (`air_rings::tests::probe_air_hops`) lists how each pass is reached.
   All 21 open runs of the D plate are reached by a retract hop. Cause: an
   open run starts at the part wall, beside the last front. The descent of
   a return must read no more than the bite (15.0 deg of rim). The rim is
   `r + 1.5` cells (`quality::rim_for`), so a disc that rides on a cut path
   reads its wall at about 25 deg. A landing that reads air must stand past
   the last front's path, and then the lead-in onto the run crosses that
   path. The last front's run ends at the centre boundary, so the lead-in
   cannot go round its end. Two changes were tried and reverted:
   - The lead-in gap capped at one load width less 0.02 mm. The runs are
     no longer split, but each start reads 25 deg of rim: 31 hops (was 23).
   - A lead square to the first chord on the cleared side (`find`, else
     `high`). It reads 25 to 84 deg on the descent and is refused.
   So with fronts cut as open runs, the hop count is at least the number of
   open runs. The budget of 8 needs a different cut of the open fronts (round
   G). The crossing law, the load law, the bite and the budget were not
   changed.
8. **Round G: the open fronts are cut region by region.** The runs of
   consecutive fronts in one open region form a chain
   (`adaptive::air_chain::chains`, matched by start and end within 3r).
   Each chain is cut whole before the next one. A run that goes on its
   chain is reached on the floor: back along the run just cut, to the
   vertex where the normal of the new run start meets that path
   (`air_chain::onto`), then the lead-in starts on that vertex. The
   lead-in only touches the path just cut, so it does not cross it. The
   move back is on the cut path, so it is on cleared area. Where no such
   vertex is found, a straight feed chord (`air_chain::link`) is tried,
   read on the stock within the load bound and the crossing law; else the
   joins run as before. Reading on the D plate level: 23 hops before,
   10 hops now (budget 8), worst load 0.1538 unchanged, 42 passes, 11784
   moves. The 3 region entries are hops, as planned. The other 7 hops are
   wall laps: a lap is parallel to the part wall, and each front run ends
   on the wall, so a lap piece meets a run end and is split there by the
   crossing law. A bridge chord across that run end also crosses it.
   Tried and reverted in round G: a feed chord between lap pieces (each
   bridge crosses the run end), and a closed lap started beside the last
   pass (its join still reads over the bite). The crossing law, the load
   law, the bite, the rim and the budget were not changed.
9. **Multi-Axis Rough keeps the old band** until the maintainer decides. It
   shares the open-side footprint, but its levels were not read with the
   air planner.

**Why.** The band and the joins remove the planner budget refusal of the
user's D plate without a new budget. The refusal of item 4 is the smallest
safe answer: the generator never routes the band round steel, and a band
cut short at a clamp would leave a ring of stock that the plan did not read.
Generation now says why and what to do, before the post gate has to.

**Considered and dropped.**

- Shrinking the band round a fixture (a keep-out region in the footprint):
  dropped for this PR by the maintainer's decision. It needs a band reading
  where the band is narrower than `2r`, and a proof that the ring left there
  is still in the load bound.
- A helix bore in the air: no material to bore, so a plunge is enough.
- Air entry for a tool that is not centre-cutting: needs the plunge law to
  read the stock outline. Left for later.
- Round E: a looser crossing test, or a higher hop budget. Both dropped by
  the brief: the crossing law is the verifier's law (a), and the budget
  says when the plan is slower than the rings it replaces.
- Round E: capping the lead-in gap under one load width, so a lead stays
  between its run and the last front. Not tried in this round. It is the
  first item for round F, with a join law for the open runs that does not
  run back across the last front.
- Round F: a higher bite for the descent, a smaller rim for the landing, or a
  higher hop budget. All dropped: each loosens a check. The two tried
  changes of item 7 are dropped because they do not lower the hops.

**Open point for the maintainer: the load margin.** The worst load of the D
plate air plan is 0.153760 of D at (29.77, 22.31). That is at the bound,
with a slack of 0.00135 deg of rim: less than one rim sample. No margin was
added. The plan is in the bound by the reading, but a change in the
sampling could move it over, and then the level is refused by name to
Offset rings (never posted over the bound). Decide if the air plan needs a
margin (for example one rim sample, 1 deg) under the optimal load.

**Cost to change later.** The band width is one function
(`machining3d::stock_band`). The refusal is one function
(`operation::open_side::air_band_refusal`); a keep-out band would replace it.
Each change moves the posted G-code of whole-model Adaptive 3D Pockets.
