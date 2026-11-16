## #1170 - Wire EDM 4-axis: per-ruling widening and a stated verify tolerance

**Decided:**

*Widening.* A 4-axis cut has a `widening` choice. `PerRuling` is the
default: each ruling stands off the two profiles by what its own twist
needs. `Uniform` is the earlier behaviour: every ruling stands off as far
as the worst one. A project file without the key reads as `PerRuling`,
and the key is written only for `uniform`. The widening is in the stamp
digest, because it moves the wire.

The per-ruling path adds a transition ruling 1 mm (`TRANSITION`) in from
each end of a long span, so a widening ends near the twist and not across
a whole flank. A transition that changes nothing is pruned, and the path
is read again after the prune. Each end of a ruling takes its own scale;
the rulings of a fan take one scale together. After the path is built,
the existing ruled kerf law judges the emitted wire at every height it
reads (the whole-law guard). A path that fails it is refused by name; it
is never emitted.

*Verify tolerance.* A 4-axis cut states `verify_tolerance` (mm, default
0). The wire verify's kerf law lets a reading enter the model's facets by
no more than that value. The threading law and the slug law never use it.
The operation door accepts 0 to 0.25 mm (`VERIFY_TOLERANCE_MAX`). The
verify refuses, as unreadable, a value that is negative, not finite or
not below the kerf offset. The value is not in the stamp digest: the
verify reads it at each post, and the stored path does not depend on it.
The post gate's report line states the tolerance and the deepest reading
that it let pass.

*Gouge text.* The kerf law now reads every plane and does not stop at the
first gouge. The finding keeps the first plane's gouge and adds the
deepest reading at any height. A 4-axis gouge with no stated tolerance
adds a hint that names the model's chord tolerance. `slugs_said` moved
from `app` to `sim`, where the words of the other wire findings are.

*Posted gate.* `posted_gates/wire_4axis.rs` compares its direct
regeneration with the app's golden by `same_program`: the same lines and
words, each number within 3 counts (3 um). The two goldens
(`wire_4axis_punch.cl`, `wire-iso/wire_4axis_punch.nc`) are regenerated;
their pose words moved by 1 um.

**Why:** A uniform widening makes the straight stretches of a punch
oversize by the worst twist's amount, which the issue names as the
defect. The per-ruling result is not unique: several wires hold the kerf
law, and the one found depends on the vertices of the profile rings. The
whole-law guard therefore decides safety, not the construction. A
tessellated loft's facets stand past its ruled surface by up to its chord
tolerance, so a correct cut reads as a gouge of that size. Only the user
knows the tolerance the model was made with, so it is a stated parameter
with a strict default, and the report shows what it excused.

**Considered and dropped:**

- An absent `widening` key that reads as `Uniform`. It keeps old files
  byte-stable, but the default then differs between a new and an old
  operation. An old operation's stamp is stale after this change in both
  cases.
- One scale for each ruling across its two ends. It widens the flat end
  of a ruling whose other end twists.
- No transition rulings. The widening then spreads over the whole length
  of a flank next to a twist.
- An enum of the app's own for the widening. The motion type has two
  plain variants; a copy adds a mapping and no meaning.
- A verify tolerance for each machine or each document. The chord
  tolerance belongs to the lofted model of one cut.
- A tolerance derived from the mesh. The verify cannot know the true
  surface from the facets alone.
- A tolerance for the 2-axis and tapered cuts. Their walls are planes or
  cones over the profile, and the issue asks for the 4-axis cut only.
- An exact compare in the posted gate. It fails on a 1-3 um difference
  that comes from the ring representation, not from a defect.

**Cost to change later:** A change of the default widening changes
posted words of each 4-axis cut and makes each stamp stale. A different
transition length changes the goldens. Moving the tolerance to a
different owner needs a project-file migration for `verify_tolerance`.

**Known limits:** The `tangled` refusal of the per-ruling path has no
fixture that reaches it. The uniform widening folds on the sheared
fixture with one twisted flank and is refused there; per ruling cuts it.
The posted program was not run on a real EDM control.
