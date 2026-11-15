## #2009 - Show the checked floor of a machine-default holder

**Chosen:** In `tool_body_page::envelope`, display the landed profile
floored at the editor tool's parameter cylinder when a machine-default
body is unchanged. Match its facet allocation, +Z axis, zero offset and
spindle frame. Explicit bodies and edited or imported drafts keep their
existing placement preview. Read the current tool dimensions each time.

**Why:** Collision checks retain a conservative parameter-cylinder floor
for an assumed holder. Rebuilding the mesh preview creates an explicit
body and loses that floor, so a small default holder appeared smaller than
the checked shape. Reading the stored profile also preserves its exact
bands instead of deriving a slightly inflated envelope from its facets.
This changes only the editor display; collision and post behavior stay as
they are. It needs no command, dependency or process change.

**Dropped:** Changing the collision rule would weaken its conservative
assumption. Flooring explicit bodies would prevent their intended
replacement of the cylinder. Reusing only the rebuilt mesh envelope would
still lose the stored bands. Caching the effective floor independently of
the editor dimensions would show stale bands after parameter edits.

**Cost to change later:** Revise the localized envelope selection and its
seven UI tests for the checked floor, parameter edits, explicit bodies,
moved and replaced drafts, invalid placement and lathe-frame drafts.
