## #1566 - Clear wide bores with concentric rings

**What was decided.** Bore clears each cut level from the centre outward.
A radial sweep opens each next band, then a closed ring clears it.
The last ring finishes the wall. The existing cutter-centre disk certifies
the entry and links through the existing level ladder.

Keep the existing single-ring path when the wall-ring radius is at most
twice the cutter reach at the floor. Otherwise, use
`ceil(wall_ring_radius / (0.8 * floor_reach))` rings. Divide the radius
equally between them. Refuse more than 4,096 rings before allocation.
Detect holes uses the same bounded plan as generation.

For a blind hole, use the cutter's flat bottom radius:
`tool_diameter / 2 - corner_radius`. For a through hole, use the corner's
torus reach at the through allowance. Refuse a floor band too small to
resolve. This includes a ball cutter at a flat blind floor. The normal
corner fillet where the floor meets the wall remains intentional.

**Why.** The centre dive and one wall ring leave an uncut band in a wide
hole. Overlapping rings clear that band. The radial sweep puts each ring
start in the column just cut. The blind floor needs the flat bottom reach;
the full cutter radius can hide an interior ridge. A shared plan keeps
detection and generation consistent. The ring limit bounds allocation.

**Considered and dropped.** A spiral or a new helix entry adds a separate
algorithm and entry design. Removing only the old width refusal would
leave an unsafe single-ring path. Keeping the full radius for blind holes
would retain the floor ridge. Unlimited rings could exhaust resources.

The app verification fixture uses 0.1 mm wall allowance. Its 64-facet STL
wall lies inside the nominal circle by about 0.045 mm at the large hole.
The allowance keeps the test path inside that model. Motion coverage tests
also exercise zero wall allowance.

**Cost to change later.** A configurable stepover or spiral must preserve
floor coverage, the wall ring, entry and link certification, and the shared
detection limit. It also needs a bounded allocation rule. The current
policy adds no project-file field.
