## #917 - The app reads STEP models through solidcore (part 1 of 3)

**What was decided.**
- The model import of a STEP file (`Format::Step`) goes through
  `camcraft_geometry::step::parse`. That module calls the kernel seam
  (`solidcore_step::import_step`, then `machining_mesh` for each solid).
  `camcraft-app` does not depend on `camcraft-kernel`. No new edge between
  our crates: geometry already depends on the kernel for Solidcraft (#1456).
- The kernel crates move to the solidcore pin of #2781.
- The truck mesh path of the app is removed (the `step_*` helper modules of
  `crates/app/src/geometry/import/` that only the truck reader used). The
  truck dependencies of `crates/app/Cargo.toml` are removed.
- The public STEP entry for text in memory stays
  `camcraft_app::geometry::import::parse_model(data, Format::Step)` and
  `parse_model_graded`, with `import::step::parse_noted(data, budget)` under
  them. A caller that has STEP text (PR #2707) calls these.
- The browser build (wasm32) does not link solidcore. It refuses a STEP file
  by name: "STEP models open in the desktop app only, not in the browser
  build".
- A model over the facet budget is meshed again at a coarser chord: at most
  6 more tries, the chord times the facet ratio, clamped to x2..x16. The kept
  chord goes to `Graded::coarse_mm` as before (#864, #1054).
- A syntax refusal keeps the #912 location: the geometry module adds the
  entity id and its first line, and the string or comment that never
  closes, to the kernel's "syntax error at line N".
- `Graded::open` stays, and the project field `mesh_open` stays. The STEP
  importer does not set it now: solidcore refuses a solid that does not mesh
  closed. A saved project that recorded a hole keeps it.
- A sheet (an open shell, a surface model, a lone face) is refused by name
  ("a surface model of open shells, not a closed solid"). Before, truck
  meshed some sheets open.
- The kernel bench (`crates/kernel/tests/step_agreement.rs` and the truck
  comparison tests) stays in this PR. Part 3 drops the truck side of the
  bench and removes `truck-stepio` and `ruststep` from the workspace. Part 2
  ports the tool coordinate system, unit and depth readers (`step_csys`,
  `step_units`, `step_depth`) to the solidcore-step p21 parser and removes
  `ruststep` from the app.

**Why.** The maintainer asked to remove stepio and finish the switch to
solidcore. One reader for each format means one set of refusals and one
mesh law (watertight, outward, deviation measured). A split in three PRs
keeps each review small.

**Test verdicts that changed** (the `#[test]` count of each file is kept):
- `import_step_guard.rs`, `import_step_guard_default_cap.rs`: truck's
  per-face evaluation cap ("within 100000 evaluations", "did not converge",
  "stopped after 3 faces or edges") and the #1568 finer-pass fallback are
  gone with truck. The pins now assert: bounded time and a named verdict;
  the dome meshes at the chord cap; a small budget keeps a coarser mesh and
  reports its chord; a shell of 41 repeated faces is refused.
- `import_step_curve_guard.rs`: truck's "edge ... subdivision" and "swept
  ... subdivision" words are gone. The pins assert a refusal that names the
  entity (`#`).
- `import_step_closed_shell.rs`: "closed shell #1288 (10 faces) meshes
  open", "got no mesh from the kernel" and the "flipped: face #32 (" note
  are truck words. The pins assert a closed mesh (no open edge) or a named
  refusal, and that the MCP result carries the same notes as the import.
- `import_step_open_shell.rs`: the bent closed shell lands whole or is
  refused by every loader with the same reason.
- `import_step_outward.rs`: the inward box imports outward; a lone curved
  face is refused as a sheet.
- `import_step_thread.rs`: the four-turn thread flank is a sheet, so it is
  refused by name. The #1602 measurement of the flank is lost here; a
  follow-up issue asks for a closed threaded solid fixture.
- `import_step_doubly_curved.rs`: the saddle sheet became a closed block
  with the saddle on top (volume 4000 mm3); the curvature measurements read
  the top facets.

**Considered and dropped.**
- A new `camcraft-app -> camcraft-kernel` edge. Dropped: the Issue Law asks
  for the seam through geometry, and geometry already has the edge.
- Keep truck as a fallback for files that solidcore refuses. Dropped: two
  readers give two sets of answers, and the maintainer asked to remove it.
- Mesh sheets open, as truck did. Dropped: a toolpath over a missing wall
  is a gouge; a named refusal is safe.

**Cost to change later.** Low. The STEP path is one function in
`crates/geometry/src/step.rs`; the app facade did not change its API.
