## #1836 - Fanuc-family posts write every length word with a decimal point

**What was decided.** The five Fanuc-family posts (`fanuc.cps`,
`fanuc-lathe.cps`, `haas.cps`, `mazak.cps`, `doosan-dnm.cps`) write each
length word with a decimal point: `Z14.`, never `Z14`. The same applies to
the rotary angle words (`fanuc.cps`, `doosan-dnm.cps` `A`/`B`/`C`, the lathe
`C` word), the lathe feed per revolution and thread lead (`F0.2`, `F2.`)
and the G76 `R` finish allowance. The goldens of these posts were made new
in the same commit.

**Why.** Fanuc parameter 3401 bit 0 (DPI, published Series 0i parameter
manuals) sets how the control reads a word with no decimal point. With
DPI=0 the value is in least input increments: `Z14` under IS-B is 0.014 mm.
A rapid to `Z14` then goes to almost Z0, which can cause a collision. With
the point, the control reads the same value with DPI=0 and DPI=1. The
project's other ISO posts (`sinker-iso.cps`, `sodick-wire.cps`) already
keep the point for the same reason.

**What was kept.**
- Words that the control documents as "no decimal point" stay in whole
  least increments: the lathe G76 `P`/`Q` and G83 `Q` words. DPI does not
  change how the control reads them.
- Dwell `P` (whole milliseconds), rpm `S`, tool `T`, `H`, `D` and program
  numbers stay integers.
- Feeds per minute stay as before: whole mm/min (`F300`), or in/min at two
  decimals. A no-point F per minute is read in mm/min (or in/min) on these
  controls, and a wrong feed does not move the tool to a wrong position.
- Comment text (the tool note `D10 R0.5`, the part-load advance) and
  macro-B expressions keep the old trimmed form. A macro-B constant is
  not scaled by DPI.

**Considered and dropped.**
- State "DPI=1 required" in each post header and in the machine
  documentation. Dropped: it moves a collision risk to a parameter that
  the user must check on each machine, and the issue offers the point as
  the first option.
- Change the non-Fanuc posts (Sinumerik, Heidenhain DIN/ISO, Tormach,
  grblHAL, TinyG, Carvera). Dropped: those controls read a no-point
  value in mm or inch, and their goldens stay as they are.

**Cost to change later.** One helper (`pt`) per post and a golden refresh.
