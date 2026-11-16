## #1497 - Sodick wire post: the 4-axis refusal stays; a lean before the first rapid is refused

**What was decided.** `assets/posts/sodick-wire.cps` keeps refusing a
4-axis Wire Contour (`PARAM WireFourAxis`) by name: no G74 / G75 and no
U V words are written. The one change is a refusal gap closed: a
`WireU` / `WireV` stated before (or on) the section's first positioning
move used to be cleared by `onRapid`, so the post wrote `G40 G00`, `T91`,
the `C` word and `G01` as if the wire stood upright. That lean is now
refused by name ("a leaning positioning move (upper guide offset)") and
the whole section is not threaded or cut; the next section posts as
before. A vertical 2-axis stream posts byte-identically (the golden
`nc/sodick-wire/wire_slug_punch.nc` is unchanged).

**Why the refusal stays.** What the public Sodick source states
(Sodick Inc., "Wire EDM Machine Operation", 1998, Mark 21 / EX21, read on
easymanua.ls: A600 pp. 6-2 to 6-8 and A320D pp. 8-12 to 8-14):

- G74 "4 Axis On" and G75 "4 Axis Off" are modal (p. 6-4).
- U and V move the upper guide parallel to X and Y; for 4-axis they
  are "incremental value from the X or Y-axis plane in an absolute
  mode" (p. 6-2), and "U or V axis values are incremental from the X Y
  values, in absolute from it's present position" (p. 6-8). Offsets from
  X Y, then - but the "absolute" half of both sentences is not
  explained, and no G74 program example is given.
- For 4-axis the control's SET screen takes Table To Program ("Distance
  from worktable to bottom of part - X Y Axis Plane") and Table To Next
  ("Distance from worktable to top of part") (p. 8-13); TP / TN words can
  load them from the program (p. 6-3).

So Sodick's X Y are the wire's point on the part's bottom plane and U V
are (probably) the offset at the part's top plane, both measured from the
worktable. Camcraft's stream instead carries the lower guide's X Y and
the upper guide's offset from it, from the machine's `edm` guide planes
(docs/CNC-INTERMEDIATE.md). Converting one into the other is a straight
line through two Z heights, but it needs the part's bottom and top as
heights above the worktable, and neither the machine file (no table
datum) nor the stream (no workpiece planes in this form) states them.
Writing the guide offsets as U V would put the taper on the wrong planes
- a silently wrong 4-axis cut. Under the safety floor (posted G-code),
the refusal is the honest output until (a) the "absolute" wording and the
U V plane are confirmed by a later public Sodick source or a machine
owner, and (b) the machine file states the table height of the guide
planes (or the stream states the part's bottom and top) so TP / TN and
the conversion can be written from stated values.

**Considered and dropped.** Writing G74 with U V copied from `WireU` /
`WireV` (wrong planes: the offset at the guides is larger than at the
part's top). Assuming the lower guide sits on the worktable (not stated;
Sodick's own "Table To Lower" constant says it does not). Writing TP / TN
from `StockZTop` / stock bottom (work-offset frame, not the worktable).

**Cost to change later.** One onParameter / startCut change in the post
plus a machine-file field for the worktable height of the guide planes;
the refusal tests in `crates/post/tests/sodick_four_axis.rs` then turn
into a golden.
