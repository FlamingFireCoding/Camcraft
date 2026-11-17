## #2356 - Refuse G73 and G84 dwells that the four posts do not preserve

**Decided:** Fanuc, Haas, Mazak and Doosan DNM posts refuse a positive
G73 or G84 dwell by name. The diagnostic describes a post limitation,
not a claim that the controller cannot dwell. The guard checks raw
`p.P > 0` before seating the tool or reusing an open cycle group. A
fractional millisecond dwell cannot disappear through rounding. Zero-dwell
cycles remain available.

**Why:** The issue permits a named refusal. This is the smallest change
that stops the posts from silently dropping a requested dwell. A guard
before the modal shortcut also covers a later hole that adds a dwell to
an already open cycle group.

Public manuals describe different behavior across these controls:

- [Fanuc B-64604EN-2/01](https://studylib.net/doc/28318238/fanuc-series-0i-model-f---operator-s-manual---machining-c...)
  §5.1.1 lists G73 without P. Sections 5.1.8 and 5.2.1 describe P for
  G84. The Doosan DNM post targets the Fanuc 0i/31i family named in its
  provenance header.
- [Haas G73](https://haascnc.com/service/codes-settings.type=gcode.machine=mill.value=G73.html)
  supports a modal P dwell. The published
  [Haas G84](https://haascnc.com/service/codes-settings.type=gcode.machine=mill.value=G84.html)
  word list does not list P.
- [Mazak Matrix Series M](https://douglasrudd.com/manuals/Mazak%20EIA%20-%20Programming%20Manula%20for%20Mazatrol%20Matrix.pdf)
  §§14-5-5 and 14-5-15 include P for G73 and G84. G73 describes time or
  spindle revolutions; G84 describes time.

These are behavior references only. No vendor implementation or manual
wording was copied. The change was not run on controller hardware.

**Considered and dropped:** Adding native dwell output for each control
would require separate coverage for units, modal P state and tapping
modes. It would make this safety fix larger. Silently omitting the dwell
or assuming one P convention across all four posts was rejected because
it can change the machining operation without telling the operator.

**Cost to change later:** Replace each supported post's refusal only after
pinning its documented dwell units, modal transitions and tapping mode.
Keep the other posts' guards and the zero-dwell regression coverage.
