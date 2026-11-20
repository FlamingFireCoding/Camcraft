## #3000 - Rotary Contour rings about a horizontal axis

**What was decided.** For a fitted axis with `abs(axis.dir.z) < 0.0175`,
Rotary Contour reads top and bottom as axial ball-center stations.
It reads clearance and retract as tool-tip radii about the fitted axis.
These values do not depend on the axis line's world Z position.
Other axis stances keep their existing world-height conversion.

The generator reuses the waterline contact law and the radial link builder.
A coordinate permutation preserves each ball center between their planning
frames. Arcs become chords before this conversion. The existing pose law
then places the tool tip under that same ball center.
The generator refuses unsafe clearance and retract radii by name.
It also refuses inverted or non-finite station bounds by name.
This change adds no dependencies.

**Why.** A horizontal axis has no usable world-Z divisor for axial stations.
Explicit station bounds permit complete rings about X.
Radial clearance and retract moves let the tool leave each ring safely.
Reusing the existing contact and link laws limits the new geometry code.
Keeping the other stances unchanged preserves their established height contract.

**Considered and dropped.** Removing the stance guard alone retains division
by zero and does not provide safe links. A global conversion of rotary heights
to radii belongs to #2774, which has separate live work.
Changing standing-contour semantics would extend this fix beyond its defect.
A new ring-generation algorithm would duplicate the existing waterline law.

**Cost to change later.** A common rotary height contract will need coordinated
parameter, UI, and compatibility changes. The station and clearance pins must
change with that contract. The ball-center conversion and radial link reuse
can remain if their geometric conventions remain the same.
