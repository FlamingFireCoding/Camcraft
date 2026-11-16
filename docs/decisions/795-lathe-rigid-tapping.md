## #795 - Tormach lathes state `spindle.rigid_tapping`

**Decided.** The three Tormach lathe files state `"rigid_tapping": true`:
`tormach-8l.mch`, `tormach-15l-slant-pro.mch` and
`tormach-15lx-slant-pro.mch`. The pin
`every_tormach_mill_router_and_lathe_file_states_rigid_tapping`
(`crates/app/tests/rigid_tapping.rs`, renamed from
`every_tormach_mill_and_router_file_states_rigid_tapping`) now checks the
lathes too. It expects exactly three Tormach lathes. The plasma table
(`tormach-1300pl.mch`) stays outside the check.

**Why.** Each operator manual has a rigid-tapping section with `G33.1`.
The research comment on #795 (2026-11-15) holds the full table. The
sources (behaviour only, clean room):

- 8L: *8L Operator's Manual* (UM10753), section 10.3.18 "Rigid Tapping
  (G33.1)", p. 151. *PathPilot Tools and Features - 8L* says that Tap uses
  electronic gearing (G33.1, rigid tapping).
- 15L Slant-PRO: *15L Slant-PRO Operator's Manual* (UM10225), section
  6.5.17: "For rigid tapping, program G33.1 Z~ K~." The spindle encoder
  (PN 32560) is standard.
- 15LX Slant-PRO: *15LX Slant-PRO Operator's Manual* (UM10860), section
  10.3.13: "For rigid tapping, program G33.1 Z~ K~". The spindle motor has
  an integrated encoder on EtherCAT.

**What it changes in posted output.** Nothing. The program gate refuses a
tap only on a stated `false`; `true` and unstated pass the same way. The
post mirror now carries `rigidTapping: true` for these machines, and the
script default is already `true`. `tormach-lathe.cps` still refuses a
`TAP` cycle by name. No golden moved.

**Dropped.**

- Leaving the files unstated. The manuals settle the question, and #795
  asks for a stated value when they do.
- Writing `G84` for the lathes. `G84` is the mill canned cycle; the
  Tormach lathes rigid-tap with `G33.1`. Lathe tap output is later work,
  not this change. That work must keep these rules from the manuals: the
  tap is on the centerline (`X0`), the spindle is in `G97`
  (PathPilot refuses `G33.1` in `G96`, SB0046), the spindle starts before
  the block, and the block is `G33.1 Z.. K..`.
- A separate lathe test. One test over all Tormach files keeps one place
  that says which Tormach machines state the capability.

**Cost to change later.** One line in each file and one assert in the pin.
A real PathPilot lathe check is still necessary before a lathe tap post
cuts a part.
