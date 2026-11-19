## #917 - The kernel makes the pcurves that a STEP file does not give

**What was decided.**
- The solidcore pin moves to the fork's commit
  `2016386442dfee3b2a0c3a1bfdd4017c5254dac8` (#2781), by the procedure of
  `916-solidcore-step-bench.md`. The record at that commit has a `Blend`
  surface. The mesh bound and the mesh deviation give no closed form for
  it (`None`), as for a spline, so the measured deviation decides.
- The kernel crate implements the solidcore `Trims` seam
  (`solidcore_ops::pcurve::Trims`) in `crates/kernel/src/solidcore_trims.rs`
  (`KernelTrims`). `read_step` now calls
  `solidcore_step::topology::read_solids_with(&exchange, &KernelTrims)`
  instead of `read_solids`.
- A face that has an edge without a closed-form pcurve (a free-form edge on
  a spline face, for example a twisted B-spline flank) goes to the solidcore
  kernel alone, as an open one-face sheet. The uses without a pcurve go in
  bare (`solidcore_convert::brep::to_kernel_with_bare_uses`). The kernel
  makes their UV trims as it makes each trim it needs, and
  `solidcore_convert::brep::to_record` gives them back.
- The route is the maintainer's Solidcraft `SmlibTrims`, reused with his
  permission and moved onto the public API at the pinned solidcore commit.
- The trims run on the same kernel state as the import (`SolidcoreKernel::k`),
  not on a fresh kernel. Its tolerances, opt-in fixes and cancel token then
  hold for the trims too.
- Nothing is taken unchecked. Each of these is refused by name ("kernel
  pcurves: ..."), and the reader then refuses the solid:
  - an edge that lies more than 1 mm off its surface (`MAX_EDGE_GAP_MM`, the
    reader's own `MAX_TOLERANCE`). A smaller gap raises the edge and vertex
    tolerances to `SURFACE_GAP_MARGIN` times the gap, as the reader does;
  - an edge interval that the kernel moved past the edge's reach (the
    larger of its vertex and edge tolerances);
  - a made pcurve whose lift onto the surface misses its 3D edge by more
    than the held edge tolerance (33 samples, ends included).
  The reader checks the whole record again after this
  (`record_check::agreement`).
- New dependency edge: `camcraft-kernel -> solidcore-ops`, at the same
  pinned commit as the other solidcore crates. It is a crate of the solidcore
  workspace and is already in `Cargo.lock` through `solidcore-step`. No new
  third-party crate. No new edge between our own crates.
- The bench (`crates/kernel/tests/step_agreement.rs`) pins the new
  verdicts. `crates/kernel/tests/step_twisted_flank.rs` pins #2780 at the
  kernel: `twisted_flank.step` reads closed at 8000 mm3; the variant with a
  vertex off its curves reads near that volume or is refused by name.

**Why.** At the pinned commit the reader refuses a face whose free-form
edges have no pcurve ("pcurves failed on ..."). Real files have such faces:
a user STEP file with a twisted B-spline flank was refused. The kernel
already makes trims for its own booleans and blends, so the same code makes
these. The maintainer chose this route (option 2) over the alternatives
below.

**Considered and dropped.**
- Keep truck as a fallback for the faces that solidcore refuses. Dropped:
  two readers give two sets of answers, and the maintainer asked to remove
  truck.
- Our own pcurve fit (project 3D samples onto the surface, fit a 2D
  B-spline). Dropped: it is a second trim maker beside the kernel's, with
  its own tolerances to keep in step.
- Wait for the reader upstream to make the pcurves. Dropped: there is no
  date, and the user file stays refused until then.
- A fresh kernel for each face. Dropped: it loses the opt-in fixes and the
  cancel token of the import.

**Cost to change later.** Low. The seam is one trait. If the reader later
makes the pcurves itself, `read_step` calls `read_solids` again and
`solidcore_trims.rs` is deleted; the bench pins show any change of verdict.
