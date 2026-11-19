## #1591 - 3D Adaptive: Method choice with a Trochoidal method per level

**What was decided**

- `OpParams::Pocket3D` has a `method` field (`AdaptiveMethod`: Spiral or
  Trochoidal). It is read only under the Adaptive strategy. The default
  is Spiral.
- Under Trochoidal, each level of the plain ladder is planned by the 2D
  lane planner (`plan_all_lanes`, `plan_cleanup`) on the cutter-centre
  area that the ladder already has. The new module
  `machining2d::adaptive_troch_level` plans, posts and checks one level.
  `machining3d::troch` holds the run state and the new entry point
  `pocket3d_trochoidal`.
- A level that the method refuses is cut by the Spiral plan. A note says
  why and names the Z of the level. The level is built on a copy of the
  path, so a refusal posts nothing of the trochoidal attempt.
- The material top is proved, not assumed. A trochoidal level can leave
  stock, so a later bore or plunge starts above the deepest trochoidal
  level whose cleared area holds its disc. If no level holds it, the
  bore starts above the stock top. A proved bore starts on the proved
  plane, as the ladder's own descents do. Thus the helices of two levels
  share no depth and cannot cross.
- In a trochoidal run, each level enters as a first level does. This
  includes a fallback (Spiral) level below a fallback level. No level
  reads the plane above as cleared. The cost is a longer descent in air
  on a run where many levels fall back.
- Each posted level is checked against its level area and against the
  crossing law. The crossing check reads the level's own moves and all
  moves before it.
- The rest pass, the offset shells and the rotary wrap keep the plan
  that they had. Their existing notes say so.
- The method is passed as an argument of the internal `ladder`, not as a
  field of `Pocket3DParams`.
- The project file writes `method` as a word for each 3D Pocket, as 2D
  Adaptive does. A file without the word reads as `spiral`. No format
  version was added.
- The stamp gets one more byte only under Adaptive with Trochoidal.
- Under Trochoidal the footprint does not use the Adaptive air band
  (#1950). The loops are not planned from the air.

**Why**

- The level area of the ladder is the level law: footprint less the
  part above the plane, eroded by the tool radius and the stock to
  leave. Loops inside it are inside the level area and beside the
  model. The posted level is checked again, move by move.
- The 3D descents (`descent.rs`) read the plane above as the material
  top. That is true after a Spiral or ring level. It is not true after
  a trochoidal level that left stock. A rapid to that plane could hit
  stock. Thus a trochoidal run proves the top or uses the stock top.
- After a trochoidal level the tool is at the end of that level, not on
  the plane of an older fallback level. A keep-down level link would
  feed from there through stock that the loops left. The "Do not touch"
  keep-out of #2149 also reads the plane above as the top of the band
  that a descent crosses. With no plane above, the keep-out covers the
  full height, which a bore from the stock top needs.
- A level area of a 3D part is often narrow near a wall. A refusal of
  the whole operation for one such level makes the method hard to use.
  The issue permits a fallback by name.
- A `method` field in `Pocket3DParams` changes 74 struct literals in 28
  test files. An argument of `ladder` changes 7 internal call sites.
- A Spiral operation has the same stamp as before. Existing 3D
  Adaptive operations do not go Stale.
- The MCP params patch refuses a key that does not come back from the
  typed round trip. An unwritten default made `"method": "spiral"` an
  unknown field. Thus the word is always written.
- 2D Adaptive added its `method` word without a format version (#1509).
  This change follows that precedent.

**What was considered and dropped**

- Refuse the operation when one level cannot be cut by loops. Dropped:
  see above.
- Offset rings as the fallback of a refused level. Dropped: the Spiral
  plan is the other value of the same choice, and it already falls back
  to rings by name.
- Record a fallback level as fully cleared, so that the next bore starts
  lower. Dropped: the HSM plan can leave stock inside its own budget.
  The cost is a longer bore helix in air below a fallback level.
- The four trochoidal controls of 2D Adaptive (#1588: engagement limit,
  return, return feed, loop width) on the 3D page. Dropped from this
  change: the issue asks for the Method choice only. The 3D method uses
  the defaults (90 deg, automatic return).
- A trochoidal rest pass or trochoidal offset shells. Dropped: the rest
  targets are sim crumbs and the shells lift vertices off the plane.
  The lane planner proves its loops on a plane.
- Write `method` only when it is `trochoidal`, to keep the bytes of a
  Spiral operation. Dropped: MCP could then not set the method back to
  Spiral (see above).
- A project format version for the new word. Dropped: an older build
  ignores the unknown word and cuts the Spiral plan, which is a legal
  program for the same operation.

**What it costs to change later**

- A `method` field in `Pocket3DParams` instead of the argument: a
  mechanical edit of the struct literals.
- Trochoidal controls on the 3D page: `troch::Run::level` passes
  `AdaptiveTrochParams::default()` at one place.
- A format version for the word: one constant and one migration note.
