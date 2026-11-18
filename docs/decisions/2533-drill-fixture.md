## #2533 - Restore the Drill test fixture

**Chosen:** Set `break_through_from` to `camcraft_app::drilling::BreakThroughFrom::Tip`
in the height advice fixture. Keep both machine transition tests. Add a test
that checks the mill default when the document has no machine.

**Why:** The fixture predates the new field. `settle_drill_break_through`
assigns `Tip` to legacy Drill operations from projects before v68. This
explicit value preserves the fixture's meaning and restores the test build.
The change does not change production behavior.

**Dropped:** `FullDiameter` is the default for new operations. It does not
preserve the legacy reference used by this fixture.

**Cost to change later:** Change one initializer if the fixture needs a
different reference.
