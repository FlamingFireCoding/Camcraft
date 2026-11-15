## #1586 - MCP orientation tools

**Decided:** Expose `align_part_cylinder` and `place_setup_origin` in the
existing MCP alignment tool table. Cylinder alignment calls
`AlignPartToSetup::align_from_cylinder`. Origin placement resolves through
`commands::place_setup_origin` and executes `SetWcsOrigin::for_setup`.
Each successful call adds one undo step and defaults to the active setup.

**Why:** These are the GUI's existing command paths. Reusing them keeps
the cylinder's target-axis check, setup ownership, stock handling and undo
behaviour consistent across the GUI and MCP. Origin placement preserves
the WCS rotation. An explicit unchanged-origin refusal prevents a useless
history entry because `SetWcsOrigin::for_setup` itself accepts that case.

**Considered and dropped:** Duplicating the cylinder fit or origin math
in MCP would give safety checks two implementations. Direct ECS writes
would bypass undo. Silently ignoring a vertex source's `point` or a box
source's `pick` would accept conflicting requests, so source-specific
argument guards refuse them by name. New tool modules were unnecessary;
the existing alignment module remains below 400 lines.

**Cost to change later:** A different public argument shape requires
updating the MCP schemas, handlers, headless tests and skill rows together.
The shared commands and saved project format do not need to change.

**Review fix - named vertex origin:** Resolve the vertex through the
requested setup's pose in the shared origin command. Keep the existing
active-view vertex reader for viewport callers. Switching the active
setup temporarily was rejected because reading a named setup must not
change session state. The MCP pin compares active and inactive requests,
checks the known aligned coordinate, and checks undo and Setup2 isolation.
