## #1928 - A Contour on an open run of the stock stands one radius off the run

**What was decided.** A Contour whose selection holds an open run on the
setup's stock (`part_solid_of` false) no longer rides the run with the tool
centre. The tool centre stands one tool radius plus the stock to leave off
the run, on the side that Side names
(`machining2d::contour_open_run_sided`,
`crates/motion/src/machining2d/contour_run_stock.rs`).

- Outside: the tool stands on the side of the run without stock.
- Inside: the tool stands on the stock's side of the run.
- Auto: the tool stands on the side away from the part (the setup's parts
  in the stock, `keep_law::parts_in_stock`).
- The stock side is read from the stock's sections with the #1777 probes
  (`material_side`). The part side for Auto is read from the part's section
  split by the run (`boundary_chains::sides_of_chain`).
- Climb keeps the stock on the right of the tool (#1555); conventional
  travels the other way.
- Refused by name, never guessed:
  - Inside or Outside with no stock beside the run at any cut level;
  - Auto with no part at the cut levels, with the part on both sides of the
    run, or with the part on different sides at different levels;
  - a tool beside the run that would overlap the part;
  - an offset that is not positive, or that folds onto itself.
- The selection note "Side does not apply to open runs" now shows only
  when no pick is on the stock: for stock runs, Side applies.

**Why.** The maintainer picked an edge of the stock with a D6 tool. The
tool centre rode the edge, so the tool took 3 mm off the model. #1777 kept
this centreline ride for stock picks ("there is no part to keep there"),
but a stock edge is a boundary the user draws to cut beside, and the tool
must stand off it as on any open profile (RS-274 G41/G42 practice).

**Considered and dropped.**
- Keep the centreline ride and add a "Tool on the line" choice. Dropped: a
  new parameter needs a command and an MCP field, and the issue asks for
  the offset; the centreline is still available through Containment On of
  an open Boundary.
- Auto as "off the stock" (the stock's air side). Dropped: a stock edge
  that the user picks is usually to cut beside the part, and away from the
  part is the side that cannot gouge. Outside gives the stock's air side.
- Use the resolved `cp.side` (Auto turned into Inside or Outside by the
  ring reading). Dropped: that resolution reads closed rings; for an open
  run it would turn Auto into a fixed side with no reason the user sees.

**Not changed.** Runs on a part (`contour_open_run`, #1777) are unchanged,
and so is its fallback: a part run with no part material beside it at any
level still rides its line, with a note. The viewport still draws the
picked run. No golden CL or NC file changed.

**Cost to change later.** Small: one function in the motion crate and one
call in `generate.rs`.
