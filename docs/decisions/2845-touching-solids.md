## #2845 - Keep contact faces with their closed solids

**Chosen.** Link fragments at a four-use edge only when one complete
matching remains. Each pair has opposite edge directions. Facets on the
same ray cannot pair because they enclose no material there. Keep an
existing two-use body together. Check every shared edge again after all
links. Keep the slicer's closure and outward-volume checks, and the winding
check's existing fault classifications.

**Why.** A contact face can have no plain boundary edge. Plain-edge links
then leave its facets apart from the rest of its solid. A unique matching
can restore that link without choosing an arbitrary angular neighbour.
The later checks must still prove that the proposed links form valid
bodies. Larger or ambiguous fragment arrangements retain their refusal.

The feature graph keeps all incident facets until it checks the contact.
It then emits one facet pair for each valid solid. This gives face patches
and edge chains the same contact rule as slicing and winding. Store the
first two uses inline. Allocate extra storage only when an edge needs it.

One model can hold two closed solids. The former `stitch_cross` refusal
fixture contains two complete outward cubes with one shared face. It has
no extra bridge or other self-intersection. The test now requires the same
cut and verification result as the single-block reference. It also requires
that no depth level is skipped. This follows the expected-change rule.
The actual non-manifold refusal cases keep their assertions.

This branch started from the open PR #2904 at `853522f2e`. That prerequisite
merged as `bac4a96c` while this work was in progress.

**Dropped.** Do not pair arbitrary angular neighbours. Coincident walls
can make that choice ambiguous. Do not rely only on STEP solid ranges:
the facet-only winding and slicing APIs must handle the same contact.
Do not add a `SmallVec` dependency for two inline uses.

**Cost to change later.** Support for more fragmented edge uses needs a
unique matching rule and safety cases for closure, winding, fins and
line-only contact. Keep the final edge and volume checks. A larger inline
capacity can change locally without a project-file or public type change.

## #2845 - Refresh the Linux bore pin after the importer change

**Chosen.** Pin the Linux bore path to 3158 moves, 3130 planar feeds and
1141 arcs. Require the imported mesh to contain 6228 facets. Require all
four cutting levels: Z 0, 5, 10 and 15. Keep the clean verification,
fallback and planning-budget checks. Reject adjacent path counts in a
separate exact-pin test.

**Why.** PR #2904 replaced the 952-facet truck mesh with the solidcore
mesh at the same chord cap. Decision #917 records this mesh change. The
new mesh changes the cutting regions and fitted arcs. The fixture and
HSM planner did not change. The old Linux counts described the truck
mesh, while the review measured the solidcore path.

**Dropped.** Do not widen the count range or disable the heavy pin. Do
not weaken the arc proof or verification checks to match an old count.
Rejected arc fits retain their safe chords.

**Cost to change later.** A future mesh or planner change must explain
and measure the new counts. The mesh, cutting levels and verification
checks must still agree with that measurement.
