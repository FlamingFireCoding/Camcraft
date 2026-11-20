### Refuse an item with no unit context when the file states a non-radian angle unit (#3079)

**What was decided:** The STEP model read refuses each solid item that no
representation gives a unit context when a context of the file states a
plane angle unit that is not the radian. The refusal names the item and
the angle unit. An item is listed when a representation with a context
lists it, or when it is a piece of such a solid (the reader gives a piece
the context of its solid). The check is `UnlistedAngles` in
`crates/kernel/src/step_units_guard.rs`. The seam applies it to the solids
that the reader builds, and to the items that the reader fails to read.
For a failed item the unit reason replaces the read message, because the
wrong angle unit is the first cause of the failure.

**Why:** The reader gives an item with no context the default units: its
angles are read in radians. In a file that states degrees, the item then
has wrong geometry. That is a silent wrong answer (Prime Directive). The
issue asks for the rule of the length unit: keep such an item only when
every stated angle unit is the radian, or when no angle unit is stated.

**Considered and dropped:**
- Stacking on PR #2927, which edits the solid build loop of the same
  function. It has `needs-rework` and `conflicts`. This change adds its own
  lines before the failure loop and does not touch the lines of #2927.
  Whichever PR merges second refreshes.
- Refusing only items that use an angle (a cone, a torus). The issue asks
  for the item rule. An angle can hide in many entities, and a missed one
  is a silent wrong answer.
- Refusing the whole file. The other items of the file have a context and
  read correctly. The importer already refuses a file that reads only in
  part, by name.

**Cost to change later:** Remove the check when the pinned reader gives an
item with no context the units of the file. Keep the three pins in
`crates/kernel/tests/step_short_units.rs`.
