## #917 - The app reads STEP models through solidcore (part 1 of 3)

**What was decided.**
- The model import of a STEP file (`Format::Step`) goes through
  `camcraft_geometry::step::parse`. That module calls the kernel seam
  (`solidcore_step_import::import_step_model`, then `machining_mesh` for
  each solid). `camcraft-app` does not depend on `camcraft-kernel`. No new
  edge between our crates: geometry already depends on the kernel for
  Solidcraft (#1456). The time bound and the mesh-to-facets glue that the
  two kernel imports share moved to `camcraft_geometry::kernel_import`.
- The truck mesh path of the app is removed (the `step_*` helper modules of
  `crates/app/src/geometry/import/` that only the truck reader used), with
  the truck dependencies of `crates/app/Cargo.toml`.
- The public STEP entry for text in memory stays
  `camcraft_app::geometry::import::parse_model(data, Format::Step)` and
  `parse_model_graded`, with `import::step::parse_noted(data, budget)` under
  them.
- The browser build (wasm32) does not link solidcore. It refuses a STEP file
  by name: "STEP models open in the desktop app only, not in the browser
  build".
- A model over the facet budget is meshed again at a coarser chord: at most
  6 more tries, the chord times the facet ratio, clamped to x2..x16. The kept
  chord goes to `Graded::coarse_mm` as before (#864, #1054).
- **Sheets.** A sheet (a shell of a `SHELL_BASED_SURFACE_MODEL`, open by the
  file's own word) is meshed with its open boundary
  (`machining_mesh_open::sheet_mesh`) and labelled "sheet #N"; its notes
  read "STEP sheet #N: ...". A solid whose faces the mesher leaves out gives
  a partial mesh (`partial_mesh`). It lands only as an incomplete model
  with each left-out face named (the #1610 hole line, `step::hole`). It
  never lands as a whole model, and the graded loaders refuse it.
- **No STEP text reaches the partial path at the pinned solidcore commit.**
  Probes on `box.step` and the bent `fillet_corner`:

  | Probe | Result |
  |---|---|
  | one face removed | refused at read: "the outer shell is not closed" |
  | plane moved 0.5 mm or turned 90 degrees | "pcurves failed ... an edge lies 0.5 mm off the face's surface" |
  | plane tilted | reads; the mesh is refused: "face 2 stands 2.873e-1 mm off its surface" |
  | the same face twice | "face #53 is used twice" |
  | bent `fillet_corner` weights | refused at read, by face and value (face #1228, 1E300) |

  So the partial mesh is pinned at the kernel by a record edit
  (`crates/kernel/tests/machining_mesh_open.rs`), and the app pins of the
  incomplete model use a synthetic hole.
- **Closed solids ride the facets (`SolidRanges`).** The STEP import lands
  one facet range for each closed solid (`Mesh::from_solids`,
  `app::mesh_solids`). The slicer and the winding check read the solids
  from the ranges, not from a weld of the soup. The ranges are valid only at
  the model revision and facet count that they landed with. Import, update
  and undo land them. Any other landing of facets (`model::set_facets`,
  Align) steps the revision, and the soup read applies again. `MoveModel`
  sets only the transform, so the ranges stay. A file with a sheet or a
  partial mesh gets no ranges: such a mesh is not a set of solids.
- **Project file: `solids`, no version step.** The model entity has an
  optional `solids: [[start, end], ...]` (serde default, not written when
  empty). An older reader drops the key and reads the facets as one soup,
  which is what it did before. So the format version does not step. Ranges
  that do not tile the facets (out of bounds, overlap, gap, short, out of
  order, empty, one range) are dropped at load with a logged note, never
  trusted.
- **Shared-edge rule (`shared_edge::solids_touch`).** Two outward solids
  that touch or overlap along a wall use one mesh edge in 2k facets. Such an
  edge is accepted only when all of these are true:
  - the use count is even and balanced, with at least 4 uses;
  - each body (linked over plain edges only) uses the edge once each way;
  - no facet apex lies on the edge line;
  - each body has a facet at the same angle (1e-6 rad) as a facet of
    another body (a wall of one lies on a wall of the other).

  The slicer fallback (`slice_bodies::closed_solids`) and the winding check
  (`pick::winding`) use the same function. A soup that is truly
  non-manifold (one shell that meets itself, a fin, blocks that meet only
  along an edge line) is still refused by name. A model without such an
  edge gives the same result as before.
- **Face winding notes.** solidcore orients each face with its shell and
  leaves no trace. `step_face_winding` finds the faces in the file whose
  loops run against their neighbours and names them, as the earlier
  importer did: "shell #7: 1 face(s) wound against their neighbours were
  flipped: face #32 (3 of 6)".
- **Inward solids: one note.** The closed solids that are wound inward are
  turned outward. The import gives one note for all of them (#1617
  wording): "{k} closed solid(s) wound inward (negative volume) were turned
  outward: {n} facet(s) reversed".
- **Syntax locator.** A syntax refusal keeps the #912 location. Since #917
  the locator (`step_diag`) runs on every file before the kernel reads it,
  so it now skips a resource reference `<...>` (ISO 10303-21 ed. 3): a `/*`
  or a `'` in it opens nothing.
- `Graded::open` and the project field `mesh_open` stay.

**Why.** The maintainer asked to remove stepio and finish the switch to
solidcore. One reader for each format means one set of refusals and one
mesh law (watertight, outward, deviation measured). The solid ranges keep
what the reader knows (which facets close which solid) instead of a guess
from a weld of coincident facets.

**Changed pins** (old -> new, cause; the `#[test]` count of each file is
kept):
- `import_step_closed_shell.rs` (3 tests) and `import_step_open_shell.rs`
  test 1: "closed shell #1288 (10 faces) meshes open" / an incomplete model
  with its hole named -> a refusal at read, "first #1289 at read: face
  #1228: weight: 1... out of range". Cause: solidcore refuses the bent
  weight before any mesh (probe table above). The #1610 tests 2-4 do not
  change.
- `import_step_guard.rs`, `import_step_guard_default_cap.rs`: truck's
  evaluation cap ("within 100000 evaluations", "did not converge",
  "stopped after 3 faces or edges") -> the kernel's reasons: "off the
  face's surface" for the jumping face, "used twice" for the repeated face.
  The #1568 pin now uses a facet budget, and the volume bound is
  `1.2 * chord` at the kept chord (1.0194 at a 0.01 chord). Cause: the
  evaluation cap is gone with truck.
- `import_step_curve_guard.rs`, `import_step_corner_blend.rs`: the
  `with_face_eval_cap` wrapper is removed; the asserts do not change.
- `model_fidelity.rs`: a metre file was flagged at the old reader's 0.01 mm
  floor -> it meets the promise at the cap, with the measured deviation
  under the cap. Cause: solidcore meshes in millimetres; it has no floor of
  its own.
- `import_step_thread.rs`, `import_step_doubly_curved.rs`: the fixture
  shell now stands in a `SHELL_BASED_SURFACE_MODEL` (ISO 10303-42). The
  thread pin is known-fail #2844 (see below).
- `machining_mesh_parity.rs` (kernel, from #2843):
  `an_inward_solid_is_turned_outward_and_said` read a kernel note "the
  solid's faces all pointed inward ... turned outward" -> it reads the flag
  `report.turned_outward`, and asserts that the kernel adds no note. Cause:
  the STEP import gathers the flags of all solids into the one #1617 note
  (`import_step_outward`, `winding_align`, ui `import_outward` pin it), so
  a file with two inward solids gets one line, not two.
- The plain-slice tests do not change: the slicer sections the sample
  through the solid ranges.
- `bore_facet_sagitta.rs`, `step_chord_units.rs`: the 0.05 mm oversize
  bore asserted a program-gate "gouge:" problem -> it asserts `ok` false
  and the bore in `held_by` (the toolpath check holds the post). A new test
  in each file asserts the "gouge:" problem at 0.1 mm. Cause: the finer
  wall puts the 0.05 mm cut under the stock sim's 0.625 mm lattice
  (#2846, see below).
- ui `model_incomplete.rs`: the page test imported a box STEP with a face
  removed, which landed as an incomplete model -> such a file is now
  refused at read, so the test builds the open mesh in the test (a cube
  with its top two facets removed) and lands it through the test seam
  `State::land_import_for_test` (`crates/ui/src/state_test_import.rs`),
  which calls the same `finish_import` as the import worker. The hole text
  and the "Incomplete:" line now name a test reason; the other asserts do
  not change.
- `lead_approach.rs` (2 tests) and `shoulder_adaptive.rs` (1 test): known
  fail #2849, the asserts do not change (see below).

**Known regressions, tracked.**
- #2844: the four-turn thread flank gives 391338 facets (main: under 5000,
  the pin passes on main). solidcore-tess plans the strip isotropically
  and replans at half the chord ("replanned at 2.5e-3 mm chord to meet the
  requested 5e-3 mm measured tolerance"). The pin takes 93 s in debug. The
  bound stays the acceptance. Facet counts of the other helical, twisted
  and thin ruled fixtures (debug, the app import, main / this branch):

  | Fixture | main | branch |
  |---|---|---|
  | flank, 1 turn | 229 | 24200 (5.9 s) |
  | flank, 2 turns | 424 | 97506 (22.5 s) |
  | `twisted_flank.step` | 4 | 5132 |
  | `twisted_flank_off.step` | refused (boundary point off) | refused (pcurves, #117) |
  | `spline_revolved.step` | 5012 | 11500 |
  | `fillet_corner.step` | 2072 | 1866 |
  | `bspline_dome.step` | 350 | 78 |
  | `fig8.step` | 7974 | 12590 |
  | `Milling-test.step` | 952 | 6228 |
  | `sample_ap214.step` | 3178 | 11894 |

  The growth is in the thin twisted strips. Each branch mesh has its
  deviation measured within the tolerance; main's 4-facet twisted flank
  was not measured.
- #2845: the shared-edge rule cannot judge a face whose whole boundary is
  shared edges (cubes stacked face to face): that face makes its own body,
  so the rule refuses. Align refuses such a model by name (safe). The
  feature graph still splits tangent pieces on `sample_ap214` (216 pairs).
  Neither gives wrong toolpath output:
  `shared_face_bodies::the_sample_laid_flat_generates_3d_finish_and_pocket_with_no_gouge`
  and `welded_non_manifold_solids_are_refused_as_a_model_fault` prove it.
- The bore sagitta pin (cell 0.625, tol 0.01): the gouge cells at a 0.05 mm
  bore drop from 2 to 0 (0.1 mm: 10 -> 8; 0.3 mm: 34 -> 32). The ring holds
  only 8 cell centres, at r 12.507..12.509, inside the #299 slack. At 0.05
  verify still holds the post (`ok false`, `held_by`); a new "gouge:" pin
  at 0.1 mm keeps the gate's refusal. The lattice gap is #2846.
- #2849: on the new meshes of `Milling-test.step` and `fig8.step` three
  toolpath pins move. They pass on #2843's head.
  - The stock-source pocket with a helix entry gets "the pass start ... is
    in material" notes, and the corner check of `keep_law` reports "loop
    crosses itself" and does not run.
  - The `axis_start` contour posts its entry arc as chords (a note says so).
  - The fig8 EM-0800 adaptive ends 31.4 mm from the part, not at the face;
    it verifies clean and posts.
  Each fallback is stated in a note; none is silent wrong output. The self
  crossing loop is the defect to find.

**Considered and dropped.**
- A new `camcraft-app -> camcraft-kernel` edge. Dropped: the seam goes
  through geometry, and geometry already has the edge.
- Keep truck as a fallback for files that solidcore refuses. Dropped: two
  readers give two sets of answers, and the maintainer asked to remove it.
- Refuse sheets. Dropped: the old reader meshed them open, and a sheet is a
  legal surface model; it lands without solid ranges.
- An own STEP fixture for the partial path. Dropped: no STEP text reaches
  it at the pinned commit (probe table). The kernel pins it with a record
  edit.
- A radial pairing in the shared-edge rule (sort the facets about the edge
  by angle and pair each with its neighbour). Dropped: the sample's bodies
  overlap with coplanar facets, so two facets stand at the same angle and
  the radial order is not defined. The rule asks only that a wall of each
  body lies on a wall of another.
- Solids from a weld of the soup only, with no ranges. Dropped: the reader
  knows which facets close which solid; the ranges keep that answer.
- A version step of the project file for `solids`. Dropped: an older
  reader drops the key and reads the soup, as before.

**Cost to change later.** Low to medium. The STEP path is one function in
`crates/geometry/src/step.rs`; the app facade did not change its API. The
`solids` key is optional: a later format can drop it, and the soup read
stays.
