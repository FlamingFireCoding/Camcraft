## #1557 - the through-hole break-through is a stated parameter

**Decided.** Drill, Bore and Thread carry `break_through: Option<f64>` (mm),
shown as "Break-through" in the hole kinds' Levels group (Default / a
length in the document unit), saved as `break_through` in the project file
(written only when stated) and settable over MCP through the same params
JSON. `None` is each kind's old figure: 1.0 mm for Drill and Bore
(`THROUGH_CLEAR`), one pitch for an internal Thread - so every existing op
generates and posts byte-identical and keeps its digest (the digest writes
the value only when stated). Generation notes the run and the deepest tip
("through holes run 1.000 mm past the part bottom (Break-through), the
deepest to Z..."), in the document unit. A negative or non-finite value is
refused at creation and by the kernels. On a lathe's axial or cross Drill a
stated value is refused by name (the turret's exit stays its own law).
`PROJECT_VERSION` steps to 63 so an old reader refuses a file instead of
dropping a stated break-through.

**What lies below.** Where workholding or a table is modelled in the
setup, the fixture laws already judge the tip: a break-through that reaches
modelled steel is a `FixtureHit` on the flute and holds the post (pinned);
air between modelled parallels stays legal. Where nothing is modelled, a
tip under the stock bottom is **named, not refused**: a `Break-through
below stock:` note gives the hole, the Z and the depth under the stock.

**Why not refuse under an unmodelled stock bottom.** With nothing modelled
we do not know what the stock sits on - parallels (air), a spoil board
(cutting into it is how a router through-cuts) or the table. Refusing would
stop every through hole in a plate sawn to size, a program that posts today,
which the issue's own Done means ("noted or refused") and the byte-identical
rule both argue against. Modelling the support is the way to have it judged.

**Considered and dropped.** A tool-relative default (drill point length):
it changes today's output; it can come later as a third default. A
break-through per hole: the issue asks for an op parameter, and holes are
re-detected.

**Cost to change.** Turning the note into a refusal is one branch in
`operation::break_through::judge`.
