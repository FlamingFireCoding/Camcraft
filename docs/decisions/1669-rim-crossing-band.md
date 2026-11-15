## #1669 - Rim vertex crossing distance band

**Decided:** A rim vertex crosses a straight trace only when its neighbours
stand on opposite sides farther than the existing `CROSSING_TOUCH` distance
(1e-9 mm). Scale orientation values by the segment length so this distance
stays the same for short and long traces. Apply the same band to signed
radial distances for an XY arc. Polyline footprints keep the straight rule.

**Why:** Exact signs read a nearly coincident wall as an open rim crossing.
That excuses the whole move from the wall check. A neighbour inside the
band now counts as a touch, so the move still answers to the walls. True
crossings beyond the band still pass the rim classification.

**Considered and dropped:** Changing the global crossing tolerance would
change unrelated crossing laws. Sampling arcs would add approximation and
miss the exact radial distance already available here. Keep both existing
algorithms and restrict this change to their vertex-neighbour test.

**Evidence:** Unit pins cover either near neighbour, reversed crossings,
short and long segments, polyline footprints, and the XY arc branch.
The wall pin uses nearly vertical lattice vertices: region normalization
keeps their bends while their perpendicular offsets stay below the band.
All three pins fail with the old code and pass with the fix. Restoring each
old guard separately makes its pins fail; restoring the fix makes them pass.
The quick gate and camcraft-sim all-target clippy both pass.

**Cost to change later:** Local to the two vertex tests and their pins.
A larger tolerance needs separate evidence about which wall excursions may
be treated as touches. No global tolerance or posted output changes here.
