## #1444 - Slot Undercut: T-slot, keyseat, dovetail and lollipop passes that never move in Z under material

**What was decided**

1. **Four cutter kinds, no new tool fields.** `TSlot`, `Keyseat`, `Dovetail`
   and `Lollipop` join `ToolKind` (file words `t_slot`, `keyseat`,
   `dovetail`, `lollipop`; additive, no `PROJECT_VERSION` step). They read
   the existing fields: `diameter` is the head, `flute_len` the head's
   height, `shank_d` / `shank_len` the neck, `tip_angle` the dovetail's
   flank angle off the bottom face. One reading for every consumer:
   `camcraft_tools::undercut::Undercut` (head shape, neck radius, contact
   point).
2. **One operation kind, `Slot Undercut`** (`OpParams::SlotUndercut`, tag
   44, appended). The centre line is typed: `a`, `b` and a DXF-style
   `bulge` (0 = straight, else a circular arc). The undercut `width` is
   measured at the cutter's widest point, so the pass offset comes from
   the profile's contact radius, not from the tip.
3. **The Z law is a construction law and a verified law.** The generator
   changes Z at one XY station only (`run_in` behind `a` on the entry
   tangent) and checks that on the built path. The verifier has a new
   mode (`Mode::Undercut`) and law (g): any move that changes Z while
   material stands in or over the head's disc is `Warning::UndercutLift`
   (blocking); a neck or holder band with material above its bottom is a
   `ShankHit`. Both are re-read against the stock the program ahead left
   (`acquit_undercut_by_prior`), so a neck slot or an entry pocket that an
   earlier operation cut is air. With no such cut the conviction stands.
4. **An undercut clears no column.** In `Mode::Undercut` the verifier's
   rest model takes no cut from the path, and the column stock engines
   (program verdict, carried stock, rest grids) remove nothing for an
   undercut operation. A column holds one top per (x, y): lowering it to
   the head would remove the roof and report a false gouge, and would let
   a lift through the roof pass as a retract in its own kerf.
5. **The true profile where a solid can hold it.** The solid law judges
   head and neck against the part mesh with the real head: `Prof::Dove`
   (frustum) and `Prof::Ball` (whole ball) beside the existing disc. The
   field stock (`Stock::simulate_undercut`) removes the head and the neck
   as two convex solids (their union is the non-convex cutter), with a new
   exact frustum solid (`SdfTool::Dovetail`). The stock view uses it.
6. **Refusal instead of support where nothing is verified:** a flipped WCS
   and a rotated work offset refuse the kind; an aligned setup and a WCS
   yaw do not list it (refused by their own doors); no 3+2 tilt.

**Why**

- The issue's safety statement is about Z moves under material. A plain
  contour's laws read a vertical retract as "rides its own kerf", which is
  false under a roof. The law had to say so in the verifier, not only in
  the generator, because a path can be edited, pasted or produced by a
  later generator.
- Reusing `shank_d` as the neck keeps every existing reader of the tool
  record correct: `Tool::profile`, the drawn-cutter law and the verifier's
  shank band already describe "a cylinder head on a narrower shank". Only
  the dovetail and the ball needed new profile code.
- A union of convex solids keeps the field engine's convex sweep law
  (golden section on a convex distance) true for each piece. A general
  non-convex revolved profile would need a bracketed search (issue 896's
  research, slice 2) and was not needed for these four heads.

**Considered and dropped**

- *A generic `FormMill` kind with a drawn outline* (issue 896, slice 1).
  It needs the drawn outline to survive `cutter_fit::refit`, a digest of
  the flute outline and a non-convex sweep. That is issue 896's work. The
  four kinds here are parametric and need none of it. Issue 896 stays open
  for corner rounding, back chamfer and slitting saws.
- *A new `SimOp` field for the undercut.* `SimOp` has about 90 literal
  construction sites. The undercut rides `Planned::undercut` in the app
  and `Stock::simulate_undercut` in the sim crate instead.
- *New neck fields on `Tool`.* Not needed (see above); the file format and
  every tool dialog stay as they are.
- *Multiple depth levels.* One level per operation: the head cuts the
  whole undercut height when the cutter matches the slot. A taller
  undercut takes a second operation with another Bottom Z.
- *Round far ends.* The far end links offsets with a straight lateral
  feed. The cutter centre never leaves the band within the shift of the
  centre line, so the undercut is never wider than asked; a blind end is
  left square by up to the shift.
- *Picked geometry.* The centre line is typed (as the Slot's is). A pick
  law for undercut faces is follow-up work.
- *Cutting the neck with the column engine.* It would be correct only
  where the neck slot is narrower than the neck (a collision the law
  already refuses), so it adds nothing.

**Literature.** The issue asks for a Perplexity literature search first.
This session had no web or Perplexity tool, so no search was run. The code
was written from published material known to the author, named in each
module's provenance line: Machinery's Handbook (T-slot cutters, Woodruff
keyseat cutters, dovetail cutters: the cutter side views); the side-entry
rule itself is general shop practice and is not taken from one text;
Requicha and Voelcker,
"Solid modeling: a historical summary and contemporary assessment", IEEE
CG&A 1982 (a non-convex solid as a union of convex ones); Frisken, Perry,
Rockwood and Jones, "Adaptively sampled distance fields", SIGGRAPH 2000
(CSG on distance fields by min and max); Boyd and Vandenberghe, "Convex
Optimization" 3.2.5 and Kiefer 1953 (already the solid law's sources: the
convex gap function and golden-section search, which hold for the concave
dovetail and ball profiles). Tri-dexel (Benouamer and Michelucci 1997) was
considered for the column engine and not used: the field engine already
holds an undercut. The maintainer can ask for the Perplexity search to be
run against this list. camlib was not read.

**Cost to change later**

- The kind words and the op's keys are in saved files from the first
  release that ships them: renaming one is a format step.
- `Mode::Undercut` and `Warning::UndercutLift` (`undercut_lift` over MCP)
  are part of the verify contract and of cached findings.
- Moving to a generic drawn form cutter (issue 896) can keep these four
  kinds as presets: `Undercut::of` is the one place that reads them.
