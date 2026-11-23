## #3830 - Every bore-side ring takes the entry

**What was decided.** A bore-side, untabbed ring after another ring at the same level takes the entry on
its first level and the level transition below it, through `engage_level`. The pass link still feeds
straight where the keep-down proof holds. `ring_link` returns `false` when the entry must engage.

**Why.** The pass link lifted and plunged straight into solid at the plunge feed, with no note. Three
square bores at a 5 degree ramp posted 6 axial dives and `notes: []` on main.

**Dropped.** Outside rings (the wall arc and the pass link there have their own tests, and the user's
report was about holes). Changing the arm of keep-down schedules (it would change posted output).

**Cost to change later.** Outside multi-ring contours still plunge through the pass link: a follow-up.
