## #1697 - the STEP bench meters truck's faces

**Chosen.** The STEP bench in `crates/kernel/tests` (step_agreement,
step_import) now meters every face that it gives to truck's mesher
(`tests/step_util/truck_guard.rs`). The meter is the bench copy of the
first meter that crates/app put on its STEP faces (#1536, 25 M evaluations
for each face). The meter also reads the bench's cancel token. After the
cap, or after the cancel, each evaluation gives one fixed point, and the
subdivision stops in one sweep. The bench then refuses the file by name
("shell #N: face i of n (B-spline surface) not meshed: ...").

**Why.** The release gate of the v0.20.0 cut stopped on
`bspline_discontinuous.step`. That fixture came in with #1536. The
solidcore reader and the solidcore mesher answer on it in a few
milliseconds (measured in a probe run, debug). The bench's truck reader
called `robust_triangulation` without a meter. truck's subdivision does
not end on a B-spline that is not continuous, and truck reads no token.
Thus the test thread waited for ever, and the cancel had no effect. The
app was not hit: its importer meters each face since #1536.

**Considered and dropped.**
- A longer time limit: the read never ends, so no limit is enough.
- A knot check before the mesh (refuse a B-spline with an interior knot
  past its degree): this does not stop the second known runaway, the pole
  of a corner blend (`fillet_corner.step`, #1546), where the knots are
  correct.
- Run truck in a child process and kill it at the limit: more machinery,
  and each runaway file then costs the full 60 s.
- Leave a runaway thread to run on in the background: it uses CPU and
  memory for the remainder of the test binary.

**Cost.** `truck-topology` becomes a dev-dependency of camcraft-kernel
(workspace pin, already in the graph through crates/app): truck-stepio does
not re-export `CompressedShell`, which the meter must build. No
`camcraft-*` edge is added. The library is not changed.

**Verdict rows.** The three fixtures that came in after the bench's last
triage (`bspline_discontinuous.step`, `bspline_dome.step`,
`fillet_corner.step`) get rows with the verdicts that the bench now reads.
