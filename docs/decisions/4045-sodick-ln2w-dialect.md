## #4045 - Sodick wire: an LN2W post dialect and a Sodick AQ600L machine file

**What was decided.**

- A second post script, `assets/posts/sodick-ln2w.cps` ("Sodick Wire EDM
  (LN2W)"), stands beside the Mark 21 script. The Mark 21 script
  `sodick-wire.cps` has no change, so its output is byte-identical.
- Each cut starts in the order that the owner of an AQ600L (LN2W) gave on
  #4045: `G00 X Y`, `Z0.`, `T91`, `T84`, `T94`, the `C` word, `U0. V0.`,
  then the first `G01`. `T84` / `T85` and `T94` / `T89` come from
  `edm.flushing` and `edm.tank`, as in the Mark 21 script.
- The post writes no `G20` or `G21`. The owner's control alarms on `G21`
  (`E00170`). Thus the script declares `programUnits = "mm"`, and the
  engine refuses an inch document by name. A header note says that the
  unit setting of the control must be millimetres.
- A constant taper posts as `G51` / `G52` with `A` (degrees for each
  side) before the first leaning move, and `G50` before the move that
  stands the wire upright. A positive stream angle is `G51` when the
  machine file states `edm.taper_angle.positive: top_left`.
- `TP` and `TN` stand before the positioning move of a tapered cut. The
  stream gives X Y on the top of the work (`PARAM WirePlane`). Thus the
  program plane is the top of the work: `TP` is the height of the top
  above the worktable, and `TN` is the height of the bottom.
- A new machine key, `edm.table_to_part_bottom` (a length, 0 or more),
  states the height of the bottom of the work above the worktable. A new
  stream parameter, `PARAM WireBottom`, states the Z of the bottom of the
  work beside `WirePlane`. The post computes `TN` = the key, and `TP` =
  the key + (`WirePlane` - `WireBottom`). Without the key the post refuses
  the taper by name.
- A skim pass takes the `edm.skim_flushing` word (`T85` on the AQ600L)
  and its own `C` word before its first move.
- A new machine file, `assets/machines/sodick-aq600l.mch` ("Sodick
  AQ600L"), names the LN2W post in `controller.post`.
- The goldens are the EDM test parts square, 2-axis and taper
  (`mkinney/cam-tools-and-fixtures`, `samples/edm-validation`, commit
  `c687f53f9`), copied to `testdata/models/edm/`. They are our own parts.
- The header keeps the line `NOT VERIFIED ON A CONTROL`.

**Why.**

- The engine evaluates one script text and has no include. A dialect
  option in the Mark 21 script cannot be the default post of a machine
  file, because `controller.post` names a registry entry. A second script
  gives the AQ600L its default post and cannot change the Mark 21 output.
  The shared helper functions are copied.
- The owner's known-good program (recorded on #1500) states `TP100.` and
  `TN0.0` before any motion. Thus the control accepts a program plane
  above the next plane, and the two words before the motion.
- The path stays the wire-centre path with the kerf in it. The owner's
  start sequence has no `G41` / `G42` and no `H` word.

**Sources.**

- Sodick Inc., "Wire EDM Machine Operation Training, LN1W / LN2W Control"
  (part number 6300015, January 2012). This session could not open the
  document (the web request was blocked). The facts are those that the
  repository already records from it: #4045 (body), #1500 (comments),
  and the decisions #1496, #1497, #2649 and #2686.
- The statements of the owner of a Sodick AQ600L with the LN2W control,
  relayed on #4045 and #1500 (2026-11-25).
- No other CAM product's post or output was read.

**Not confirmed. Check before the PR is flagged ready, or at the control.**

- **The AQ600L figures.** X/Y 600 x 400 mm, U/V 120 x 120 mm and taper
  +-20 degrees are written from the worker's memory of Sodick's AQ600L
  specification. This session could not open a Sodick source, so no
  source is named. A person must check the figures against Sodick's
  published AQ600L specification, and name that source here.
- The corner rule `mitre` in `edm.taper_angle`. No recorded source states
  what the LN2W control cuts at a sharp corner under `G51` / `G52`.
- `G51 A` and `G50` each on a line of their own, before the move.
- The number form of `TP` / `TN` for values other than the owner's
  `TP100.` and `TN0.0`.
- A program with no `G40` at the start. The owner's sequence and the
  manual's example have none. A header note says that no wire offset is
  used.
- The guide heights 0 and 130 mm, the conditions `C001` / `C002` / `C903`
  and `edm.table_to_part_bottom` 0 in the machine file are examples. The
  file says so.

**Dropped.**

- A dialect option in `sodick-wire.cps`: see "Why".
- `G40` on the `G00` (the Mark 21 form): it is not in the owner's
  sequence.
- `G41` / `G42` with `H` registers for each pass: the wire laws judge the
  generated wire-centre path, and no stored `H` value ties a control
  offset to that path (#1498). A `COMP` record is refused by name.
- 4-axis cuts (`G74` / `G75`, U V profiles): #2130 is open for them. The
  post refuses them by name.
- X Y on the bottom of the work: the stream of an angle-word machine
  carries the top plane, and the owner's program shows that `TP` can be
  above `TN`.
- `TP` / `TN` from the stock or from the work offset without a table
  height (#1497 dropped this).
- A row for the new key in the machine editor of the UI. The key is
  read from the file and kept through an edit.
- An inch program: no unit word can be written.

**Cost to change later.** The start order is one function (`startCut`).
The taper words are one function (`taperWord`), and the heights are one
function (`taperHeights`). A change of the program plane to the bottom of
the work needs a plane rule in `edm.taper_angle` and in
`posting_wire::angled`. Each change moves the three goldens under
`crates/app/tests/golden/nc/sodick-ln2w/`.
