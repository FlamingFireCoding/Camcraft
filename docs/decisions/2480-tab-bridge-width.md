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
- Measure the bridge at the part wall and scale it on curved walls: the
  narrowest section is on the channel centre line, so a centre-ring
  window of `tab_width + D` is enough on straight and curved walls.

**Cost to change later.** A project saved before this change gets the
longer tab windows (each tab grows by `D` on the centre ring) the next
time its contour operation generates. Golden files with tabs change with
it.
