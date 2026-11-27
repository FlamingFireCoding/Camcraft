## #3643 - Tool component bodies from STEP, STL and OBJ files, and MCS_INSERT (mesh-body slice)

**What was decided.**
- `MCS_INSERT` is classified as `CsysRole::Mounting` in
  `camcraft_geometry::step_csys` (the app's `step_csys.rs` re-exports
  that type, so the app reads it too). Only the exact name (case and
  blanks ignored) is taken; `MCS_1` or `MCS_INSERT_2` stay `Other`. A
  file that names both `MCS` and `MCS_INSERT` has two mounting frames,
  and `ToolCsysReport::find` refuses it as ambiguous. It is not guessed
  which one wins.
- A new `camcraft_tools::mesh_body::MeshBody` reads one component body:
  - STEP through the existing kernel import
    (`camcraft_geometry::step::parse`) at a 0.01 mm chord, or coarser
    inside the facet budget, with its named coordinate systems from
    `camcraft_geometry::step_csys`;
  - STL (ASCII and binary) and OBJ through a new small reader in
    `camcraft_tools::mesh_file`, with frames that the caller stores
    (`MeshBody::with_frames`), classified by their ISO 13399 names.
- The component frame is the body's one mounting frame (MCS or
  MCS_INSERT), as `ComponentGeometry` defines it: origin on the machine
  side, +Z toward the machine. Each workpiece-side port (CSW, CSWx,
  CSWx_y) and the cutting point (CIP) is read in that frame. A port or
  cutting point that is not below the mounting frame is refused by
  name (`NotBelowMount`): this catches a file whose MCS +Z points away
  from the machine, which would otherwise place the body upside down.
- A spindle body (`MeshBody::spindle_geometry`) is the revolved
  envelope of every facet (`HolderProfile::envelope`), drawn as a
  stepped `SpunProfile` and stored as a `ComponentBody`. The drawn
  outline only grows the envelope:
  - each join of two bands takes the larger radius;
  - bands closer than 0.02 mm in radius, or shorter than 0.02 mm, are
    joined, so no drawn segment is shorter than `MIN_SEGMENT`;
  - a band with no facet (a gap along the axis) gets 0.02 mm, not 0;
  - at most 31 bands remain, so the outline fits `MAX_POINTS`;
  - a last check refuses the body (`Outline`) if any envelope band is
    not inside the drawn outline over its whole length.
- A lathe body (`MeshBody::lathe_geometry`) is the facets as a
  `LatheBody`, as #4194 builds the lathe envelope: all bodies of the
  stack go to `LatheClearance::of` as one set.
- Never smaller than the steel: a meshed STEP body grows by the
  kernel's certified deviation (`StepMesh::deviation_mm`).
  - Spindle: each band radius and both ends grow by it.
  - Lathe: `LatheBody::padded` records it, and `resolve_lathe` grows
    the clearance bands by the largest pad of the stack
    (`LatheClearance::padded`, a Minkowski grow by a box, proof in the
    module comment of `lathe_clearance_pad.rs`).
  - An STL or OBJ body has no deviation: the file is the body.
- Facet budget: one component body has at most `MAX_BODY_FACETS` =
  1 000 000 facets, and a file at most 512 MB. Past the budget the body
  is refused by name, with #1041 named in the text. A STEP model is
  first meshed coarser by the existing importer (up to its six tries);
  only when no chord fits is it refused (`StepOverBudget`). Automatic
  coarsening of an STL or OBJ (#1041) is not done here.
- Every refusal has a name (`MeshBodyError`, `MeshFileError`). No path
  panics: the readers use checked lengths, `get`, and `try_from`. A
  fuzz-style test cuts short and flips bits in STL, OBJ and STEP files.

**Why.**
- The brief asks for the body in the crate that owns component
  geometry (`camcraft-tools`). That crate already depends on
  `camcraft-geometry`, which holds the STEP mesher and the coordinate
  system reader, so no new crate edge is needed.
- The app's STL and OBJ importers are in `crates/app/src/geometry/import`
  and return the app's `ImportError`, so `camcraft-tools` cannot call
  them. A small reader in the tools crate was chosen over a move of the
  app importers, because the move touches the model import path and its
  tests, which this slice must not change.
- A revolved outline is what the spindle laws read today
  (`ComponentBody`, `HolderProfile`), so the mesh enters the same
  judged path as a parametric holder, and the existing union of bodies
  in `tool_assembly_body::judged` works with no change.
- The deviation is a measured bound, so the grow is the smallest grow
  that keeps "never smaller than the steel" true for a curved surface,
  where the facets lie inside the surface.

**Considered and dropped.**
- A mesh variant of `ComponentBody` that the spindle laws read
  directly: dropped. It changes `judged` and the collision laws, and
  the revolved envelope is already how an imported holder mesh is
  judged today.
- Moving the app's STL and OBJ readers to `camcraft-geometry`: kept for
  later. A follow-up can make the app import call
  `camcraft_tools::mesh_file` or a shared reader.
- Refusing a pad larger than half a lathe band: dropped. The grow
  takes the union of as many neighbour bands as the pad needs.
- Taking `MCS` over `MCS_INSERT` when a file names both: dropped. The
  two are the same role, and a guess can place an insert wrongly.

**What it costs to change later.**
- The budget and the chord are constants in `mesh_file.rs` and
  `mesh_body.rs`. #1041 can replace the refusal with a coarser mesh.
- The app wiring is not in this slice: a document field for the mesh
  file and its stored frames, the commands and MCP tools that set it,
  and the call to `MeshBody::spindle_geometry` / `lathe_geometry` in
  `crates/app/src/tool_assembly*.rs` (open PRs #4249 and #4278 change
  those files now).
- The near-tip check of `resolve_lathe` (`BodyPastEdge`) reads the
  facets without the pad. It finds a wrong hand or seat frame, and it
  is not a clearance law.
- The boring-bar overhang in a sleeve is not in this slice.
