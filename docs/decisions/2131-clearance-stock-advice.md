## #2131 - Show clearance advice on the Heights tab

**Decided:** Show the full sentence from `operation::clearance_in_stock_advice`
on the Heights tab through the form consequence helper. Read the live draft
after the field editors run. Pass its setup, source, parameters, and the
existing operation name when one exists.

Reuse the display cache in `height_memo` for this advice. Key the result by
its geometry and setup inputs, the program sequence and operation values,
the draft source and parameters, the operation name, and the display unit.
Generation, verification, and posting continue to read their own inputs.

**Why:** The UI must give the same advice as the MCP tools before Generate.
The draft values make the sentence follow an unapplied clearance edit. The
program inputs make it disappear when a cutting operation moves ahead. The
cache avoids another geometry read while a still frame has the same inputs.

**Considered and dropped:** A UI heuristic would duplicate the existing
stock and operation rules. A Generate-only warning would leave the reported
gap. PNG assertions would add release-only evidence where structural text
assertions can pin the behavior. A document-version key alone would miss
unapplied draft changes and could retain stale advice.

**Cost to change later:** A presentation change stays in the UI and its
structural tests. A change to the advice inputs must update the cache key
and its invalidation tests. This decision adds no crate dependencies and
changes no generation or post rules.

**Review fix decided:** Include the effective machine kind from
`machine::kind_of_setup` in the advice cache key.

**Why:** A setup without its own machine follows the document machine.
`LoadMachine` must invalidate cached drill advice when that machine changes
between mill and lathe. The effective kind also respects a setup's own machine.

**Considered and dropped:** The document machine's identity would cause
unnecessary rebuilds for changes that keep the same kind. It would also
describe the wrong input when the setup has its own machine.

**Cost to change later:** Update this key and its transition tests if advice
starts to read machine properties other than kind.
