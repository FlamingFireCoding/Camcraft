## #1501 - Sodick ALC400G wire EDM machine file

**Decided.** One shipped machine, `assets/machines/sodick-alc400g.mch`
(picker name "Sodick ALC400G", kind `wire_edm`), registered in
`posting::MACHINES`. Pinned by `crates/app/tests/sodick_alc400g.rs`.

**Sources (public, Sodick's own pages; read 2026-11-15).**
- https://sodick.eu/products/wire-edm : ALC400G family, X/Y/Z 400 x 300 x 250 mm, U/V 150 x 150 mm, taper +-25 deg at 130 mm.
- https://www.sodick.co.th/EN/EDW.php : same X 400, Y 300, Z 250, taper +-25 deg at 130 mm, U x V 150 x 150 mm.
- Sodick ALC series brochure (text copy at https://www.scribd.com/document/479890863/Sodick-ALC-series-wire-cut-EDM-machines): ALC400G X 400, Y 300, Z 250, U x V 150 x 150 mm, taper +-25 deg at 130 mm.

**Stated in the file.** X 400 mm, Y 300 mm, U 150 mm, V 150 mm of travel;
`edm.max_taper_deg` 25. The sources give a travel length, not a machine
origin. The file states each travel centred on zero (like the shipped mills),
so the length is the published one and the position is our convention.
The 25 deg is the published figure at 130 mm thickness; it is the file's
limit for any thickness.

**Left out (no public source found; the app refuses what needs them).**
`edm.lower_guide_z`, `edm.upper_guide_z`, `edm.awt`, `edm.wire_diameters`,
`edm.conditions`, `edm.codes`, the Z travel (a wire machine has no Z axis in
our model: the guides carry it). A cut is refused by name ("states no EDM
condition") until the user adds a condition row from their own control. The
`C` and three-digit condition names of the issue were not found in a public
Sodick table, so none ship. The test adds one clearly marked test row to show
a punch posts.

**Post.** `sodick-wire.cps` is offered for the file (every wire machine lists
both wire posts; a machine file does not name a post). That post was written
from the 1998 Mark 21 / EX21 manual. Sodick's ALC-G brochures list a newer
control (the ALN400G sheet names a "K-SMC-LINK" NC unit), so the ALC400G is NOT
confirmed to be a Mark 21 machine. The post is offered, not promised: it still
says "not verified on a control" in every program.

**Considered and dropped.**
- A Mark 21 model (A320D, AP350, AP500): only dealer listings and a 1998
  manual give travels, and they disagree (A320D U/V 25, 50 or 1.96 in per
  listings). Not a maker's published sheet, so not used.
- Numbers from used-machine dealers: not a public maker source.
- Guessing guide heights or AWT from the model class: the issue forbids it.

**Cost to change.** Edit the `.mch` and the pinned figures in the test.
