## #2284 - Allow Deburr in a verified setup frame

**Chosen:** Add Deburr to the verified aligned-setup kinds and the supported
kinds named in the refusal message. Require a proof that a part turned about
two axes cuts the same path and posts the same program as a part with that
turn baked into its mesh. The part as modelled must cut differently or refuse.
The proof covers entity and selection sources, `TopOfModel` heights, the
analytic turned rim and a simulation without gouges. Three existing refusal
sentinels use Probe, which remains unsupported, instead of Deburr.

**Validation:** Both proof tests failed with the old allowlist and passed
with Deburr enabled. Restoring the old allowlist reproduced the failure;
restoring the fix passed. Separate mutations that read material or model
heights in the model frame also failed, then passed after restoration.
The quick gate passed all 15 steps.

**Why:** Deburr already consumes setup-frame geometry. Its generation arm
passes `material.facets` to the world-free `finishing3d::deburr` kernel.
`OpFrame::material_of` reaches `material::named_material` through the judged
material reader. Named material uses `setup_align::facets_in` and
`setup_align::entity_aabb_in`; selection sources union the same reads.
Model-relative heights use `setup_align::part_aabb_in`. The region projects
the supplied material, and the neighbouring-part and stock-source protection reads
use `setup_align::facets_in` or `setup_align::parts_facets_in`. Stock and
fixtures remain furniture in the setup frame. The missing verification entry
refused this existing path before it could run.

**Dropped:** Removing the refusal without a turned-versus-baked posting proof
would leave a wrong-frame cut undetected. Adding another transform inside
Deburr would duplicate the material reader and risk turning the part twice.
Changing the kernel is unnecessary because it receives owned geometry and
does not read the document world.

**Cost to change later:** A new Deburr world read must preserve the same
setup-frame contract and extend the proof when it adds another source or
height rule. The verification entry can be removed if that contract cannot
be proved; the existing refusal then blocks aligned generation by name.
