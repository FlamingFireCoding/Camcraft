## #2005 - A tool body replaces the parameter holder in the editor preview

**Chosen:** When body bands are present, draw the cutter and shank from a
copy of the tool with zero holder length. Use the body bands to paint and
size the holder. With no body, keep the original parameter profile.

**Why:** The editor painted the parameter holder as well as the body that
replaces it. The duplicate fill and outline obscured the displayed shape,
and the unused holder dimensions distorted the preview's frame. Removing
the parameter holder before painting fixes all three effects. Collision
checking and simulation already use the body's band stack.

**Dropped:** An opaque body overlay would leave the unused outline and
frame dimensions. Changing collision or simulation would change behavior
that already selects the body. A broader preview refactor would add scope
without being needed for this fix. The separate mismatch between raw body
bands and the conservative parameter-cylinder floor for an assumed
machine-default holder is tracked in #2009.

The existing pages module already exceeds the file-size guideline, so the
production edit stays small and the paint tests live in a separate file.
This is pure rendering: it needs no new command or dependency. Revisiting
the choice costs a localized preview change and updates to the fill,
outline, framing, and no-body control tests.
