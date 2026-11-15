## #1502 - Part corners and circle centres in the WCS marker pick

**Decided:** with the WCS page's "Pick markers in viewport" armed, a click
that misses the nine box markers falls through to the part's own feature
points (corners) and round-edge centres (the opt-in `centres` kind of
#692, one run per click). A corner queues the existing undoable `SetWcs`
at the vertex's world position (`pick::align::vertex`, made `pub`); a
centre queues the existing `SetWcsCircle`. A click on nothing is left to the
pick mode as before.

**Why:** the issue asks for corners and hole/slot centres beside the markers.
It reuses the pick rule and commands the page already has; no new command.
Markers keep priority so the existing grid behaviour is unchanged.

**Dropped:** a new "Part points" anchor source (a third, fourth combo
entry) - more UI for the same click; hover highlight of the candidate
points while armed - would change `feature_click::ask` for every pick and
is a follow-up if wanted. MCP: `set_wcs_from_features` (Origin role takes a
vertex id or a circular edge's centre) and `set_wcs_circle_centre` already
reach the same set, so no `MCP:` follow-up.

**Cost to change:** one function in `crates/ui/src/wcs_circle_pick.rs`.
