## #2919 - Read short STEP unit names before model import

**What was decided:** Change the parsed record names `GUAC` and `SUNT` to
`GLOBAL_UNIT_ASSIGNED_CONTEXT` and `SI_UNIT`. Do this before the existing
unit checks and the topology read. Keep one canonical exchange structure
for both consumers.

**Why:** The pinned parser expands some short names, including `LNGUNT`,
but leaves these two names unchanged. The unit reader looks for their long
names. It can otherwise miss the context and use millimetres for an inch
or metre model. The existing checks can validate the declared units once
both names use their long forms.

The mappings come from the public
[ISO STEP short-name registry](https://standards.iso.org/iso/10303/tech/short_names/short-names.txt).
The parser exposes `Exchange.entities`, `Entity.records`, and `Record.name`
as public fields. Its record lookups compare those names directly. The
change uses that public API and does not change entity references or
parameters.

The authored fixtures `testdata/models/short_units_inch.step` and
`testdata/models/short_units_metre.step` use our unit-box geometry. The
regression tests check the imported dimensions, volume, and declared units.
Separate cases exercise each alias and retain long-name controls.

The agreement corpus lists each model fixture explicitly. Its new rows
record the expected differences for these two fixtures. The model importer
reads their scale correctly. The raw tessellator still assumes millimetres,
and the truck reader cannot read the short geometry names. Extending that
list keeps the corpus inventory complete; it does not change those readers.

**Considered and dropped:** Text replacement could change strings and
comments. A guessed scale could silently produce wrong geometry. A blanket
refusal would reject legal files that the existing unit reader can handle
after name expansion. A dependency update would widen this fix beyond the
two missing mappings.

**Cost to change later:** Remove the two mappings when the pinned parser
expands both names itself. Keep the inch and metre regression fixtures and
their scale checks.
