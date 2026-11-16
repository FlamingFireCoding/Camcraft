## #2121 - a rapid at the clearance plane is judged against the stock; a plane inside raw stock is named when the operation is made

**What was decided**

1. The mill column law (d) lost its clearance gate. Every lateral rapid
   under the stock top is marched against the column, whatever plane it
   rides. The stock top alone stands the law down.
2. The convictions this adds (a lateral rapid at or above the clearance
   plane) get two re-reads in the operation layer. Each only removes a
   conviction, and neither touches a rapid under the plane:
   - `verify::acquit_traverses_by_solids`: the tool stack against the
     stock and part solids under the strip the rapid sweeps. Needed
     because the column is capped at the stock's highest point, and in a
     3+2 plane's frame at the tilted block's highest corner.
   - `verify::acquit_traverses_by_prior`: the stock grid the program
     ahead left, under the swept capsule, plus the part's own facets.
3. The warning stays `RapidInStock`. Its sentence is now "rapid move N
   travels through stock (below the clearance plane, or at it below the
   stock top)". A path note "Clearance in stock: rapid move N travels
   through stock at Z ..." names the cause with the plane and the top.
4. Command creation does not refuse the plane. A word
   (`operation::clearance_in_stock_advice`) names it where the reading
   is plain: an upright mill or vertical hole kind, a stock with one
   flat top, no unsuppressed operation ahead. The MCP `add_operation`
   and `edit_operation` tools answer with it as `warning`.

**Why**

- The gate assumed the clearance plane is air. A Fixed or model-based
  plane under the stock top is ordinary input, and the program posted.
- Without the prior re-read, a legal program would be refused: Face to
  Z 5, then a Contour with clearance Z 8 under the raw top Z 10.
- Without the solids re-read, a legal 3+2 program was refused: the
  full run showed `stock_source_holes.rs`'s tilted stock chamfer
  convicted on move 0, a rapid that is 4 mm above the block in world
  space. The tilted block's far corner stood above the plane.
- The solids re-read clips each facet to the strip instead of taking
  the facet's highest vertex (#1095's bound), because a tilted block's
  top is two large triangles whose highest vertex is that far corner.

**Considered and dropped**

- Refusing the plane at command creation. Built first, then dropped.
  The door cannot see where the rapids will run: a profile beside the
  block never crosses it, so it would refuse legal programs. It also
  refused four existing fixtures at creation (`operations.rs`,
  `opframe_picks.rs`, `posted_gates`, `turning.rs`), which shows how
  much standing input sets a plane under a raw block. The law holds
  the post for the ones that are unsafe; #104 chose a note the same way.
- A new warning variant carrying Z. Dropped: about forty sites match
  `RapidInStock` (acquittals, warned moves, the UI fold, MCP); a
  variant one of them missed would be drawn or acquitted differently.
- Tightening `acquit_by_stock_top` itself to the clipped bound. Dropped
  here: it would change which #1091/#1095 convictions stand. The new
  re-read is confined to the convictions #2121 added.
- Naming a Retract plane under the stock top. Dropped: it is legal
  inside the operation's own kerf, and the law already reads every
  link and dive at it (#96, #140).
- Turning, rotary, radial and multi-axis kinds. Their rapid laws
  (meridian, rotary, radial, posed) never had a clearance gate.

**Cost to change later**

- Making the word a refusal is one call in the heights door
  (`commands/operation/gates.rs`); the fixtures above would need their
  planes raised first.
- The UI does not show the word yet (follow-up issue named in the PR).
- A path cached before this fix keeps its cached warnings until it is
  regenerated; the op stamp salt was not bumped.
