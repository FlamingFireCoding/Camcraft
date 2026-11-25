## #3831 - One model section for each prismatic Z slab of a ramped path

**What was decided.** The verifier sections the model one time for each
prismatic Z slab, not one time for each plane that a ramp move ends at.
A prismatic slab is the Z range between two neighbouring vertex levels in
which each facet that a plane crosses is vertical (two of its vertices have
bit-equal XY). Three changes go together:

1. `Rest::island_slice` gives the planes of a prismatic slab one shared
   section (`verify/slab.rs`). A plane that the path cuts flat at keeps
   its own section. A plane outside a prismatic slab keeps its own section.
2. The reach law (c) samples a shared slab section along its walls each
   0.25 mm (`slab::wall_samples`), in addition to the corners and the edge
   midpoints.
3. The gouge law (b) asks the cheap distance test before the wall probe
   (`edge_is_wall`). The two tests are pure, so the order changes no result.

**Why.** A keep-down ramp ends each move at a Z of its own. The measured
cost (release build, 38,432 facets, 16 holes, one ring, 2,422 moves) was
282 s. It had three parts:

- 2,406 model sections, each a walk over all facets (43 s).
- A wall probe for each region edge of each move, each a walk over the
  section (40 s). This part grows with the rings times the moves.
- 61.5 million reach samples (about 1 GB), because each plane added its
  own samples of the same walls (198 s).

After the change the same case takes 1.2 s, with the same 2,422 moves and
the same 326 warnings. Four rings take 5.1 s.

**Why the section is exact.** No vertex is strictly inside a slab, so each
plane of the slab crosses the same facets. If all those facets are
vertical, the solid between the two levels is a prism. Each plane of a
prism is the same point set. The test for "vertical" uses no arithmetic,
so rounding cannot call a sloped facet vertical. A plane nearer than
4e-6 mm to the top level of its slab is not placed, because the slicer
probes up to 3e-6 mm above the plane. A shared section that fails to slice
is not kept: each plane then asks for its own section, as before.

**Why the wall samples.** This part is a decision, not a proof. Before the
change, each plane put a vertex where the plane met the diagonal of the two
facets of a wall. That vertex moves from one end of the wall to the other
as the plane rises. Law (c) reads no sample's plane, so a ramp with 500
planes read each wall at about 500 points. One section alone has only the
corners, the diagonal point and the midpoints. On the test fixture
(a ramp 1 mm into a boss wall) that gave 61 `ShankHit` findings where the
old reading gave 80. Samples each 0.25 mm along the walls give the same 80
findings, in the same order, on each fixture that I compared (500 and
2,000 moves, three offsets, two modes, a stepped part). 0.25 mm is the
sample step that the solid law already uses.

The new reading is not bit-identical to the old reading for a ramped path:
the sample positions are different. It can convict a move that the old
reading missed between two of its samples. I found no case where it
convicts fewer moves, but the tests are fixtures, not a proof.

**What does not change.** A path that only cuts flat (a level ladder)
reads the same sections and the same samples as before, bit for bit. A
model with a sloped or curved wall in the slab is sectioned at each plane
as before. The moves and the posted program do not change: the verifier
only reads the path.

**Considered and dropped.**

- Only exact changes (the probe order, a memo of the column surface). They
  give about 3 times, not 200 times, and the 1 GB sample set stays.
- One section for each slab with the old corner and midpoint samples only.
  It lost 19 of 80 findings on the fixture. A collision law must not get
  weaker for speed.
- One section for each slab for flat planes also. It would change the
  samples of each level ladder in each existing project. The flat planes
  are few, so the cost of their own sections is small.
- A lower move count for the keep-down ramp (the other option in the
  issue). That changes posted G-code and is a different decision.
- A time limit in a test. The pins count sections and wall probes, which
  do not depend on the machine.

**Cost to change later.** Small. `Slabs::of` is the one gate: if it
returns `None`, each plane is sectioned on its own again. The test helper
`Rest::plane_exact` keeps that old reading available as a reference.
