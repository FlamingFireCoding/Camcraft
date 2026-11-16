## #1891 - Bounded 3D edge-break volumes beside legacy cones

**Chosen.** Keep legacy vertical-cone records and add an optional framed
volume. Each nondegenerate polyline segment has one outward bisector and
one across-edge direction, both unit vectors and perpendicular to each
other and the segment tangent. Frames rotate with a tilted work path.
Legacy cones continue to drop when the frame tilts.

**Dimensions and profiles.** A framed width is the full across-edge span,
from `-width / 2` to `width / 2`. Depth is the maximum inward distance
from the selected edge. Segment end planes bound the sweep; the actual
part clips this cap. Width and depth must be finite and positive, and
depth must not exceed width. The operation's own width still bounds each
record through the existing #529 check, with its existing lattice slack.

At across coordinate `s`, let `sag(s) = R - sqrt(R² - s²)`. A flat cap
ends at the declared depth. A concave ball cap ends at `depth - sag(s)`.
A convex rounded cap ends at `depth - sag(width / 2) + sag(s)`. Radius
must be finite, positive and at least half the width. The edge sag must
not exceed depth, so the full profile stays nonnegative. These explicit
laws avoid inferring permission from the generated cutter moves.

**Why analytic connected intervals.** Clip each world-Z column against
the segment planes and profile, then join only the intervals connected
to the model surface. A detached lower cap cannot forgive cutting the
material above it, and a deeper disconnected interval cannot extend the
credit through a gap. Cuts below the resulting surface remain gouges.
This works for vertical and oblique edges without projecting their
width onto XY. Legacy cones keep their existing surface tolerance.

Circle intersections use exact quadratic roots rather than sampled
profiles. The sag uses a rationalized expression to retain shallow arcs
at large radii. The root calculation normalizes the column's direction
in the cross-section before squaring it; tiny finite rates must not
underflow into an unbounded arc credit.

**Persistence and refusals.** The volume is an optional project-record
extension. Framed records require a zero legacy slope, so older readers
drop them and judge the sharp model instead of misreading them as cones.
Existing 2D chamfer and v1 deburr records keep their wire shape and law.
Framed endpoints retain full precision because micron snapping can make
a short oblique segment disagree with its frame. Legacy endpoints still
use their existing lattice.

The extension is read as raw JSON before strict typed validation. A bad
profile, unknown key, invalid dimension or corrupt frame drops that
record, not the entire project and not merely its extension. Diagnostics
identify the edge-break record and its index; the operation-bound warning
names the operation. This preserves conservative verification without
changing the unrelated project reader API or project version.

**Dropped.** XY distance for 3D breaks loses vertical and oblique bounds.
Sampled profiles can miss a boundary and forgive material between samples.
Replacing all cone records would change existing chamfer/deburr behavior
and require unnecessary migration. A project-version or process change
is unnecessary because older readers reject the zero-slope extension
safely. New 3D toolpath generators belong to the surrounding #1827 work;
this issue adds the records and verification they consume. Existing
generators, cutter moves and golden files keep their behavior.

**Cost to change.** Moderate: the public CL record has an optional field
and new frame/profile types. Changing the cap convention would require
updating its geometry and project DTO conversion together. The wire
format remains local to the edge-break DTO; no dependency change is
needed.
