## #3679 - A feed plunge deeper than one stepdown below the uncut floor is named

Taken unattended, as the recommended option under the owner's standing
word (Autonomous Work Law). The issue's Expected offers two forms: "The
finding blocks the post, or it is a named warning that the user must
see." Taken: **the named advisory form**.

**What was decided.**

- A new verifier law, `camcraft_sim::verify::plunge_depth` (law (d3)),
  judges every non-rapid axial descent (feed or plunge, XY within
  `z_tol`, descending) against the rest model's floor: for each disc
  sample of [`rapids::footprint`] at radius `r - tightened(gouge_tol)`,
  the depth is `rest.surface_before(q, i) - (to.z + cutter.lift_at(rho))`
  where stock stands (`rest.in_stock`). The move is named once, at its
  worst sample, when `!(depth <= limit + z_tol)` with `limit` the op's
  stated stepdown. The `!(.. <= ..)` form fails closed: a non-finite
  floor, a non-finite stepdown or a NaN depth is a conviction, never an
  acquittal. `Mode::Revolve`, `Mode::Radial` and `Mode::Rotary` stand
  the law down (law (d)'s own words: the XY column is not those paths'
  geometry).
- The gate is the new `VerifyContext::plunge_stepdown: Option<f64>`,
  the same `None` discipline as `finish_tol`, `model`, `spindle` and
  `lathe_axis`: `None` keeps the law silent and every stored verdict
  byte-identical. The operation layer
  (`crate::cutting_rows::plunge_stepdown`) sets it for the kinds that
  cut in vertical levels and state a stepdown: 2D Contour, Pocket,
  Offset2D, Slot, TrochoidalSlot, Adaptive2D, Pocket3D, MultiAxisRough
  and the staged Face. Drills, bores, threads, the turning, wire and
  plasma families keep `None`. A tilted (3+2) frame keeps `None`: the
  law reads world-Z descents, and a tilted plane's descents answer to
  its own entry method's law.
- The finding is `Warning::PlungeTooDeep { at, mm, limit_mm }`, MCP
  kind `plunge_too_deep`, UI line 227 - **advisory**, in the
  `blocks_post` exemption list beside `FluteShort`.

**Why advisory, not blocking.**

- The maintainer's P3 re-triage: "Nothing gouges or collides, and the
  posted part geometry is correct. The risk is the tool load." This is
  the #3525 pattern exactly - there the owner's standing word chose
  "the diagnostic exists to name the cause; the collision checks decide
  if the post is held", and `FluteShort` (a cut deeper than the flute)
  is advisory for the same load-class reason.
- The generator still produces these dives at every tabbed contour
  window with a lead (#3678 owns the fix; PR #3684 pinned the later
  windows' 18 mm crescent dives on purpose). A blocking law would hold
  the post of every such program - including the example catalog case
  `2d/contour-tabs`, whose contract test regenerates and posts the
  committed NC - until #3678 lands. The Done means asks that the
  catalog case "gives the same finding until #3678 is fixed", not that
  it stops posting; the advisory form gives the finding with no red
  suite and no known-fail marker to remove later.
- The crescent case convicts on its own reading either way: at the
  dive station the samples beyond the swept band read the stock top
  (6.0 mm above the last level of the catalog case against its 2.0 mm
  stepdown), the samples inside it read the level above (one stepdown,
  clean), so the worst sample names the crescent exactly as the
  maintainer narrowed it - centre-cutting or not, the full-disc depth
  engagement is the reading, per the issue's own words.

**Considered and dropped.**

- **Blocking** (the context pack's first form): dropped for the
  reasons above. It would also have required known-fail markers
  (#2567) on `catalog_2d_contour_tabs::{regenerates, rebuilds}` and on
  `contour_whole_step`'s clean-verdict cases, all pinned to #3678 and
  all to be removed again by its fix.
- A generator-side peck guard in this PR: #3678 owns it; this PR is
  the verifier law only (the coordinator's scope split,
  `docs/decisions/3453-tabbed-contour-entry.md`).
- Judging the descent's full swept column (stations along a nearly
  axial move): a descent within `z_tol` laterally is one column in
  practice; the lateral and ramped feeds are the level law's, not
  this law's.
- A mode gate for `Undercut`: dropped - `Rest` takes no cut there, so
  the floor is the stock top and an undercut Z-dive names toward
  safety; the app wires no undercut kind, so the branch runs only in
  hand-built contexts.
- Wiring `Circular` (the ring pocket family): dropped with the context
  pack's list - `steps_mut` does not seed it, and the ring family's
  own hops dive in their own kerf; a later PR can add it with its own
  pinning tests.

**Limits.**

- An over-deep dive still clears its own column in `Rest::cut`, so a
  later re-entry above the dive bottom reads that column as cleared.
  Whether an unsafe dive should stand as a proven floor for the later
  laws is filed as its own follow-up issue (P7).
- The reading is bounded by the footprint's own sampling bound (law
  (d)'s: stock between two samples by less than a ring spacing can
  pass), the same bound every disc law here shares.
- The tilted 3+2 frames stay unnamed until a pose-aware reading exists.

**Cost to change later.** The variant name `plunge_too_deep`, its
fields and the context field `plunge_stepdown` are part of the project
file and of the MCP output from this PR on. To make the finding block
the post, remove it from `blocks_post`'s exemption list and mark the
catalog case's suites known-fail until #3678 lands.
