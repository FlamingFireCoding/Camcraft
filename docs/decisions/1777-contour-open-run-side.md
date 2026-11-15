## #1777 - A Contour on an open run of a part cuts on the run's air side

**What was decided.** A Contour whose selection holds an open run (an edge
chain that does not close) on a part no longer rides the run on the tool
centre. The tool centre stands one tool radius plus the stock to leave off
the run, on the side that holds no part material
(`machining2d::contour_open_run`, `crates/motion/src/machining2d/contour_run_side.rs`).

- The side is read from the part, not from the Side setting. At every cut
  level the part's section is taken 0.01 mm above the level (so a floor on
  the level is not read as material beside the run), and two points
  0.05 mm left and right of the middle of each run segment are tested.
- Side (Inside / Outside / Auto) applies to closed rings only. The path
  note says so, and the selection note now says "Side does not apply to
  open runs".
- Climb travels with the material on the right of the tool (#1555);
  conventional travels the other way.
- Refused by name, never guessed:
  - material on both sides of the run at a level ("on both sides");
  - the part on the left at one level or segment and on the right at
    another ("its air side is not clear");
  - the tool beside the run would overlap the part, for example where the
    run ends at a wall ("the tool beside the open run cuts the part");
  - an offset that folds onto itself.
- A run with no part material beside it at any level (for example, a run
  above the part's top) keeps the old behaviour - the tool centre rides
  the line, because nothing is there to cut - and the path note says so.
- Runs of a selection whose material is not a finished part
  (`part_solid_of` false: picks on the setup's stock, or no part facets)
  keep the centreline ride unchanged - there is no part to keep there.

**Why.** The maintainer picked one open edge of a block with Side Inside.
The tool centre rode the edge, so half the cutter cut the part that must
stay. The verifier flagged the gouge and the post was blocked (so no wrong
G-code left the program), but the operation could not do what the user
asked. Cutter compensation to the air side of an open profile is the
published practice (RS-274 G41/G42); the part's own sections are the one
source that knows which side is air, so the rule needs no new parameter.

**Considered and dropped.**
- Choose left/right of the run from the Side setting. Dropped: an open run
  has no inside, and the run's direction comes from the chain walk, not
  from the user - the result would look random. The part already tells
  the side.
- Read the side from the part's silhouette instead of its sections.
  Dropped: an undercut or a step gives a silhouette that disagrees with the
  material at the cut level; the sections are what the tool meets.
- Share `offset_run` with `contour_open.rs` (the open-boundary ride).
  Dropped for now: `contour_open.rs` is in draft PR #1756 (issue #1727),
  and editing it here would conflict. The offset is duplicated in
  `contour_run_side.rs`; a follow-up can merge the two once #1756 lands.

**Not changed.** The viewport still draws the picked run (`chains_of`),
not the offset ride; the toolpath preview shows the ride. No golden CL or
NC file changed.

**Cost to change later.** Small: one function in the motion crate and one
call in `generate.rs`. A user parameter for the side of open runs could be
added on top of it; the refusals would stay.
