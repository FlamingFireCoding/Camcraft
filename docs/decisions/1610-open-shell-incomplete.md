## #1610 - a STEP closed shell that meshes open loads incomplete

**Decided.** The model import (`ImportModel`, so the GUI, MCP and the
script API) lands a STEP closed shell that meshes open as a model instead
of refusing it. The model carries the fidelity-unknown sentinel
(`model_fidelity::UNKNOWN_MM`, #1056) and, beside it, its hole in words
(`schema::MeshOpen`, saved as the entity key `mesh_open`). Every operation
judged against it, a fixture made from it and the program verdict are
refused at the post by the existing fidelity gate; the refusal, the model
page, the import status line and the MCP/script `reduced_fidelity` facts
name the hole ("imported incomplete - closed shell #N ... meshes open: ...")
instead of the pre-v59 note.

**Why.** The issue asked for exactly this reuse: the sentinel already rides
every path a chord rides (stock copies, setup clones, undo, save), so the
refusal cannot be lost on any of them; only the words needed a home.

**Considered and dropped.**
- A second chord sentinel for "open": every chord path would carry it, but
  the save writes an unknown chord as the `fidelity_unknown` flag, so the
  file would need a new key anyway, and the hole could still not be named.
- A project version bump for `mesh_open`: an older reader drops the key
  and keeps `fidelity_unknown`, so it still refuses every operation on the
  model; only the words are lost. Not the unsafe-old-reader case the
  version bumps exist for. A `mesh_open` without `fidelity_unknown` on a
  model is refused as corrupt on load.
- Landing incomplete everywhere: the fixture import, the tool-body import
  and the mesh-only loaders (`load_model`, `parse_model`,
  `*_graded_within`) still refuse the file by name - they have no
  incomplete state to land in. A shell the file does not declare closed
  with a dropped face, and a closed shell wound inconsistently, are still
  refused as before.

**Left as is.** A stock copied from an incomplete model carries the mark,
not the hole: its refusal names the source model and reads the source's
hole by name, so after the source is deleted the copy's refusal falls back
to the pre-v59 words (still refused). The real fix for the meshes stays
#1602.

**Cost to change later.** Small: `mesh_open` is one module, one component
and one optional file key.

### PR #1699 review follow-up

**Decided.** Pin the incomplete-model page and painted import status with
structural UI assertions. Capture the same missing-wall STEP fixture in
the real headless shell for review, with its placeholder viewport disclosed.
The before image temporarily restores the previous graded-loader entry
point and shows the actual import refusal; the after image shows the
current incomplete model page and status. Restore the loader after capture.

**Why.** This checks the displayed behavior and supplies reproducible visual
evidence without changing the release-only pixel goldens.

**Considered and dropped.** Updating pixel goldens would violate the UI test
policy. Reconstructing the previous page around an incomplete model would
misrepresent the prior loader, which refused this input entirely.
