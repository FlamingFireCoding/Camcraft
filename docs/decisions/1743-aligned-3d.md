## #1743 - Run 3D operations in the part-aligned setup frame

**Decided:** Allow `Pocket3D` and `Finish3D` in an aligned milling setup.
Allow their area controls through the same door. Keep the refusals for
other operation kinds, 2D area controls, tilted planes and non-mill
machines.

**Why:** The existing operation frame already reads part facets, model
heights and picks in the setup frame. Generation and verification use those
facets. Stock and work offsets already belong to the setup frame. The
remaining blanket refusal prevented these existing paths from running.
`Pocket3D` includes the adaptive strategy, so it needs no separate generator.

**Considered and dropped:** Removing every alignment guard would enable
operation kinds that this change does not verify. A second transform or a
duplicate generator would repeat the existing frame logic. Mapping the CL
path back to the document frame would move the path away from the part as
the setup holds it. The output stays in the setup frame.

**Evidence and limits:** The pins compare an exact X half-turn of an
asymmetric tapered wedge with the same geometry already transformed in an
unaligned setup. The toolpaths and posted G-code match. Each case has no
blocking generation warning, and stock simulation succeeds with zero gouge
cells. The cases cover offset and adaptive roughing, waterline finishing,
model-relative heights, a controlled pocket and picked finish geometry.
The controlled pocket also differs from the default path. Restoring the
old allowlist or the old area refusal makes the relevant pins fail.
The pins do not exercise the user's filleted part, other finish patterns,
all roughing options or a physical machine.

**Cost to change later:** Another operation kind or area-control family
needs a frame-read audit and equivalent generation, verification and post
pins before its guard is relaxed. This change adds no stored data or
project migration.
