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
