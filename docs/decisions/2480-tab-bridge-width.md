## #2480 - 2D Contour tab width is the narrowest bridge width

**What was decided.** The tab window on the tool-centre ring is the tab
width plus the tool diameter (`ring::tab_window`). The tab width that the
user sets is now the narrowest width of the bridge that stays, on the
centre line of the cut channel. Before, the window was the tab width
alone, so the bridge was `tab_width - D` wide, and a tab width of `D` or
less left no bridge.

**Why.** The cutter disc reaches `D/2` past each end of a window on the
centre ring. A user sets the tab width as the material that holds the
part (general CAM practice). The old meaning freed the part without a
message when the width was `D` or less, and verify does not find a freed
part (#1565).

**Considered and dropped.**
- Keep the old meaning and refuse a tab width `<= D`: this keeps the
  surprising meaning, and every tab stays narrower than set.
- Measure the bridge at the part wall and scale it on curved walls. (The
  first version of this PR said that a centre-ring window of
  `tab_width + D` is enough on all walls. The review showed that this is
  wrong at corners and with leads: see "Review fixes".)

**Cost to change later.** A project saved before this change gets the
longer tab windows (each tab grows by `D` on the centre ring) the next
time its contour operation generates. Golden files with tabs change with
it.

### Merge of main after #2460 (takeover refresh)

**What was decided.** Main (#2460) moved the level loop of
`contour_impl` into `emit_band`, which the whole-model contour bands
share. Git merged `contour.rs` without a conflict, but the PR's line
`say(&mut lead_notes, n)` did not compile there, because `lead_notes` is
now a `&mut Vec<String>` parameter. The line is now `say(lead_notes, n)`.
The tab window rule and the fill note thus apply to the single-band path
and to the whole-model bands. In `examples/first-router-part.camproj`,
the format version and the drill note come from main. The contour cache
comes from this PR. A regeneration with `WRITE_GOLDEN` gives the same
contour cache.

**Considered and dropped.** A full regeneration of the example: it also
changes stamps, a stock field and tool numbers that main has not
regenerated. These lines are not part of this PR.

### Review fixes (blockers B1 and B2)

**What was decided.**
- The rule is on the true material. The bridge at a tab is the smallest
  distance between the cut on its two sides (the walk ends and the leads),
  less `D`. It must be at least `W`, less a numeric slack of 1e-6 mm.
- B1 (corners): `tab_bridge::fit_tabs` starts each window at `W + D` and
  builds the motion of each keep. It then measures each bridge and grows
  the windows of a short gap by the deficit. A bound of 64 rounds applies.
  A picked start (#1473) grows its windows backward only, so each level
  still enters at the pick.
- B2 (leads): the same loop measures the lead-in and the lead-out of each
  keep. A lead that reaches into a tab thus widens its window. No lead
  type is refused, and `examples/first-router-part.camproj` keeps its
  Arc leads.
- If the windows fill the loop, or no bridge fits (the path turns back
  on itself within `W + D`), the tabbed levels of that loop cut nothing.
  The note starts with `tabs:` and says why. The part stays held.

**Why.** The bridge is measured on the generated geometry itself. One law
then covers sharp inside corners, the corner arcs of an outside ring, a
curved wall and every lead family, without a formula for each case. It
also covers the Perpendicular lead at a corner, which no formula listed.

**Considered and dropped.**
- Move tab centres off the corners: the user sets equal spacing, and a
  moved tab does not fix a lead that reaches into the tab.
- Refuse the Linear, Arc and Tangential leads on tabbed levels: the leads
  can be kept with a wider window, and a refusal makes the example
  project and the saved projects fail.
- A closed formula (chord of the window at a corner, plus the lead reach):
  it misses the lead-fit and the lead-extension changes, and each new lead
  type needs a new formula.

**Cost to change later.** A saved project with tabs at corners or with
leads gets wider tab windows the next time it generates. The fit runs the
keep motion once for each round on each tabbed ring. It is cached for
each ring in one band.

**Example project.** In `examples/first-router-part.camproj`, only the
moves of the `2D Contour1` cache change. At the tabbed levels, the ends of
each keep (the lead-in, the walk ends and the lead-out) move 3 mm away from
each tab: the Arc r3 lead reach now widens each window. The stamp, the
other operations and the tools stay as they were. A full `WRITE_GOLDEN`
run also adds tool `number` fields and a new contour stamp, which main has
not regenerated. These lines are not part of this PR.
`examples/first-router-part.nc` is recorded again with `camcraft-cli
process` from this project. Only the `2D Contour1` tab-gap lines change
(lines 116-148). The G73 line of the drill stays as main recorded it
(#2560).

### Second review (head 613ba4abf)

**What was decided.**
- A NaN bridge distance does not count as wide enough. The window grows,
  or the fit ends with the `no_fit` note.
- If `tab_count * (W + D)` is at least the loop length, the fit gives the
  "fill the whole loop" note before it reserves one entry for each tab.
  The app and MCP do not limit the count, and a count near `u32::MAX`
  would reserve about 137 GB.
- The fixtures of `contour_tab_bridge.rs` that fit their tabs now assert
  one cut run between each two tabs. A fixture cannot pass through the
  "fill" path.

**Known limit (N2).** The fit compares only two neighbouring cuts: the
last `W + D` of one walk with its lead-out, against the lead-in and the
first `W + D` of the next walk. A third cut can come closer to a tab
beyond that reach: a U-turn of the same ring, or a different ring at the
same level. Then the material at the tab can be narrower than `W`. Main
has the same limit, so this PR does not cause it. It needs a follow-up
issue.
