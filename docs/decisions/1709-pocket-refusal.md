## #1709 - Name the reason an empty 3D pocket cannot cut

**Decided:** Record a `too_narrow` boolean when a nonempty level target becomes
empty after erosion by the tool radius plus stock allowance. If the final path
is empty, use that evidence to name the tool diameter and stock to leave in the
refusal. Keep the existing model-filled message when no level reaches that
case. Both plain and offset-shell pockets use the shared ladder.

**Why:** An empty target and a target that cannot fit the cutter require
different user corrections. Including the stock allowance explains why an
opening wider than the tool alone can still be refused. The flag changes only
the empty-path diagnostic; it changes no target, motion, or refusal boundary.

**Considered and dropped:** Replacing every empty-path message with a tool-width
message would misdescribe a model that fills the whole pocket. Computing a
minimum opening width or changing the geometry is unnecessary for this
diagnostic and would expand the scope. Naming only the tool diameter would
omit the stock allowance used by the existing erosion.

**Cost to change later:** Small. The flag and message selection live in the
ladder, and three motion-crate regression tests pin the filled-model,
tool-width, and stock-allowance cases for both entry points.
