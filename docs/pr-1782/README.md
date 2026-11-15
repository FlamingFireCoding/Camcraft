# PR #1782 review pictures

These are captures of the actual egui shell and operation forms, with an actual
`Adaptive1` operation using Method = Trochoidal. The Passes captures select
Return = Microlift, height 0.4 mm. The Feeds captures show the same operation.

- `before/`: the `roughing2d_pane` rendering body from the original PR parent,
  `b0b443b211570ee54a10d753063a9698ff5b36aa`, in the merged shell. A temporary
  signature adapter extracts Method from the new grouped parameter argument;
  the old body and its widgets are otherwise unchanged.
- `after/`: the merged shipping pane, retaining main's Arc turns control and
  the PR's Trochoidal controls.

The native GPU capture failed with `No adapter found` in this environment.
A temporary CPU renderer rasterized the actual egui tessellated meshes and
font atlas instead. It omitted bitmap toolbar icons and the viewport texture;
all form controls and text come from the shipping egui code. The viewport in
this test harness is a placeholder. These are review evidence, not pixel
comparison goldens, and no release snapshot PNG was changed.

The temporary capture test, renderer and original-body adapter were removed
after rendering. The structural UI pin remains in
`crates/ui/src/egui_app/tests/adaptive_method.rs` and checks that Microlift
height appears only for explicit Microlift, alongside the Passes/Feeds fields.
