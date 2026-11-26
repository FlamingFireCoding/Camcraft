## #4107 - A marching crossing within 1e-6 of a sample sits on it

**Decided.** In `iso.rs` `tet`, a crossing whose edge fraction `t` is below `NODE_SNAP` (1e-6) or above `1 - NODE_SNAP` is put on the sample (`t` = 0 or 1). The facets that then share that sample with two corners are exactly zero-area, and `push` already drops them.

**Why.** A cut floor on the extraction lattice (the field's cell centres) leaves field values of about 1e-12 at the samples. Marching tetrahedra put the crossing picometres from the sample, under the 1e-9 weld that `open_edges` uses and under f32 precision, so the facets that shared it were kept by `push` and collapsed after a weld or an f32 export: 11,394 collapsed facets and 5,799 edges on three or more facets in the pin's block. At a 0.25 mm step the snap moves a corner at most 0.25 nm. Dual contouring is not affected.

**Considered and dropped.**
- Treating a value within an epsilon of zero as zero before the crossing: the same effect, but it changes which side of the surface a sample is on, so it also changes which tetrahedra are cut.
- Dropping facets whose corners meet after a weld, after extraction: it repairs the output and leaves the cause; the weld tolerance would become a second law.
- Making `open_edges` count every edge in the same PR: it is used as a closed-mesh check by several tests, and a stricter count may be wanted separately. The pin counts every edge itself.

**Cost to change later.** One constant and one line in `tet`. Going back is deleting the line; the pins in `crates/sim/tests/iso_node_floor.rs` name the behaviour.
