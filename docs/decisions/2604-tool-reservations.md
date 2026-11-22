## #2604 - Machine tool reservations

**Decided:** A machine can state `tooling.reserved` entries with a `number`,
a `name`, or both, and `use: "probe"` or `use: "tool_setter"`. Numbers must
be integers in 1–99999. Zero is not a tool. Malformed entries and duplicate
selectors refuse by name. Name comparisons ignore ASCII case.

New tools use the active machine's reservations. Edits check the machines
that use the tool. Existing tools keep their numbers. A probe takes its
declared pocket; an occupied probe pocket refuses. There is no tool-setter
tool kind, so neither cutters nor probes can take a setter reservation.
The 18 shipped Tormach mill and router files reserve T99. Generic, lathe
and plasma machines reserve nothing unless their file states a reservation.

Existing commands and their MCP tools share the guard. Verification and
posting refuse unsafe assignments, including old documents. The standalone
post CLI checks the stream before it writes output. It keeps the physical
tool selection across sections and requires a fresh kind for each new tool.
A missing selector or required kind refuses. The kind can precede or follow
the tool selection. Allocation uses a hash set. CLI reservation indexes are
built once, so these checks do not repeat full scans.

**Why:** A tool call must not select a probe or setter for cutting. Machine
data states those identities; a guessed convention cannot establish them.
The stored machine field remains in the public project fixture. Its exact
round-trip assertion stays intact. This branch stacks on #3157 because both
changes edit `parse_mch_with`.

**Dropped:** Silent renumbering could change an existing machine assignment.
Inferring T1 for other manufacturers could reserve a real cutting tool.
A new command would duplicate the existing edit path. A reserved name does
not select a controller dialect; this change does not alter emitted syntax
or infer name-based tool calls.

**Cost to change later:** More reservation uses need explicit tool-kind
rules and tests. Controller name-based calls need a separate post change.
Changing the stored shape needs project compatibility handling. Existing
machine assignments must remain explicit during either change.
