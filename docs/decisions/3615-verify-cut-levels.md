## #3615 - The verifier samples island walls at the cut levels, not every cut Z (HELD: equivalence failed)

### Outcome (read this first)

The plane-set builder was implemented and the all-plane equivalence pin was
run. **The pin failed.** On the existing sloped ramp fixture
(`ramp_at(500, 36.0)` in `crates/sim/src/verify/reach_tests.rs`) the new
plane set loses 19 part-true `ShankHit` convictions that the all-plane law
makes. The change is less conservative, so it does NOT merge. The work is
held on the draft PR; the issue is `blocked`.

### What was built

Law (c) (`band_reach_counted`, `crates/sim/src/verify/reach.rs`) takes its
model planes from the cut LEVELS plus the deepest cut, in the new
`crates/sim/src/verify/cut_planes.rs`. A cut level is a cutting move whose Z
does not change: `|from.z - to.z| <= z_tol`. The deepest cut is the lowest
`min(from.z, to.z)` over the cutting moves. Nothing else in any law changes;
`Rest::new` still keys a cut per feed chord, and the tangent law and every
column-surface law read those cuts as before.

The level tolerance is `VerifyContext::z_tol` (production `Z_TOL = 1e-6` in
`crates/app/src/operation/material.rs`). A zero tolerance still recognizes a
level move, because an unchanged Z differs by exactly zero.

### Why the change was attempted

An interpolated ramp emits one feed move per thinned vertex, each with its
own Z (`crates/motion/src/links.rs`, `push_zig`). `Rest::new` keys a cut on
`m.to.z`, so a ramp gave about 1,700 planes where the path has 3 cut levels,
and the sample set and verdict walk grew with the vertex count.

### The finding (STOP condition)

Fixture: `reach_tests::ramp_at(500, 36.0)` - 500 feed moves, X 10 to 90, Z 9
down to 1, Y 36, over the 20 mm boss (X and Y 40 to 60, Z 0 to 10) of
`reach_tests::model()`, with a 6 mm cutter and a 10 mm shank 5 mm above the
tip.

- New set (levels plus deepest; the ramp has no level move, so the set is the
  single deepest plane Z 1): `ShankHit { part: true }` at moves 250 to 273
  and 293 to 329.
- All-plane reference (500 per-chord planes): `ShankHit { part: true }` at
  moves 250 to 292 and 293 to 329.
- Lost: moves 274 to 292, 19 part-true convictions. Each is a real collision:
  the shank disc (bottom `to.z + 5`) stands below the boss top Z 10 and
  reaches 1 mm into the boss wall, and the flute did not clear that wall
  point.

Cost measured on the away-from-wall ramp (`ramp_at(2000, 34)`), where the
verdicts do match: new `slices = 1, sample_tests = 0`; old
`slices = 2000, sample_tests = 32,016,000`. The speed win is real; the lost
verdicts disqualify it.

The prismatic boss slice is nominally the same at every Z, but the extra
per-chord planes also place triangulation-tie vertices along the wall at
z-dependent positions. Those points are what catch moves 274 to 292. Sampling
one plane cannot reproduce them.

### Considered and dropped

- **Quantize the plane key and cap the sample count.** The lower-risk variant
  that issue #3615 also names. Not taken: the maintainer's condition is the
  levels-plus-deepest set, and this variant only bounds the damage.
- **Change the tangent law's per-move section plane.** Not taken: it would
  change law (b), not only law (c)'s sample set.
- **Adjust the ramp fixture to match.** Forbidden by the issue's condition 4.
  The fixture and the reference are unchanged; only the `slices` cost
  assertion was relaxed from equality to `<=` (a cost counter, never a
  verdict), and the findings equality stays strict.

### What a fix needs

A plane set that keeps the wall coverage these triangulation-tie points give
while collapsing the duplicate planes - for example, keep the levels and the
deepest cut AND a bounded set of points per wall, or derive samples from the
model's mesh edges directly instead of from plane slices. That is a new
design, not the levels-plus-deepest set.
