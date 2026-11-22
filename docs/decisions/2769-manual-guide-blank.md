## #2769 - Fanuc post: the stock as a Manual Guide i blank block

**What was decided.**

- The stream states the setup's stock as a blank with two new PARAMs:
  `StockBlank` (`box`, `cylinder-x|y|z`, or a reason word) and
  `StockBlankZBottom` (the lower face, world mm). The app fills them for
  every milling section from the document: the stock entity, its shape, its
  rotation, the setup's stock source and the WCS.
- The engine passes the two PARAMs only to a script that declares
  `stockBlank = true`. Only `fanuc.cps` declares it. All other posts get the
  same stream as before, so their output does not change.
- `fanuc.cps` has a new boolean option `manualGuideBlank`, off by default.
  When it is on, the first section writes one blank block after `G21`/`G20`
  and before `G90`:
  - a box: `G1902 B D H I J K` (size; bottom corner to work origin);
  - a bar along Z with its axis on the work origin: `G1900 D L K`
    (diameter, length, top face above the work origin).
- Any other case writes the comment `(NO MANUAL GUIDE BLANK - <why>)` and no
  block: a stock from a model, from the previous setup, a turned stock, a
  rotated or flipped WCS, an inverted machine axis, a bar along X or Y, a
  bar off the work origin, a missing size, or a legacy stream.

**Why.** The issue asks for an opt-in block and a comment, never a wrong
block. The engine filter is necessary: `heidenhain-klartext.cps` refuses
each PARAM that it does not list. The filter keeps every other post and
every golden byte-identical. A shape word in the stream lets the post tell a
box from a cylinder. The stock bounds alone cannot do that.

**Dropped.**

- `G1906` (a bar along X). The research comment says it needs machine
  parameters (14717, 27003). The `K` reference of the bar forms is not
  confirmed on a control. A bar along X writes a comment until a kit test
  confirms `G1900` and `G1906`.
- Always emit the PARAMs to every script. Klartext would refuse them.
- Emit the PARAMs only when the setup's stored option is on. The CLI
  `--post-option` and MCP overrides would not reach the stream builder.
- A new blank block for each setup. Manual Guide i draws one blank for the
  program, so the block is written one time, for the first section.

**Not confirmed on a control.** The `G1900` `K` sign (top face above the
origin, from the manual's "front end face" reading) and the box `I J K`
signs come from the research comment on the issue. The kit test by the
reporter on a Robodrill confirms them. A wrong value there changes only the
simulation drawing: the control skips blank blocks in AUTO/MEM mode.

**Cost to change later.** Small. A new shape is one word in
`posting_stock_blank.rs` and one branch in `fanuc.cps`. Another post that
wants the blank declares `stockBlank = true`.
