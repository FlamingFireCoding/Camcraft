## #2561 - a contour tab is checked against every cut of its level

**Decided.** After `tab_bridge::fit_tabs` fits each ring, the new
`tab_clear` reads each tab's line on the ring against every other cut of
the tabbed levels: the other keeps of the same ring, the far part of the
two keeps beside the tab, and every other ring.

- Another cut nearer than the tool diameter `D` cuts into the tab. The
  loop is held: its tabbed levels cut nothing, and a `tabs:` note names
  the tab and the distance. This is the response that `fit_tabs` already
  gives when no bridge fits.
- Another cut between `D` and `W + D` away leaves the tab at its width,
  with less than the tab width of material beside it. The loop is cut as
  before, and a `tabs:` note names the tab and the width of the strip.
- A picked start that a grown tab covers gets a `tabs:` note that names
  the pick and the point where the tabbed levels start instead.
- `SLACK` stays `1e-6` with no margin for the chord error. A test reads
  the bridge on the posted arcs in chords of `1e-5` mm and pins it to
  `1e-4` mm.

**Why.** The shape-mask measurement on main found tabs with a neck of
0 mm and no note: a tab on the wall of a notch 10 mm wide (6 mm cutter),
and a tab that faces a second part 10 mm away. The return run, or the
other ring, passes 4 mm from the tab and removes it from its stock. A
part can then come free with fewer tabs than the user asked for (Prime
Directive: correct, or refused by name).

The strip case is a note and not a hold. The strip between two kerfs has
the full cut depth, and the tab has only the tab height, so the strip is
not the weak point. The same reading also occurs across a thin wall of
the part itself, which is not a tab defect. A hold there would refuse
common nests.

**Dropped.**

- Move the tab away from the other cut. This changes where the user's
  tabs are, and needs a placement law. Not in this fix.
- Hold only the loops that have no good tab left. The user asked for a
  count; fewer tabs without a refusal is the silent case again.
- Remove a held ring's cuts before the other rings are checked. The
  check uses all cuts as first fitted. This is conservative: a hold only
  removes cuts.
- A margin on `SLACK`. A margin of some microns grows each tab window
  on a straight wall and moves the recorded router example program. The
  measured error on the posted arcs is under `1e-6` mm on the fixtures of
  the test. The flatten step (0.05 mm, 8 samples or more for each arc)
  limits the sagitta to about 0.005 mm in the worst case; the tests
  already accept 0.02 mm.
- The meter in `shape_mask.rs`. That file has 409 lines, so the meter is
  `crates/motion/tests/common/tab_neck.rs`.

**Limits.** The check is local to the tab's line on the ring. It does
not follow the stock strip farther along (a strip that narrows away from
the tab). The two keeps beside a tab count only from where they have
left the tab (`W + D` of walk, then the first point `W + D` from the
line); `tab_bridge` owns the part before that. Tabs of two rings that
face each other at the same place join the two parts and not the stock:
this reads as "cuts into" only when the other ring cuts there.

**Cost to change.** `tab_clear::fit_band` is one function with two
thresholds. A later placement law (move the tab) replaces the hold arm.

## #2561 - review fixes of PR #2705 (takeover by the owner account)

**Decided.**

- `tab_clear::departed` drops only the monotonic exit of a keep beside
  the tab: the pieces whose distance from the tab line keeps growing. A
  piece that comes nearer to the line than the farthest point before it
  starts the counted part of the cut, also inside `reach` and before the
  cut is `W + D` away. A U-turn in a shallow notch beside the tab is then
  read and holds the loop.
- With one keep on the ring, its lead-in and lead-out join the tab at
  both ends. `nearest_other` now reads them through `departed` too, so a
  lead that turns back toward the tab counts.
- `tab_clear::fit_loops` takes the tabbed levels of each loop from the
  depth ladder (`Ladder::ring`). A loop with no tabbed level is not
  checked, and a ring is read against the tabs of ring `i` only when the
  two rings share a tabbed level. With no tabbed level in the band, no
  fit is made.

**Why.** The review of head `edb566d33` found a neck of 0 mm with no note
(an 18 mm deep notch), and a false hold of a deep island beside an island
that stops on a shelf above the tabs. The first is a silent thin bridge.
The second refuses a legal program.

**Dropped.**

- Check the pieces past `reach` against `D - SLACK` only. That reads the
  turn-back but not a strip narrower than the tab width, so the note of
  the strip case would depend on the notch depth.
- Check the adjacent leads for each count of keeps. With two or more
  keeps, `tab_bridge` measures those leads; the brief asked for the
  one-keep case only.

**Limits.** The distance is to the tab line as a segment. Near the ends
of the line this is conservative: a turn-back past the end of the tab can
read as nearer than it cuts. The result is a hold or a note, never a
silent bridge.
