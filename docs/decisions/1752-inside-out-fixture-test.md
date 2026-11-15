## #1752 - The inside-out STL fixture test follows the #1672 import turn

Chosen: change the test, not the importer. The test
`the_inside_out_stl_fixture_reads_inside_out_and_refuses` becomes
`the_inside_out_stl_fixture_is_turned_outward_at_import_and_lays`. It pins
that the import of `testdata/models/cube_inside_out.stl` turns the one closed
body outward, notes the turn (12 facets reversed), and that the top face then
lays like the outward block.

Why: PR #1686 (#1672) turns a closed, isolated inward STL/OBJ body outward at
import on purpose, with a decision entry and its own pins. The orientation of
a closed, consistently wound body with no overlapping neighbour is not
ambiguous, so the turn loosens no refusal. The old test still expected the
pre-#1686 refusal. The failure is deterministic (5 of 5 local runs failed on
origin/main), not a flake.

Kept: the refusal of an inside-out body that reaches the world unturned
(`an_inside_out_cube_refuses_*`, `bodies_are_judged_one_by_one`, which import
through `ImportModel::from_mesh`), and the import refusal of a nested inward
shell that cannot be turned safely (`import_outward.rs`).

Dropped: reverting the import turn (it is the #1672 fix, and the user-facing
behaviour it gives is correct); deleting the test (the fixture is the GUI
repro, and it now pins the import turn end to end through `load_model`).
