## #2361 - Map the rotated WCS suite to its app entry points

**What was decided:** Add a Gate map to `wcs_rotation_control.rs` for
`app::commands`, `app::operation`, `app::posting`, and `app::stock_sim`.
Add a selection test for all four entry points in `xtask`.

**Why:** The suite uses app commands, operation generation, posting, and
stock simulation. The quick gate must select the suite when these modules
change. The missing-map law must also pass.

**Considered and dropped:** Do not weaken the missing-map law. It detects
suites that the quick gate cannot select. Do not move this suite to another
crate. It needs the app commands and the app world.

**Cost to change later:** Update the suite header and its selection test
if the suite starts to cover different entry points.
