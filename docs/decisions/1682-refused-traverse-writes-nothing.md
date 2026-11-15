## #1682 - grblHAL, Carvera and TinyG write no motion after the no-plane refusal

**Decided.** In `grblhal.cps`, `carvera.cps` and `tinyg.cps`, `xyFirst`
returns `false` when it refuses a first X/Y traverse for want of a stated
ClearancePlane or RetractPlane, and every caller (rapid, feed, arc, and the
grblHAL canned cycle) then writes nothing for its move. X/Y stay unknown, so
every later move of the section is refused the same way (the message once
per program). This is the Tormach (#1614) and Fanuc-family (#1633) rule.

**Why.** The posts named the refusal but still wrote the combined
`G0 X.. Y.. Z..` (and the feeds after it). The app blocks posting on the
warning, but the `post` CLI writes the file with the warning on stderr, so
the refused move was in the program text.

**TinyG too.** The issue said TinyG already stopped (its `startXY` returned
false), but its `onRapid` still wrote the combined rapid after the refusal;
the issue's own "Done means" pin asserts TinyG, so its `onRapid` takes the
same guard. Considered and dropped: leaving TinyG for a follow-up - the pin
the issue asks for would be red.

**Cost to change.** Script-only; the pins are
`first_traverse_clearance::a_first_traverse_with_no_stated_plane_is_refused_by_name`
and `::no_xy_motion_follows_the_refusal_whatever_the_first_move`.
