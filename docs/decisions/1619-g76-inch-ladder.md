## #1619 - Tormach lathe G76: the grid J makes R1's ladder cut the record's passes

**Decided.** `g76Grid` in `assets/posts/tormach-lathe.cps` no longer takes
J (first depth) to the grid with a plain `floor`. LinuxCNC's G76 with R1
("constant depth", public LinuxCNC G-code reference) cuts J, 2J, ... with
the last pass clamped at K, so ceil(K/J) passes. The record lays
n = ceil(height / first depth) equal rungs, so J must satisfy
(n-1)J < K <= nJ. J is the shallower grid rounding when that holds;
when the shallower J would add a pass, it is the one grid step up, never
a full grid step heavier than the record's first cut. When no grid J
meets both, the record is refused by name ("no first depth J on the
control's grid makes R1's equal ladder cut the record's n passes").

**Why.** The inch golden posted J0.0314 K0.0944: ceil(0.0944 / 0.0314) = 4
passes where the record laid and the verdict judged 3. J0.0315 is
0.000004 in (0.1 um) over the record's 0.8 mm on the diameter and cuts
0.0315, 0.0630, 0.0944: three passes, the root still rounded outward.
A scan of realistic records (first depth 0.05-0.5 mm, height 0.3-2 mm,
four minor diameters, the stream's half-micron lattice) found the old
rule mis-stating 2.7% of inch records and the new rule refusing none in
either unit; mm words are unchanged (2 x first depth already sits on the
0.001 mm grid).

**Considered and dropped.**
- Shrinking K (root further outward) to fit the floored J: the thread
  would be cut short of its minor by up to n grid steps, a thread not to
  depth.
- Refusing whenever the floored J miscounts: it refused the plain M12x2
  inch fixture, a common thread, for a 0.1 um difference.

**Cost to change later.** One function in the post script and the inch
G76 golden's J word.
