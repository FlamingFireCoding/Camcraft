## #3475 - A tool wider than its slot is refused at the command

**What was decided.**

- Slot: the generator's fit rule (`h = width / 2 - r - leave`, with
  `h = 0` permitted) moved into one pure function,
  `camcraft_motion::machining2d::slot_centre_band`. The slot generator
  and the commands call it. There is no second copy of the rule.
- `tool_gate` (the tool law's door in `commands/operation/gates.rs`) now
  ends with `slot_gate`. Thus `CreateOperation` and `EditOperationLinks`
  (MCP `set_tool`, the UI's links edit) refuse a plain Slot whose tool is
  too wide. The text names both sizes and the largest tool that fits, in
  the document unit: "The tool 'EM-WIDE' is 10 mm wide. The slot is 6 mm
  wide. Choose a tool of 6 mm or less." The UI shows the same text after
  its "links not updated: " prefix. It is one command path.
- The second door: `ChangeTool` refuses an edit that makes a tool wider
  when a plain Slot that uses the tool can then not hold it
  ("S1 uses it: The tool ..."). It reads the operations that name the tool,
  as `retype_refusal` does.
- The generator keeps its own refusal. A project file from an older build
  can still hold the bad pair.
- `slot_dxf_sketch::picks_that_are_no_slot_are_refused_by_name`: the plain
  Slot on the 3 mm outline under the 4 mm tool was accepted at create and
  refused at generate. Now the create refuses it. The trochoidal slot is
  not changed.

**Why.** AGENTS.md "Validate early". The slot rule is a function of three
numbers that the command has, so the check is exact. It cannot refuse a
slot that the generator cuts.

**Dropped.**

- 2D Pocket. Its fit depends on the region: the containment (an open
  side lets a wider tool cut from outside), the stock and the part-sliced
  region. The command does not have that region without the part slice
  and the area controls. A bounding-box or width estimate can refuse legal
  work, so none was added. A warning was not added, because it would need
  that same region to be exact. A `source:agent` follow-up issue holds the
  pocket work.
- A refusal in `EditOperationParams` when the slot width is edited below
  the tool. It was not in this issue's scope (the tool assignment); it
  goes in the same follow-up issue.
- Trochoidal Slot: its fit has a different rule (the trochoid needs a tool
  smaller than the width) and its generator already names it.

**Cost to change later.** Small: `slot_gate` and `resize_refusal` are one
call each; a pocket check can join `tool_gate` the same way.
