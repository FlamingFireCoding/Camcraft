## #1456 - Open Solidcraft `.sc` designs as a model (app wiring)

**What was decided.** A `.sc` file is one more model format of the one
model import (`geometry::import`): `Format::Solidcraft`, read from the
`sc` extension. The domain module, `camcraft_geometry::solidcraft`, calls
`camcraft_kernel::solidcraft::import_solidcraft` (the cached final body)
and `camcraft_kernel::machining_mesh::machining_mesh` at the STEP chord
cap. App passes that cap (`step::CHORD_CAP`, 0.005 mm) as an argument.
The parser turns the welded mesh into the model's facets. Model > Open,
the viewport drop and MCP `import_model` all reach that one import, so the three take `.sc` with no other change. Each
kernel refusal (reader or mesh) becomes `ImportError::Solidcraft` with the
kernel's own text: "Solidcraft design refused: <reason>". A STEP facet
budget given for a `.sc` is refused by name, as for STL and OBJ.

**The new crate edge: camcraft-geometry -> camcraft-kernel.** The review
requires the parser and facet conversion in the geometry crate. The app
keeps the dispatch and command wiring. It maps the domain error to
`ImportError`. The app no longer depends directly on kernel. Imported items:
1. `camcraft_kernel::solidcraft::import_solidcraft`
2. `camcraft_kernel::solidcraft::SolidcraftImport` (its `solid` and `design`)
3. `camcraft_kernel::solidcore_impl::SolidcoreKernel`
4. `camcraft_kernel::machining_mesh::machining_mesh`
5. `camcraft_kernel::machining_mesh::MachiningMesh` (its `positions`,
   `triangles` and `report.notes`)
6. `camcraft_kernel::CancelToken` (a new re-export of solidcore's token, so
   geometry needs no solidcore dependency of its own)
7. `camcraft_kernel::KernelError` and `machining_mesh::MeshRefusal`
   (through their `Display` text only)

Why the caller cannot pass the value in: geometry reads the user's design
bytes and returns its mesh. A recorded mesh cannot represent a new user
file. Passing a closure from app would retain the parser in the document
layer, which the review rejects.

What rebuilds when kernel changes: `camcraft-geometry` and its consumers,
including `camcraft-app`, `camcraft-ui`, the CLI, MCP, and their tests.
The first native build also compiles solidcore and its tess, record, step
and convert crates (pure Rust, already in `Cargo.lock`).

**Native only.** The edge is under `cfg(not(target_arch = "wasm32"))`.
The browser build keeps solidcore out of its wasm: a `.sc` there is
refused by name, and its picker does not offer `.sc`. Reasons: the
reader's panic guard (`catch_unwind`) does not work under wasm's
`panic=abort`, and no wasm build of solidcore was tried on this machine.

**Cooperative cancellation.** A watchdog thread cancels the kernel's token
after 300 seconds. The sample takes about 100 seconds in a debug build and
under a minute in release. The watchdog ends when the import ends.
The kernel polls the token between its stages. Its planner has no cancellation hook. This does not guarantee that
the import ends within 300 seconds. The size bounds of the kernel reader
(256 MiB) hold in all cases.

**Considered and dropped.**
- Stacking on #1738 (WCS flips): dropped by the user ("Option 2"). #1738
  changes no import function; this PR touches `mcp/tools.rs` and
  `ui/state.rs` in other functions only.
- A separate MCP tool `import_solidcraft`: dropped. `import_model` already
  dispatches on the extension, so an agent uses the tool it knows.
- Keeping the domain parser and conversion in app: rejected by the review.
  They use domain values and belong in geometry. Putting the conversion in
  kernel would add the reverse dependency on geometry.
- A `.sc` mesh made coarser than the cap when the mesh refuses: dropped. A
  refusal is shown by name (Prime Directive); no fallback chord.

**Cost to change later.** When the STEP import moves to solidcore (#917),
the same edge carries it; the native-only gate can be lifted once solidcore
builds and is tested for wasm.
