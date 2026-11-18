## #2543 - Repair the four red post-merge tests on main

**Decided.**
- `examples/first-router-part.nc`: recorded again from
  `camcraft-cli process examples/first-router-part.camproj`. The one changed
  line is the drill cycle depth, `Z-13` -> `Z-14.803`. #2504 saved the project
  at v68 with `"break_through_from": "full_diameter"`; the extra 1.803 mm is
  the point of the Ø6 118° drill, `3 / tan(59°)`. The pin of this program came
  from #2488 at the same time, so neither PR saw the other's change.
- `crates/app/tests/golden/nc/thread_mill_m8_multi_form.nc` and
  `thread_mill_external_m8_multi_form.nc`: recorded again. Only the tool
  comment, `T` and `H` words change. #2488 numbers each tool at `AddTool` in
  the order of addition; the fixtures add the thread mill first, so it is
  `T01`. Each operation still calls its own tool with a matching `H` word.
- `crates/ui/src/egui_app/tests/guide_projects.rs`: the demo projects go to
  a directory of the test process (`camcraft-guide-projects-<pid>`), which
  the test removes. `CAMCRAFT_GUIDE_PROJECTS_OUT` still selects the output
  for `cargo xtask docs-images` and the catalog media.

**Why.** The posted output of the first two items is correct by the rules of
the PRs that changed it, so the pins follow the output. The ui test failed
with "Permission denied" on `/tmp/camcraft-guide-projects`: a fixed name in a
shared temp directory belongs to the first account that makes it.

**Dropped.**
- Make the thread fixtures add the drill first to keep the old `T` numbers:
  it hides the new numbering rule from the goldens and changes fixtures that
  are not wrong.
- Remove `"break_through_from"` from the router example to keep `Z-13`: #2504
  chose the full-diameter break-through for new and re-saved projects.
- Use the `tempfile` crate for the ui test: a new dependency for one path.

**Cost to change.** Record the three programs again; one function in
`guide_projects.rs`.
