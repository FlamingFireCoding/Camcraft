## #2348 - Refuse a G83 dwell on the grblHAL and Tormach posts

**Decided:** The grblHAL and Tormach PathPilot posts refuse a positive
G83 (deep peck) dwell by name, before any G83 block is written. The
message has the same text as the Fanuc-family refusal of #1820 (PR #2048).
A G83 with no dwell posts as before: no P word, and no golden changes.

**Why:** The public references give G83 no dwell word on these controls:

- The [LinuxCNC G-code reference](https://linuxcnc.org/docs/stable/html/gcode/g-code.html)
  gives the G83 format as `G83 (X- Y- Z-) or (U- V- W-) R- L- Q-`. P is
  listed only on G82, G86 and G89 (and G85 has none).
- [Tormach's PathPilot G83 page](https://tormach.com/peck-drilling-cycle-g83)
  gives the program format `G83 X~ Y~ Z~ A~ R~ L~ Q~`, with no P.
- grblHAL's [core wiki](https://github.com/grblHAL/core/wiki/Additional-G--and-M-codes)
  lists G83 as "Drilling cycle with peck" and says that new codes follow
  the LinuxCNC specification. Its [README](https://github.com/grblHAL/core/blob/master/README.md)
  lists the canned cycles but defines no P for G83.

**Considered and dropped:**

- Silently omitting P (the old behaviour): it loses the requested dwell
  with no message. This is the defect.
- Writing P on G83 anyway: no public reference says that these controls
  read it. A word that the control ignores is the same silent loss.
- Expanding the deep-peck cycle into plain moves with a `G4` at the
  bottom: it changes the posted program shape and the cycle-hop law for
  one option. It is a larger change than this defect needs.

**Cost to change later:** If a control version documents a G83 dwell,
remove the refusal for that post, write P in seconds (the unit of G82 on
these controls) and change the pin in `crates/post/tests/g83_dwell_linuxcnc.rs`.
No real control was exercised here.
