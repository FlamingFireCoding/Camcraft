## #1649 - A flipped WCS axis is carried by the post, case by case

**What was decided.** A flipped WCS axis does not change the cut. It changes
the sign of the numbers that the controller reads on that axis, measured
from the same work origin: the posted word is `-(p - O)`. So generation,
verification and stock simulation stay in the setup frame and are never
refused for a flip. The stream states the flip for each section
(`PARAM WcsFlip X|Y|XY...`) and one post pass, `post::wcs_flip`, mirrors
that section with the same reflection that #374 uses for an inverted
machine axis (`invert::translate_rules`).

Each case, and what the program does:

| Case | Result |
|---|---|
| Flip X, or flip Y (a mirror) | Carried. Coordinates, arc centres, start point, origin and stock box negate on the axis. G2 and G3 swap for an arc whose plane holds the flipped axis. G41 and G42 swap. A helix keeps its lead and reverses its turn direction. A drilling cycle keeps Z, R and its retract |
| Flip X and Y (a half turn about Z) | Carried. X and Y negate. XY arcs and the compensation side do not swap. A G18 or G19 arc holds one flipped axis, so it swaps |
| Climb or conventional | Not changed. The physical cut is the same, and the spindle direction is the same. Only the G-code words change |
| Flip on an axis that the machine also inverts (#374) | Carried. The two mirrors compose, and the program has the standard signs |
| Flip Z, alone or with X or Y | Refused by name. Tool-length compensation and the cycles' R plane and retract need the controller's +Z to leave the work |
| A post script without `axisInversion = true` | Refused by name. Only Fanuc and Haas are verified on a mirrored stream |
| Flip with a rotated WCS (`WcsYaw`, Bake or Control) | Refused by name. The sign of a G68 angle under a mirror is not verified |
| Turning, threading cycles, live tool, A/B/C moves, tilted planes, EDM, wire, plasma | Refused by name. A mirror changes the thread hand and the rotary direction |
| Probing (any probe operation in a flipped setup) | Refused by name, at the verification gate and in the post pass. See "Probing in a flipped setup" below |
| A section that uses the origin or the start point of the section before it, when the flip changes between the two | Refused by name. The script would subtract the wrong origin |
| A `WcsFlip` value that the pass cannot read | Refused. It never posts as "not flipped" |

The mirrored program says so in a comment at each section:
`(WCS X FLIPPED - PROGRAM MIRRORED)`. The setup sheet says the same, and
the machine travel check reads the posted sign.

**Why.** The maintainer's words (2026-11-17): "the ability to flip axis
from a positive to negative is fundamental and should not be blocked..
posting and generation should take it into consideration rather..". The
physical cut must not move when a user flips an axis: the fixture, the
stock and the collision checks are in the setup frame. A mirror in the
post is one place, it has an equivalence test (a controller reader with
the flipped axis must trace the CAM path), and it reuses the reflection
that #374 already pins.

**Considered and dropped.**

- Generation in a mirrored frame. It moves the cut, and each generator,
  the stock simulation and the gouge checks would need the mirror. A
  climb cut would become a conventional cut.
- A controller mirror code (G51.1 and its relatives). It is modal state
  that differs between controllers, and it reverses compensation in the
  controller where we cannot verify it.
- Carrying the flip in each script. The engine pass does it one time, and
  the script flag `axisInversion = true` says that the script was verified.
- Carrying Z now. It needs a decision on G43, the R plane and the retract
  moves for each dialect. It is #2474.

**Cost to change later.** Low. The flags and the stream parameter do not
change when a refused case becomes a carried case: remove the refusal in
`post::wcs_flip` and in `setup_wcs_flip`, and add the equivalence test.
A script joins by declaring `axisInversion = true` after its goldens
pass on a mirrored stream (#2475). A flipped and rotated WCS is #2476.

## Probing in a flipped setup (review of PR #2492, 2026-11-18)

**What was decided.** The meaning stays as built: a flipped axis means
that the machine control runs that axis reversed, and the post writes the
mirrored program for it. A probe operation in a flipped setup is refused
by name. `setup_wcs_flip` names it at the verification gate ("probing
does not post in a flipped WCS ... (operations: Probe1)"), and
`post::wcs_flip` refuses a flipped section that holds probe records, so
each stream builder is covered. The MCP `set_wcs_flip` reply names the
refused operations of the setup in its `refused` field.

**Why.** The maintainer's decision (2026-11-18): keep the meaning, and
refuse probing in a flipped setup with a message that names the flip. A
probe writes a work offset (`G10 L2`) from a measured position. That write
is verified for a machine axis that runs reversed (#392,
`probeAxisInversion`), not for a control that mirrors one work offset. A
wrong offset moves every later cut, so a refusal is safer than an
unverified program.

**Considered and dropped.**

- Mirrored probing through `probeAxisInversion`. The flag proves the
  inverted machine axis only; the offset that a mirrored work offset
  needs was not verified on any control.
- A refusal only for a probe that writes an offset. An inspection probe
  reports a position in the mirrored frame too, and no consumer of that
  report is verified for it.

**Cost to change later.** Low: remove the probe arm in
`setup_wcs_flip::op_problem` and the `probe` check in `post::wcs_flip`,
and add an equivalence test for the offset write.
