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

**Refusals that this change removes on purpose.** The old horizontal-stance
refusal ("a laid bar is finished with passes along it: Rotary Parallel") was
not a safety law. It said only that the height law had no world-Z divisor.
This change gives the laid stance its own station law, so two app tests that
pinned the old refusal now pin the new result:

- `operations::the_rotary_twins_refuse_each_other_stances_by_name` becomes
  `a_standing_bar_refuses_rotary_parallel_by_name`. The standing-bar refusal
  of Rotary Parallel stays, with its name check. The laid twin moves to
  `a_laid_bar_cuts_rotary_contour_rings_about_x`. That test requires status
  Ok, zero blocking warnings, radial poses about X, ball centers inside the
  axial station window, rings at more than three stations, and the
  column-verdict refusal of a rotary program.
- `setup_align_multiaxis::turned_rotary_contour_cuts_posts_and_fits_axis_like_the_baked_part`
  expected a refusal for the as-modelled stance (the bar lies flat along Y: `unturn_xz` maps the bar axis z to y). That
  stance now cuts. The test requires a fitted axis along Y, radial poses about
  Y on each posed move, and a path that differs from the turned path. The
  turned and baked cases keep their equal-path and equal-post checks.

Dropped: keep the refusal for these two fixtures only. That would make the
generator depend on the caller, and #3000 asks for exactly this stance.
Dropped: change the tests to "differs or refuses". That accepts a refusal,
so it is weaker than the old assert.
