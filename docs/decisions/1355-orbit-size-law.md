## #1355 - Count the sinker orbit in rough and finish sizing

**Decided:** Keep the condition's electrode law, `e = c - 2u`.
Count the XY orbit radius in the burned width: `e + 2(g + r)`.
A rough must stay inside the finished cavity. A finish must reach its size.
Validate the orbit before the size judgment.
Set the tip's floor stand to `u - r`, because the orbit moves only in XY.
This leaves `u - r - g` on a rough floor and brings a sized finish to the floor.

**Why:** Radius widens the burn but does not deepen it.
Keeping the old stand, `u`, leaves one radius of stock on the floor.
Adding radius to the simulated spark gap would instead deepen the burn incorrectly.
Remove the interim refusal of every rough or finish orbit.
Keep a named refusal for a finish vector pattern whose direction count is not a multiple of four.
Those patterns do not reach all four walls, so radius alone cannot certify the finished cavity.

**Considered and dropped:** Keeping the blanket orbit refusal prevents valid passes.
Unchecked acceptance can burn past a wall or leave a finish short.
Keeping the old floor stand leaves stock. Inflating the floor's spark gap misstates the physical burn.
A generalized law with unequal per-axis offsets would support more vector patterns, but needs separate sizing and floor rules.

**Evidence and limits:** Synthetic circular, four-direction vector, and planetary passes reach a clean stock-simulation verdict with zero gouge.
The pins check the rough and finish tip heights, wall overburn refusal, partial-vector refusal, and malformed orbit refusal.
The old implementation and four individual mutations each fail their pins; restoration passes.
No real EDM control was exercised. No golden files changed.
The helper lives under `tests/sinker_undersize/` so the quick gate selects its owning suite instead of every app suite.

**Cost to change later:** A per-axis sweep law can replace the conservative vector restriction.
It must calculate each wall's actual reach, preserve a safe floor stand, and add simulation pins for unequal offsets.
The current electrode law and non-orbit behavior remain the baseline for that work.
