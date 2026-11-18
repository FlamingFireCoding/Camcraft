## #2496 - A hole wider than its drill is named, and the post is not held

- **Decided:** a Drill op names each Z hole wider than its Drill-kind tool by more than `DRILL_SIZE_TOL` (0.1 mm on the diameter). The names show in two places:
  - Detect holes keeps the hole and names it in `DetectedHoles::undersize` (MCP `undersize`, the GUI status line).
  - Generate adds one `Warning::DrillUndersize` for each such hole, at its first cutting move (MCP finding kind `drill_undersize`).
  - **Why:** the drill makes its own diameter, so such a hole comes out undersize. No other law saw it: the containment law judges a cut into the part, never material left in a hole.
- **Decided:** the warning is advisory, like `Scallop`, `ProbeSearch` and `SinkerFinishOrder`. It does not hold the post.
  - **Why:** a pilot drill is a valid plan (the maintainer's triage on the issue: "a named warning that the user can accept is better than a silent skip"). The part size is wrong but nothing gouges or collides, so the user decides.
  - **Dropped:** skipping the hole at Detect holes, as Bore does (#1556). That makes a pilot drill impossible from Detect holes.
  - **Dropped:** a refusal at generate. The same reason.
- **Decided:** the law reads only a Drill-kind tool, in a cycle that drills to size: Auto, Drill, Peck and Ream. It reads only the mill's Z holes.
  - **Why:** a spot dimple, a tap (its hole is the tap drill's, by design) and a bore cycle (sized by the slide) make a hole of another size. Other tool kinds in a Drill op (a chamfer mill, a boring bar) are not drills.
  - **Not changed:** the turret's radial holes. The issue is a mill case; a radial follow-up can use the same `undersize_holes`.
- **Decided:** `DRILL_SIZE_TOL` is 0.1 mm on the diameter, with a float slack of 1e-6 mm.
  - **Why:** a twist drill cuts up to about a tenth over its size. The project's own matched fixtures model a hole that much over its drill: `drilling_axial` has a 4 mm drill in a 4.1 mm hole, and calls it "a matched axial drill".
  - **Dropped:** 0.05 mm (the first try). It named the 4.1 mm hole of that matched fixture, and the quick gate failed.
- **Not changed:** a hole narrower than the drill. That is a cut into the wall, and the solid law holds the post with a `Gouge` (pinned by `a_hole_narrower_than_the_drill_is_held_as_a_gouge_not_named_undersize`). The hole-wall excuse is capped at the facet sagitta, so it does not hide that cut.
- **Note:** the GUI's `CommandKind::DetectedHoles` field `skipped` is renamed `notes`. It carries the skipped and the undersize sentences, which the one status line shows.
