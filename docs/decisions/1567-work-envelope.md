## #1567 - Show the machine work envelope in the viewport

**Context:** the maintainer asked for a Browser checkbox that draws the
mounted machine's work envelope (issue 1567).

**Decided and why:**
- The envelope is the tool-tip travel volume in program coordinates, put in
  the world at the setup's work origin. It uses the same travel figures,
  axis directions and origin law (`posting::wcs_context`) as the program
  stroke check. The picture cannot disagree with a refusal. The tooltip says
  "measured from the setup's work origin (not the table)".
- No table position exists in the machine data, and a stroke is stated in
  program coordinates (`machine/mod.rs`, #457). So the envelope is not
  "placed" on a floor under the fixtures. The suggested fallback (floor at
  the lowest stock point, centred) would invent a number, so it was dropped.
  The part and the fixtures stand where the work origin puts them.
- Mill, mill-turn, plasma and EDM: a box of the X, Y and Z travel. Rotary
  reach is not drawn (first slice, as the issue says).
- Lathe: a cylinder on the spindle axis through the work origin. The radius
  is the larger end of the X travel (X is a radius on a lathe, as the lathe
  stroke check reads it). The length is the Z travel.
- A missing axis or travel draws nothing. The tooltip names the missing
  figure. The box is disabled only when no machine is mounted.
- The box is immediate-mode view state (`State::show_work_envelope`), like
  "Show world origin axes": no command, not saved, not in history. No MCP
  tool is needed for display state.

**Considered and dropped:**
- Faint faces: the overlay buffer is a line list. Faces need a new GPU
  path. Outline only in this slice.
- Placing the box on the table or on a fixture stack: needs figures that
  no machine file states.
- A new envelope in the `app` crate: the shape maths is pure, so it lives in
  `camcraft-post` (`machine/envelope.rs`). `app` only reads the world.
