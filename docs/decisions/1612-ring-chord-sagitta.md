## #1612 - Bound every ring-prefix chord's sagitta

**Decided:** Check each prefix segment's closest distance to the ring centre
with a clamped projection. Require that distance to be at least
`rho - ARC_FIT_TOL - ON_RING_TOL`. Keep the existing vertex, end and sense
checks. A prefix that fails keeps the exact `push_zig` chord fallback.

**Why:** Vertices on the circle do not prove that their connecting chords
follow it closely. A radius-1 prefix with a 0.98-radian flat can otherwise
fly arcs about 0.118 mm beyond that flat. The closest-point check also
covers a final segment shortened part-way along a polygon chord.
`ON_RING_TOL` accounts for the existing lattice tolerance.

**Considered and dropped:** Endpoint-only checks leave the reported gap.
A midpoint-only check can miss the closest point when the shortened final
endpoint lies inside the ring. Rejecting every ring ramp would discard
valid helical arcs that already meet the tolerance.

**Cost to change later:** Adjust the local guard and its pinning tests.
No format, dependency or caller contract changes are required.
