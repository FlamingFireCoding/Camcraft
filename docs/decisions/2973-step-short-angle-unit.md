## #2973 - Read the short name of the plane angle unit, refuse an angle unit that cannot be read

**What was decided:** Expand two more ISO 10303-21 short names before the
STEP model import reads its units: `PLANUN` (`PLANE_ANGLE_UNIT`) and `GC`
(`GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT`). The expansion of #2919 (`GUAC`,
`SUNT`) moves with them into one table in `crates/kernel/src/step_units_guard.rs`.
After the expansion, a new check refuses a file by name when one of the
units of a representation context is:

- marked as a plane angle unit, but is not an SI radian, or is not a
  conversion of a unit that resolves to a radian with a positive finite size;
- not marked as a plane angle unit, but measures a plane angle all the same
  (a prefixed SI radian, or a conversion whose measure is a
  `PLANE_ANGLE_MEASURE_WITH_UNIT` or whose base unit is a plane angle).

**Why:** The pinned parser leaves `PLANUN` short. The units reader of
solidcore-step then does not see the plane angle unit and uses radians. A
cone semi-angle of 1 degree is read as 1 radian. With our fixture the
pcurve check refuses the solid with a misleading reason ("an edge lies
8.32 mm off the face's surface"). A file whose edges do not hold the
surface to the angle could pass with the wrong shape. The check is the
second layer that the issue asks for: an angle unit that a later short
name or a malformed file hides is refused by name, never read in radians.
Without the `PLANUN` mapping the check alone refuses the fixture by name.

`GC` changes only the distance uncertainty that the reader uses. The issue
asked to check it in the same work. A unit test pins that the uncertainty
of the fixture is read.

The two short names come from the issue's reading of the public
[ISO STEP short-name registry](https://standards.iso.org/iso/10303/tech/short_names/short-names.txt)
(`PLANUN` at line 1554). The worker could not fetch the registry again in
this session, because the fetch needed an approval that was not available.

The new fixture `testdata/models/short_units_degree_cone.step` is our own
draft frustum: base radius 10 mm, height 10 mm, semi-angle 1 degree. A 1
degree draft is a common mould angle. It is also an angle where a radian
reading stays inside the legal range of a cone (0 to 90 degrees). The
volume and the top radius of the frustum depend on the angle.

The STEP agreement bench needs one pinned row for each fixture. Its truck
side is a recording, and #2790 removed the truck reader. A recording for a
new file cannot be made, and a made-up one would be false. So the bench
gets a `Truck::NotRecorded` variant. Its row judges only the solidcore side
and the length unit. The raw tessellator of the bench does not expand the
short names, as #2919 recorded for the length unit. So the pinned mesh
verdict for this file is the refusal that the tessellator gives.

**Considered and dropped:**

- Map `PLANUN` only. A file with a short name that we do not know yet would
  still have its angles read in radians without a word.
- Refuse each unit that is not marked. A legal SI radian without a prefix
  reads correctly as radians, and many files list a steradian unit. A
  blanket refusal would reject legal files.
- Patch the pinned solidcore parser or its unit reader. That widens the fix
  to a dependency update.
- Put the fixture outside `testdata/models/` to avoid the bench row. The
  issue asks for it in `testdata/models/`.

**Cost to change later:** Remove the table rows when the pinned parser
expands the names itself. Keep the cone fixture, its volume pin and the
refusal pin.
