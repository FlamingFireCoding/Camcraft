## #2468 - Match the air-entry note to the tool entry law

**Decided:** Emit `AIR_ENTRY` only when `air_entry` and
`p.links.approach.center_cutting` are both true. Keep `LevelPlan.air_entry`
as the geometric fact that the planned start is clear of stock.

**Why:** The ladder applies the same centre-cutting condition before using
air entry. A tool that is not centre-cutting keeps its existing entry law,
so saying that it plunges in air misreports the resulting path. This guard
corrects the note without changing motions.

**Considered and dropped:** Changing the ladder's entry law would change
machining behaviour to fix a diagnostic. Moving note emission into the
ladder would expand the change without improving the current condition.
Changing `LevelPlan.air_entry` would mix tool policy into its geometric
meaning. Keep the correction at the existing note emission site.

**Cost to change later:** If the ladder permits other tools to use air
entry, update this note condition and the two tool-case regression pins
together. No stored data or public API migration is needed.

### PR #2477 review fix

**Decided:** Merge `upstream/main` and use `PathBuilder::approached` for
the air-entry test fixture's clearance-plane seed, following #2420's
approach law. Extend the fixture test to assert the initial rapid. Keep
the source law test unchanged.

**Why:** The fixture must obey the same approach law as a generated path.
The review failure came from its plain builder, so no production path
change is needed.

**Considered and dropped:** Weakening the law or excluding this fixture
would hide the regression instead of correcting it.
