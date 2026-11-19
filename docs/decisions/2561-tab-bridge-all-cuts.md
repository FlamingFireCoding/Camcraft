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
