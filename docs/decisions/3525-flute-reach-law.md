## #3525 - A cut deeper than the flute length is named by its cause

Taken unattended, as the recommended option under the owner's standing
word (Autonomous Work Law).

**What was decided.**

- A new law, `camcraft_sim::verify::flute_reach`, reads the lowest cutting
  tip of a path against the stock top. When that depth is larger than the
  tool's flute length, it gives one new warning, `Warning::FluteShort`
  (MCP kind `flute_short`), at the first cutting move below the flute's
  reach.
- The operation layer (`crates/app/src/operation/flute_law.rs`) runs it at
  generation for the 2D milling kinds whose depth is a stack of levels down
  from the stock: 2D Pocket, 2D Adaptive, 2D Offset and 2D Contour. It puts
  the warning FIRST in the warning list and a note with the tool's name
  FIRST in the notes: "EM-0800-4FL has a flute length of 10.000 mm; this
  operation cuts 20.500 mm below the stock top; the shank, wider than the
  cutter, is in the cut - ...".
- Block or advisory:
  - The shank is wider than the cutter, or the cut reaches past the shank
    into a holder that is wider: the body meets the walls that the flutes
    leave. The warning holds the post. The collision findings already held
    it in the #3319 case; this does not change.
  - The body is not wider (a reduced or same-diameter shank): the shank can
    rub the walls, but nothing certain collides. The warning is advisory.
    A program that posted before still posts. The collision laws still
    judge each move.
- The verifier's findings, the collision and rapid checks, and the text of
  `RapidInStock` (#3520) do not change.

**Why.** The user of #3319 read three rapid findings and looked for a
lateral rapid that was not there. The cause is one fact that is known at
generation: the depth that the operation asks for, and the flute length.
The first line now says it. The block follows the certainty of the
engagement, as the issue asks.

**Considered and dropped.**

- A refusal at generation (`GenError`), like the drill reach law. Dropped:
  the user then sees no path and no findings to compare, and a
  same-diameter shank is a valid plan for some shops.
- Only a note, with no warning. Dropped: a note is not in the MCP findings
  and does not hold the post, so an agent would not see the cause.
- A law inside `verify_bodies` for every kind. Dropped: it would reach the
  3D, turning, drilling and rotary kinds, whose depth is not "levels below
  the stock top", and it would move many pinned warning lists.
- The tool's name in the warning text. Dropped: `Warning` is `Copy` and a
  file format; the name rides in the note, and the warning says "the tool".

**Limits.** The depth is read against the stock's highest point. For a
stock with steps (a carried or cut stock), the depth "at that place" can be
smaller, so the finding can name a depth that is too large there. The
collision laws still give the exact moves.

**Cost to change later.** The kind name `flute_short` is part of the
project file and of the MCP output. To make the advisory case block, change
one arm of `Warning::blocks_post`. To judge more kinds, extend `judged` in
`flute_law.rs`.
