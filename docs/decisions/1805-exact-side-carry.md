## #1805 - Retain cut geometry when stock turns side-on

**What was decided.** Retain the original box and the swept tool solids
alongside the column grid. A side-on carry transforms this geometry through
the same signed permutation as the stock. It then reads the occupied
intervals along the new columns. The old column centres no longer determine
the position of a machined wall.

The retained geometry keeps separate material intervals. A turn does not
replace a pocket with the hull of its walls. The top and floor remain the
outer bounds for callers that need a column body. A caller that needs the
material inside those bounds reads the occupied intervals.

The interval read selects one of three methods:

- A ray parallel to the original tool axis uses the original dexel atom.
  This preserves its floor and its treatment of contact.
- A straight flat-tool sweep uses a closed-form capsule intersection. A
  rising or falling move first restricts the swept segment to the height
  that the ray reaches.
- Other rays use the swept-tool distance laws. The Lipschitz bound, or two
  proven-inside endpoints of a convex sweep, must prove an interval cut
  before the read removes it. Subdivision stops at 0.0001 mm. An unresolved
  surface interval remains material.

The cone uses one implicit solid for its tapered nose and cylindrical
shank. It has no internal cap at the transition. Two separate solids can
retain a false skin at their shared cap under conservative subtraction.

A ray has a budget of 100,000 work units. A full read also limits the
estimated column-and-history work to 10,000,000 units. A read that exceeds
either budget, or produces a non-finite value, refuses by name. These
choices favour excess reported stock over missing real stock.

The retained geometry includes later cuts and stock clips. Each cut or clip
keeps its own frame. Further quarter turns compose those frames instead of
sampling an earlier read-back grid.

**Gouges and fixtures.** Keep a snapshot of the material that enters the
side-on setup. An empty entry ray must not charge the full turned stock
height as the depth of a gouge made earlier. The verdict before the turn
already records that gouge. A later cut that removes material from the
entry ray still produces a cut-through verdict.

The fixture checks use the occupied intervals both on columns and between
columns. The latter checks intersect the intervals shared by four adjacent
columns. A fixture in a retained cavity therefore does not meet the hull
of its walls. An unread interval query keeps the conservative column hull.

**Why.** The column grid holds top heights at sample points. Those heights
cannot recover the position of a tool flank between the points. Moving a
turned face to a cell edge can omit real stock. Keeping the cut that made
the face supplies the missing information without that estimate.

**Field engine.** Keep `FIELD_SIDE_CARRY_REFUSAL`. The field stores scalar
cut samples and vertical `cut_ups` and `cut_downs` records. Re-indexing the
bricks alone does not give these records the meaning needed for a ray along
a different axis. Its current `column_span` law also assumes one material
interval. Removing the refusal without replacing those laws could lose
stock or fill a cavity. Issue #1805 permits the refusal when the reason is
recorded. This change uses the column engine's retained geometry instead.

**Limits.** The retained source is box stock. A raw grid, a mesh or cylinder
stock, or an edit that discards the cut history still uses the conservative
carry introduced by #874. That path can over-read a turned face by one cell
and fill a cavity. It must not claim the accuracy of the retained-geometry
path. The read tolerance bounds the interval subdivision; it does not make
the column sampling an exact representation of every surface between rays.

**Considered and dropped.**

- Move the old column face to a cell edge. This can leave the carried stock
  smaller than the real stock.
- Reduce the cell pitch. This adds work but does not recover a wall position
  or a cavity that the column representation discarded.
- Keep only the new top and floor. This fills a cavity between them and
  changes the result of a later turn.
- Use subdivision for straight flat sweeps. The capsule intersection is
  both cheaper and more precise at the flank.
- Split the cone and shank at a shared cap. Conservative subtraction can
  preserve that internal cap as false material.
- Charge every empty turned ray through the full stock height. This can
  increase the reported depth of an earlier gouge without a new cut.
- Remove the field refusal after a brick permutation. The field's vertical
  cut records still cannot establish material intervals on the new axis.

**Code placement.** Move the existing ball atom into `dexel_ball.rs` and
the existing body verdict code into `carry_verdict.rs`. These are structural
splits to keep the touched files below 400 lines. They do not change those
algorithms.

**Cost to change later.** Other stock shapes need a retained source that
can answer conservative ray intersections. Other edit paths must retain
their geometry before they can use this carry. A field implementation needs
axis-independent cut records and a read that returns separate intervals.
The ray budget and subdivision tolerance can change after measurement, but
the proof required before material is removed must remain.
