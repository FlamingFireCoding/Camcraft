## #2435 - a no-pick 2D Chamfer on a modelled chamfer rides the wall; verify holds the tip to it

**What was decided.**

1. *Generator.* With no picks, a 2D Chamfer sections the source twice: at
   Top of Part Z (the edges to break) and at the corner depth
   `top - width * tan(angle)` (the wall the cone's tip touches). Each top
   loop rides the corner-depth loop within `width` of it in plan. Where the
   wall is vertical the two loops coincide and the top loop itself is
   ridden, unchanged - every sharp-edged part posts byte-identical
   G-code. Where the model already carries a bevel, the tip follows the
   bevel: width = bevel width puts the tip at the bevel's foot on the wall
   (the issue's `Y15 Z-3`), a smaller width puts it on the bevel face (the
   cone's face then coincides with the modelled bevel and cuts nothing),
   a larger one cuts a larger chamfer through it. A top loop with no wall
   within `width` under it (a blind pocket or an island shallower than
   the chamfer's depth) refuses by name with the spot. A corner-depth loop
   no top loop matches (a lower ledge, the plate under an island) is not
   ridden: it is not an edge at the top plane.
2. *Verify.* The program sim credits a 2D Chamfer's edge-break records
   only where the cone's tip stands on a wall. The facet-top probe in
   `sim::stock_sim::wall_anchor` samples inside each declared segment.
   The highest facet 0.05 mm (`WALL_PROBE`) off the edge, on at least one
   side, must stand no higher than the tip (within the verdict tolerance).
   Samples are at subdivision midpoints, not vertices: a segment normal
   at an inside corner can put the air probe on the adjacent top edge.
   A record that fails is dropped (logged), and its columns are
   judged against the model as it is, so the V the old ride cut into the
   bevel face is a blocking gouge. Deburr records are not held to this:
   a deburr legitimately breaks obtuse 3D edges (a drafted wall) whose
   plumb line runs inside the material.

**Why.** The chamfer kernel's law is "the tip is on the wall, the
generatrix on the face"; the region it was handed (the top-plane section)
is the wall only for a vertical wall. On a bevelled edge the top-plane
outline is the bevel's top edge, inside the wall by the bevel's width,
and the ride cut a V 0.71 mm deep into the part. The width then moved only
Z, because the loop never changed. The sim's edge-break credit accepted
the V because the record declared that same false edge: a bevel declared
on any edge was credited, wall or not. The kernel's own comment said a
picked edge that is not the chamfer's wall "is caught by the stock
simulation"; it was not.

**Considered and dropped.**
- *Refuse when the edge is already chamfered.* Safe, but it refuses the
  common case (a modelled chamfer the shop still wants machined, or
  re-cut larger) that the wall-following ride handles exactly.
- *Ride only the corner-depth section.* Simpler, but a width deeper than
  an island's wall would silently ride the plate's outline instead of
  the island's edge; the match against the top edges refuses that by
  name instead.
- *Hold every edge-break record (deburr too) to the wall.* It would
  convict a deburr on a drafted wall, a real obtuse-edge break.
- *Probe the corner bisector at each vertex.* Interior samples avoid the
  adjacent top edge without a separate rule for corner directions.
- *Fix only the generator.* A hand pick of the bevel's top edge chain
  still posts the same V; the verify law is what catches every route to it.

**Cost to change later.** The match slack (0.05 mm) and the facet-top
wall probe (`WALL_PROBE = 0.05` mm) are separate constants, in
`machining2d::chamfer_ride` and `sim::stock_sim::wall_anchor`. The wall
probe does not depend on the simulation lattice spacing. A gap narrower
than the probe can put both probes over material, drop the credit, and
refuse the program. A smaller probe can detect a narrower gap but is
more sensitive to floating-point error near a facet edge.
