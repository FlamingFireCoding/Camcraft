## #2306 - six more suites take the oblique "not judged" device

**Decided.** The suites `circular_pick_law`, `trochoid_law`, `keep_law`,
`kept_parts_live`, `stock_source_holes` and `pocket3d_open_side` turn their
stock 1e-6 rad about Z (`OBLIQUE`) in place of a quarter turn. This is the
device of #2177 (`docs/decisions/2177-square-stock-sim.md`). The shared
helpers are `area_support::stock_turn` / `oblique_refused` and their twins
in `pocket_support`. Each check that the operation's own clause holds the
post now also asserts that the program sim refuses the stock by name
(`stock_follow::OBLIQUE_STOCK`). The fixture box checks take 1e-4 mm in X
and Y (Z stays at 1e-9).

**Why.** #2182 made the program sim judge a quarter-turned box on purpose.
These six suites used the quarter turn to make the sim refuse, so their
"held by the op" asserts saw the sim's gouge clause first. In 17 of the 18
red tests the post was still refused. Only the device was wrong.

**Dropped.**
- Delete the "held by the op" asserts: that loses the pin on the
  generator-time verdict where the sim does not judge.
- A new quarter-turn test in each suite (as #2182 added in its four): the
  four suites of #2182 already pin that the sim judges a quarter-turned box.
- A fix of the chamfer post hole in this PR (the 18th test, case
  "chamfer-rejudge-hint", POSTED on main). The hole also exists with an
  aligned stock since #1628, in `posting_sim::judge_clause`. Its fix adds
  refusals to every judged post and needs its own review and suite runs.
  It is #2308 (P1). With the oblique device this suite pins the re-judge
  where the sim does not judge, as #1637 meant.

**Cost to change.** One helper in each support module; a future device
changes `stock_turn` only.
