## #2362 - Write P0 on G86 when bore dwell is zero on grblHAL and Tormach

**Decided.** `assets/posts/grblhal.cps` and `assets/posts/tormach.cps` (`onCycle`)
write `P0` (`"P" + secs(dwell)`) on every opening `G86` block when the bore
dwell is `0`. Modal `X Y` holes inside an open `grblhal.cps` group still
inherit the group's `P0`, while `tormach.cps` emits `G80` on each hop and writes
`P0` again on each reopened `G86` block. `G82` and `G89` keep `dwell > 0`, and
zero-dwell `BACKBORE` still posts as `G85` with no `P`.

**Why.** PathPilot is built on LinuxCNC, whose public G-code reference gives
`G86 ... R- L- P- $-`, lists a missing `P` number as a canned-cycle error when
entering `G86` after `G80` (or at cycle start), and accepts any non-negative `P`
(`P0`). grblHAL lists `G86` in its core supported G-code set and follows the
LinuxCNC canned-cycle specification. Adding `P0` on the opening `G86` block is
the smallest change that satisfies the control without altering cycle motion.
Not run on physical grblHAL or PathPilot controller hardware.

**Dropped.**
- Substituting `G85` for zero-dwell `BORE`: unlike `BACKBORE` (`G89` -> `G85`,
  both feed-out), `G86` stops the spindle and rapid-retracts, so `G85` would
  change the retract behavior.
- Expanding `G86` into manual rapid/spindle blocks: grblHAL and PathPilot both
  document canned `G86` directly.
- Writing `P0` on Fanuc-family or Sinumerik `G86`: those dialects document no
  dwell word on `G86` and refuse positive bore dwells by name.

**Cost to change.** One condition each in `assets/posts/grblhal.cps` and
`assets/posts/tormach.cps`, pinned by `crates/post/tests/bore_cycle.rs` and
the `grblhal.nc` / `tormach.nc` goldens under `crates/post/tests/golden/bore/`.
