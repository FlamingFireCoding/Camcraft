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
5. **Multi-Axis Rough keeps the old band** until the maintainer decides. It
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
