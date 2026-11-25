## #4024 - Path kernel K0: the stage types, the laws as checks, and the Outside start shape

Part of #4023 (the path kernel). This is a design record only. No code
changes with it. Nothing in it is built before the maintainer reviews it:
not the kernel, and not either design of #3934.

### Plain summary

This part has no code names. It says what a machinist sees.

**What the kernel is.** Today each strategy (Pocket, 2D Offset, Circular,
Bore, Contour, Adaptive and the others) has its own code that decides the
passes, how the tool goes from one pass to the next, and how the tool goes
into the material. The same part can get rings in one strategy and
straight slots in another. A fix in one strategy does not reach the
others. The kernel is one set of six steps that every strategy uses:

1. **Area**: what you selected, which side of it is material, and the
   stock and the clamps around it.
2. **Pass shape**: rings that follow a wall, rows, a centre line, or the
   line itself (engrave).
3. **Load control**: none, trochoidal loops, or passes spaced so that the
   tool never takes more than the set load.
4. **Order**: which pass is first (start inside or start outside) and the
   cut direction.
5. **Links**: the tool stays down between passes. It lifts only when a
   link at depth would cut the part, hit a clamp or cross a cut, and the
   status line then says why.
6. **Entry and exit**: in the air outside the stock, in a pre-drilled hole,
   in a column that is already cut, or (only where the tool must go into
   solid material) a helix or a ramp.

A strategy becomes a set of choices for these six steps. It has no
planner of its own.

**What you will see when it is done.**

- "Pass starts at" Inside, Outside and Auto cut the same passes. Only the
  first pass and the order change.
- The tool lifts only where it must. Each lift has a reason on the status
  line.
- The tool never drops straight into solid material without a reason on
  screen. If no safe entry exists, the operation is refused with a
  message that says what to change.
- A cut that takes more than the load limit (for example a full-width slot
  at the start of a ring pass) is either avoided or named on the status
  line.
- When a strategy cannot do what you asked and does something else, the
  status line says so. It never changes silently.
- During the move to the kernel, each step keeps the posted program the
  same. A step that changes the program is a separate fix with its own
  test.

**Decision 1: one pass shape per area, not per start side.** The shape
of the passes comes from the area and the strategy only. A change from
Inside to Outside never makes a different pattern.
- You see: the same rings for Inside and Outside. Outside starts at the
  stock edge, Inside starts at the part.
- Cost: some strategies today make a different pattern for each start
  side. Those patterns change once, and their tests are recorded again
  in the step that changes them.
- Recommendation: do it. It is your rule from #3741.

**Decision 2: the Outside start shape (#3934).** On a band of stock round
a part, the passes follow the stock edge while they are far from the part.
Near the part they take the shape of the part. The last pass is on the
part wall. This shape belongs to the area (a band round a part), so
Inside uses it too, from the part outwards.
- You see: on an Outside start the first passes follow the stock outline,
  not straight lines across the stock. The passes blend into the part
  shape. The tool stays down where the link is clear.
- The way the passes join where they meet the part comes from the
  Adaptive strategy, which already cuts this shape today. It is moved into
  the shared link step, so Adaptive keeps its program and the ring
  strategies get the same joins.
- Cost: Inside on a band also changes its outer passes (they follow the
  stock edge now). A trial in #3934 that made a new join rule gave 35
  lifts on one test part and 24 crossing cuts on another, so the join rule
  is taken from the proven Adaptive code and not written new.
- Dropped: run the Adaptive strategy for every Outside start (then
  Waterline and Offset rings would turn into Adaptive when you pick
  Outside, which breaks "the start side changes only the start"). Dropped:
  keep the old straight-line pattern.
- Recommendation: do it this way, in the order and link step (K4).

**Decision 3: load control changes the pass shape, nothing else.**
Trochoidal loops and constant-load spacing are a change of the passes.
They do not get their own entry or link code.
- You see: a trochoidal pass enters and links by the same rules as a ring
  pass. A cut above the load limit is named on the status line with its
  place.
- Cost: the Adaptive strategy and the trochoidal slot move their own
  entry code into the shared step. Their programs stay the same.
- Recommendation: do it.

**Decision 4: material side and start side are two separate settings.**
"Which side is material" decides where the tool may cut. "Where the pass
starts" decides only the order.
- You see: no new field on the Pocket or 2D Offset pages now. Contour
  keeps its "Side" setting.
- Cost: none for the user now. A later field is possible.
- Recommendation: keep the pages as they are; the kernel holds the two
  values apart inside.

**Decision 5: the P1 fixes do not wait for the whole kernel.** The fixes
for 2D Pocket, Circular and Bore (#3752, #3755, #3756) go on the shared
ring code of #3750 as soon as it is merged. The later kernel steps move
that code to its new place without a change of the program.
- You see: the P1 fixes arrive sooner.
- Cost: one extra move of files later, with no change to the program.
- Recommendation: do it.

The open questions with a recommendation each are at the end of this
record.

### What was decided (technical)

#### Module

`crates/motion/src/path_kernel/`, in `camcraft-motion`. The kernel works
only on domain values (regions, curves, tools, stock), so it belongs in
the motion crate ("`app` holds the document, nothing else"). The name is
not "kernel" alone, because "kernel" already means the geometry kernel and
a heavy lane (#4023).

Planned files (each under 400 lines, each with a provenance line):

| File | Stage |
|---|---|
| `path_kernel/mod.rs` | `Recipe`, `plan_level`, re-exports |
| `path_kernel/verdict.rs` | `Refusal`, `Fallback`, `Gap`, `Verdict` |
| `path_kernel/region.rs` | 1 Region |
| `path_kernel/guide/{rings,rows,medial,trace}.rs` | 2 Guide curves |
| `path_kernel/load/{troch,engage}.rs` | 3 Load control |
| `path_kernel/order.rs` | 4 Order |
| `path_kernel/link.rs` | 5 Link |
| `path_kernel/entry/{air,hole,column,material,exit}.rs` | 6 Entry and exit |

The level ladder (`levels.rs`, the walker over Z levels) stays outside the
kernel. It calls `plan_level` once for each level and emits the moves. The
descent between levels is an entry of stage 6 (`Links::transition`).

#### The strategy is a recipe

```text
pub struct Recipe {
    guide: GuideChoice,       // stage 2
    load: LoadControl,        // stage 3
    order: OrderChoice,       // stage 4
    link: Links,              // stage 5 (the existing policy struct)
    entry: EntryChoice,       // stage 6
}
// One function per strategy, and nothing else per strategy:
fn recipe(op: &<op params>) -> Result<Recipe, Refusal>
```

`recipe` validates early (Prime Directive). A malformed choice is refused
when the recipe is made, never inside a stage.

#### Data flow

The stage numbers stay as #4023 names them. The call order puts Order
before Load control, because the engagement of a cut depends on what is
already cut, and that is the order:

```text
region -> guide -> order -> load -> link -> entry/exit
```

The start side reaches the curves only through `order`. Law 3 is checked
on that seam (see the laws).

#### Common result types

```text
pub struct Refusal { stage: Stage, reason: &'static str, at: Option<Point2>, fix: Option<&'static str> }
pub struct Fallback { stage: Stage, asked: &'static str, used: &'static str, reason: &'static str }
pub struct Gap { area_mm2: f64, at: Point2 }          // unswept cutter-centre area
pub enum Verdict<T> {
    Done { out: T, fallbacks: Vec<Fallback>, notes: Vec<Note> },
    Incomplete { out: T, gaps: Vec<Gap>, notes: Vec<Note> },   // marked "incomplete", never posted silently
    Refused(Refusal),
}
```

Each `reason` is a named `const` (as `ring_links.rs:33-43` does today:
`CUTS_PART`, `HITS_FIXTURE`, `CROSSES_CUT`, `UNCUT_STOCK`, `NO_AREA`).
The app turns each `Fallback` into a status line note (today
`Warning::StrategyFallback`, `crates/sim/src/verify/types.rs:432`), each
`Gap` into the "incomplete" mark, and each `Refusal` into the operation
error. No stage returns a plain value where it substituted a behaviour.

#### Stage 1: Region (K1, #4025)

```text
pub enum MaterialSide { Inside, Outside }       // of the selection's closed loops
pub struct Bounds { stock: Option<Region>, keep_out: Region, top: f64, bottom: f64 }
pub struct KernelRegion {
    center: Region,        // cutter-centre area: tool radius and stock to leave applied
    material: MaterialSide,
    part_walls: Vec<Loop>, // edges the tool must not cross (the selected contour, islands)
    stock_edge: Vec<Loop>, // edges with air beyond them (empty for a closed pocket)
    air: Region,           // reachable air at the cut plane, outside the stock box (may be empty)
    keep_out: Region,      // fixtures, Do not touch
    tool_r: f64,
}
fn region(sel: &Selection, material: MaterialSide, b: &Bounds, tool_r: f64, leave: f64)
    -> Result<KernelRegion, Refusal>
```

- Input: a selection (edges, a face, a chain, or points that straight
  lines join), the material side, the bounds. Today the region is
  `crate::geo2d::Region` from `region_for_job` in the app.
- The signature has no start side. That is law 2 in the type system.
- Invariants: `center` is closed and not empty; the loops are oriented;
  `part_walls` and `stock_edge` partition the boundary of `center`; the
  value is a pure function of its inputs.
- Refusal: an empty, open or self-crossing selection, a tool that does
  not fit, a material side that the selection cannot have (an open chain
  with no side).
- Today's three material-side carriers map onto `MaterialSide`: Contour
  `Side` (`machining2d/contour_params.rs:54`), `ChainSide` and
  `Containment` (`crates/app/src/operation/types/area.rs`), and the
  implicit `StartSide::around_part` (`machining2d/pass_start.rs:98`) of
  2D Offset. `around_part` moves out of `StartSide` into the region in
  K1. `StartSide` then holds only the start side and the stock it reads.

#### Stage 2: Guide curves (K2, #3750, #3788)

```text
pub enum RingRef { Part, Boundary, Band }       // Band: new in K4, see #3934 below
pub enum GuideChoice {
    Rings { reference: RingRef, first_offset: f64, stepover: f64, circle: Option<Circle>, count: Option<u32> },
    Rows { angle: f64, stepover: f64, closing_ring: bool },
    Medial,                       // centre line of a slot or channel
    Trace { offset: f64 },        // the selection itself (engrave, trace, chamfer)
}
pub struct Curve { pts: Vec<Point2>, arc: Option<Circle>, closed: bool, k: u32, family: Family }
pub struct GuideSet { curves: Vec<Curve> }      // a set: no order, no start point
fn guide(r: &KernelRegion, g: &GuideChoice, tol: f64) -> Result<Verdict<GuideSet>, Refusal>
```

- `RingRef::Part` and `RingRef::Boundary` are the two references of the
  #3743 design (result 2, item 1): the selected contour, or all walls
  (Held 1991). `circle` gives exact G2/G3 rings for Circular and Bore.
  `count: Some(1)` is the one ring per level of Contour.
- Invariants: each curve lies in `center` within `tol`; the swept discs
  cover `center` up to 0.1 mm2, else `Incomplete` with the gaps (the
  completeness guard of #3744); the output is a canonical set (sorted by
  `k`, then by family, each closed curve starting at its lowest point);
  the value is a pure function of its inputs.
- Not here: the order of `k`, the start point, the direction. They are
  stage 4.

#### Stage 3: Load control (K3, #4026)

```text
pub enum LoadControl {
    None,
    Trochoidal { width: f64, pitch: f64 },
    ConstantEngagement { load: f64 /* ae/D */, reach: Reach },
}
pub enum Reach { Rings, Stock }
pub struct Controlled { set: OrderedSet, readings: Vec<Reading>, over: Vec<OverLimit> }
pub struct OverLimit { at: Point2, load: f64, reason: &'static str }
fn control(s: OrderedSet, r: &KernelRegion, stock: &StockModel, lc: &LoadControl, tool_r: f64)
    -> Result<Verdict<Controlled>, Refusal>
```

- Load control changes a guide curve (#4023). It never adds its own entry,
  link or lift.
- `None` is the identity: `control(s, .., None, ..).set == s`, and it still
  reads the load and fills `over` (a ring step of full width is named,
  not hidden).
- `Trochoidal` replaces a curve with loops along it (today
  `AdaptiveMethod::Trochoidal`, `machining2d/adaptive_method.rs:9`, and
  `trochoid_slot.rs`).
- `ConstantEngagement { reach: Rings }` re-spaces the rings by the
  engagement law of a circular path and morphs one ring into the next (the
  held design of #3933, Stori & Wright 2000, Held & Spielberger 2009).
  This is Offset rings.
- `ConstantEngagement { reach: Stock }` plans each cut on the in-process
  stock (the HSM planner, `adaptive::plan`). This is Adaptive.
- Invariants: each curve stays in `center`; each cut read above
  `load x (1 + PLAN_TOL)` is in `over` with its place; the stock model is
  the same reader for every mode (`crate::load`).

#### Stage 4: Order (K4, #3750, #3757, #3758)

```text
pub enum Direction { Climb, Conventional }
pub struct OrderChoice { start: PassStart, direction: Direction }  // PassStart: pass_start.rs:19
pub struct OrderedSet { curves: Vec<Curve>, start: Point2, resolved: PassStart }
fn order(g: GuideSet, r: &KernelRegion, o: &OrderChoice, stock: Option<&Region>)
    -> Result<Verdict<OrderedSet>, Refusal>
```

- Auto resolves to Inside or Outside in one place (today
  `ring_side::resolve`, `machining2d/ring_side.rs:50`, #3744). It never
  selects another guide.
- Invariants: the output curves are the input curves, permuted, with a
  chosen start point on each closed curve, and reversed where the side and
  the direction ask for it (the #3744 mirror). Nothing else changes.
- A start side that a guide has no use for (Zigzag rows) is a `Fallback`
  with `PASS_START_ZIGZAG_NOTE` (`pass_start.rs`), as today.
- The chain order of split pieces (#3934, below) is part of this stage.

#### Stage 5: Link (K4)

```text
pub enum Hop { Stay(Vec<Point2>), Lift { reason: &'static str } }
fn link(from: &Cut, to: &Cut, r: &KernelRegion, planned: &[Cut], policy: &Links, tally: &mut LinkTally)
    -> Hop
```

- Stay at depth. A link is `Stay` only with a proof: inside `center` or
  `air`, through cut material only, and across no planned cut (today
  `link_proof` + `through_cut` in `ladder_links.rs`, the ring rule of
  `ring_links.rs`, and the edge route of #3824).
- A `Lift` always has a reason, and the tally (`LinkTally`, `links.rs:142`)
  counts it. The status line prints the tally.
- `Links::Retract` set by the user is obeyed. The status line says so.
- The microlift of Adaptive (`microlift.rs`, `adaptive/lift*.rs`) becomes a
  `Hop` with a short-lift reason, not a separate rule.

#### Stage 6: Entry and exit (K5, #3746, #3934, #3759, #3397)

```text
pub enum EntryKind { Air, Predrilled, ClearedColumn, Helix, Ramp, ArcLead, Plunge }
pub struct EntryChoice { asked: Entry3D, approach: f64, holes: Vec<Point2> }
pub struct Entry { kind: EntryKind, moves: Vec<Move>, proof: Proof }
pub enum Proof { InAir, InHole(Point2), InColumn, IntoMaterial { angle_or_pitch: f64 } }
fn entry(first: &Cut, r: &KernelRegion, cleared: &Region, e: &EntryChoice) -> Result<Entry, Refusal>
pub struct Exit { lead_out: Option<XyArc>, retract: f64 }
fn exit(last: &Cut, r: &KernelRegion, cleared: &Region) -> Result<Exit, Refusal>
```

- The order of tries is the #3743 entry rule: air outside the stock by the
  approach distance, then a G17 arc lead (`air_entry.rs`, #3746); a proved
  pre-drilled hole; a cleared column; a helix or a ramp only where the
  tool must enter material; else `Refusal`. A plunge into material is
  only legal when the user asked for it and the tool can plunge; then the
  `Proof` names it.
- An entry where the user asked for another kind is a `Fallback` (today the
  "axial dive on the floor instead" text of `descent_dive.rs`).
- The exit is the same law reversed: a lead-out into air or cut material,
  then the retract. Today no exit type exists; the exit is the retract in
  `Links`. K5 adds the type with today's moves as its only kind.

#### The #3750 surface, and what later steps rename

#3750 (step 1, in work by another worker now) moves the 2D Offset ring
code into `machining2d/ring_set.rs`, `machining2d/ring_links.rs` (exists)
and `machining2d/ring_entry.rs`, with identical output, and writes
`docs/decisions/3743-shared-ring-planner.md`. The kernel accepts that
surface as it is. The later moves are pure moves (law 7):

| #3750 item | Kernel item | Step |
|---|---|---|
| `ring_set` rings with index k | `path_kernel::guide::rings` | K4 |
| the order of k and the start point inside `ring_set` | `path_kernel::order` | K4 (split, identical output) |
| `RingRef::{Part, Boundary}` | same names; `Band` added | K4 |
| `ring_links` | `path_kernel::link` | K4 |
| `ring_entry` (+ `air_entry`, `predrilled`) | `path_kernel::entry` | K5 |
| `StartSide::around_part` | `KernelRegion` (`part_walls`, `stock_edge`) | K1 |
| `Links::RINGS` seed (#3750 record) | kept as the ring default | - |

Until the move, a later step calls the #3750 names. A step that moves a
file does not change behaviour, and a step that changes behaviour does not
move a file.

### The laws as checks

The K7 suite (#4027) holds these checks in an existing directory suite of
`camcraft-motion` (AGENTS.md "New tests join a directory suite"). Each
check runs over a fixture table: the dog bone of #3741, the disc of
#3934, a square pocket with an island, a narrow neck, and an open-side
band. Each strategy joins the table when it moves to the kernel.

| Law (#4023) | Check | Test shape | Measures |
|---|---|---|---|
| L1 The same region and the same guide choice give the same curves. | `guide` is a pure function, and two strategies with the same guide choice share it. | Property: call `guide` twice, with the selection loops permuted and each loop started at another vertex; and call it through two recipes with the same `GuideChoice` (2D Offset rings and Pocket Waterline with `RingRef::Boundary`). | The canonical curve sets are equal within 1 um (Hausdorff distance per curve, same count). |
| L2 The material side and the start side are separate parameters. | `region` takes no start side; `order` takes no material side. | Compile-level: the signatures. Behaviour: for each fixture, the region hash is the same for Inside, Outside and Auto; flipping the material side of a closed selection changes `center`. | Region equality across 3 start sides; inequality across 2 material sides. |
| L3 The start side changes only the order and the start point. | `order` permutes, picks start points and reverses. Nothing else. | For each fixture and each start side pair: compare the curves of the plan as undirected point sets. With load control on: the `guide` output is equal, and the `control` output equals `control(order(..))` (the side reaches the curves only through order). | Curve multisets equal within 1 um; only the permutation, the start point and the direction differ. |
| L4 A fallback is visible. The status line names it. | Every `Fallback` becomes one note. | Run each strategy on fixtures that force a fallback (a narrow neck, Zigzag with a start side, a blocked arc lead). | Count of `Fallback` records = count of named notes on the operation; no plan whose used kind differs from the asked kind without a note. |
| L5 An entry into material needs a proof. Else the kernel refuses with a reason. | Every entry has a `Proof`. | Read the posted CL on an independent stock grid (the simulator stock, not the planner's). For each move that goes down to a cut Z: classify its XY as air, a pre-drilled hole, a cleared column, or material. | Count of entries into material with no `IntoMaterial` proof = 0; a plan with no legal entry is `Refused`, with the reason text. |
| L6 A cut above the engagement limit is prevented or named. | `control` reads each cut and fills `over`. | Read each cut of the CL on the independent stock grid (the contact angle law of `crate::load`). | Each cut above `load x (1 + PLAN_TOL)` has an `over` note within one tool radius of its place; full-width cuts (180 degrees) outside a named slot = 0. |
| L7 A move that migrates keeps its output. A defect fix is a separate change. | Equivalence pins of #3743 result 2, section 5. | A move step: golden `.cl` recorded on the head before the step. A P1 fix step: ring radii and verdicts. | Move: identical CL moves, same verifier verdicts, `#[test]` count not lower. Fix: radii within 1 um; only the named counts change, each with a red-first pin. |
| Link rule (#4023 stage 5) Stay at depth. Lift only with a named reason. | Every lift is in the tally. | Count the Z-up moves between two cuts of one level in the CL. | Lifts = sum of the tally; on fixtures with no split ring, lifts = 0; crossings (the verifier's Crossing law) = 0. |

### The Prime Directive in the kernel

- A stage that cannot meet a law returns `Refused` or `Incomplete` with a
  named reason. It never returns a plain result that hides a lift, a
  plunge, a gouge or a full-width cut.
- Each guide curve and each link is inside `center` or `air` by proof, so
  no stage can gouge the part by construction. The verifier (gouge,
  collision, Crossing) still runs on the output and still blocks the post.
- `Incomplete` is the "incomplete" operation state: the path shows, the
  post is blocked, the gaps are named.
- No `unwrap`, `expect` or `panic!` in the kernel. Degenerate input is a
  `Refusal` from `region` or `recipe`.

### #3934: the Outside start shape, settled in kernel terms

The maintainer answered "Decide in design" (#3934, 2026-11-25).

**The problem.** On a band of stock round a part, the old one-ring plan
(the #3744 ring set: offsets of the part, clipped by the stock box) cuts
the far corners of the band as straight clipped pieces: the pattern of
#3934. It has 0 lifts and no crossing. The issue asks for passes that
follow the stock edge far from the part and blend into the part shape.

**The two designs of #3934 and the readings.**

1. Use the Adaptive air plan (`adaptive::plan_from_air`,
   `machining2d/adaptive/air_rings.rs:74`) on each Outside start:
   stock-parallel fronts, links in cut material, the part wall last.
2. Keep the ring set and add a chain order and a link law for the split
   pieces (a port of the air plan's chain and links to the level planner).

The held trial (branch `issue/3934-outside-stock-edge`) built a new blend
of the two edges with its own joins:

| Fixture | Held trial | Old one-ring plan |
|---|---|---|
| disc, Outside | 35 lifts, 35 re-entries | 0 lifts, no crossing |
| dog bone, Outside | 22 lifts, 24 crossing planar feeds (the post is blocked) | 0 lifts, no crossing |

No reading of design 1 on these two fixtures exists. Its shape is pinned
on its own fixtures (`pocket_hsm_start_tests.rs`,
`outside_cuts_the_band_from_the_air_and_ends_on_the_part_wall`).

**Each design against the laws.**

| Law | Design 1 as written | Design 2 as written (held trial) |
|---|---|---|
| L1 same region and guide, same curves | Fails: Waterline and Offset rings get Adaptive curves on Outside only. | Holds. |
| L3 start side changes only order and start | Fails: the start side selects another planner, spacing and load control. | Holds if Inside uses the same blend. |
| "A strategy has no planner of its own" | Fails the other way: one strategy borrows another strategy's planner. | Holds. |
| Link rule | Holds (proven joins). | Fails in practice: 35 lifts; 24 crossings block the post. |
| L5 entry proof | Holds (entry in air). | Holds. |
| L6 engagement | Holds (stock read). | Not read. |

**Decided.** The shape is a guide choice of the region. The joins are
the kernel's order and link stages, taken from the proven air plan code.

- Guide (stage 2): a new reference `RingRef::Band` for a region with a
  `stock_edge` and `part_walls` (a band round a part). Its curves are the
  air plan's fronts with load control `None`: fronts parallel to the stock
  edge at the stepover while they stay clear of the part, then offsets of
  the part, the part wall ring last. Spacing is the stepover, not the
  load, so Waterline stays Waterline.
- The choice depends on the region, never on the start side. 2D Offset and
  Pocket ring strategies use `Band` on a band round a part, for Inside,
  Outside and Auto. Outside walks it from the stock edge; Inside walks the
  same curves from the part wall out (law 3).
- Order (stage 4) and link (stage 5): the air plan's chain of split
  fronts and its link proof move into `order` and `link` as the one law
  for split pieces. Adaptive keeps its program byte for byte (law 7: the
  move is its own step); the ring strategies get the same joins.
- Adaptive is then `Band` + `ConstantEngagement { reach: Stock }` on the
  same stages: one shape family, two spacings.
- Acceptance pins (K4): on the disc and the dog bone, Outside and Inside,
  0 crossings, every lift in the tally, and the lift count reported in the
  PR. The target is 0 lifts on the disc, as the old plan has.

**Dropped.**
- Design 1 as written: it breaks L1 and L3, and Waterline and Offset rings
  would silently become a load-spaced Adaptive cut on Outside.
- Design 2 as written: a new join law in the ring planner. The held trial
  shows it is not ready (35 lifts, 24 crossings), and the air plan already
  has a proven one.
- Keep the old straight pieces: they are the reported defect.
- `Band` on Outside only: it breaks L3.

**Cost.** Inside on a band changes its outer rings (they follow the stock
edge). The #3744 pins on bands are recorded again in the K4 step, with
the change named. #3934 stays `blocked` until this record is reviewed,
then it is worked as a K4 step on top of the K4 move.

### Map of the present operations

| Operation | Region (material side) | Guide | Load | Order | Link | Entry | Today | Step |
|---|---|---|---|---|---|---|---|---|
| 2D Offset | band round the part (Outside) or the area (Inside) | Rings Part; Band round a part | None | start side | ring links | air entry, arc lead | #3744-#3748 done; #3750 | #3750, K1, K4 (#3934), K5 |
| 2D Pocket Waterline | pocket (Inside) | Rings Boundary | None | start side | ring links | entry rule | `waterscan.rs`, `level_rings.rs` | #3752 (P1) |
| 2D Pocket Offset rings | pocket (Inside) | Rings Boundary | ConstantEngagement Rings | start side | ring links | entry rule | spacing at load x D; #3933 held | #3752, then K3 (#3933) |
| 2D Pocket Zigzag | pocket (Inside) | Rows, closing ring k=0 | None | start side = note | link | entry rule | `pocket_zigzag.rs` | #3753 |
| 2D Pocket Auto | pocket | Waterline or Zigzag by shape, named | - | - | - | - | `PocketStrategy::resolve` | #3752 |
| 2D Pocket Adaptive, 2D Adaptive Spiral | pocket or band | Rings Boundary or Band | ConstantEngagement Stock | start side | link (microlift as a hop) | air plan, bore, helix | `adaptive/`, `pocket_hsm*.rs` | K3, K6 (#3754, #3759) |
| 2D Adaptive Trochoidal | pocket | Medial | Trochoidal | start side | link | entry rule | `adaptive_troch.rs` | K3 (#3932) |
| Rest machining | rest area (Inside) | as the strategy | as the strategy | start side | link | entry rule | `rest_rings.rs` | #3753 |
| Circular | disc (Inside) | Rings with circle | None | start side | ring links | entry rule (helix in the core) | `circular.rs`, a walk per ring | #3755 (P1) |
| Bore | disc (Inside) | Rings with circle | None | start side | ring links | entry rule | `bore.rs`, 256 chords | #3756 (P1, after #3755) |
| 2D Contour | Contour `Side` | Rings Part, count 1 per level | None | direction | link | entry rule (wall arc) | `contour*.rs` | #3757 |
| Trace, Engrave, 2D Chamfer | on the line | Trace | None | direction | link (proof refuses a link across the engraving) | entry rule | `slots.rs`, `chamfer.rs` | #3758 |
| Face | stock top (Inside) | Rows | None | direction | link (hop over a hole) | air lead | `face.rs`, `face_entry.rs` | #3759 |
| 3D Pocket levels | the level slice (Inside) | as the strategy, per level | as the strategy | start side | link | entry rule | `machining3d/level_side.rs`, `hsm.rs` | #3754 (#3616), #3760 (P7) |

Not in the kernel: Thread mill (its own schedule) shares only the entry
rule and the lift reason. 3D finishing, multi-axis, turning, sheet and EDM
are out of scope (#4023).

### Migration order

```text
K0 #4024 (this record)
 |-- #3750 step 1: ring_set / ring_links / ring_entry, identical output (in work, parallel to K0)
 |     |-- #3752 Pocket (P1) --> #3753 rest and Zigzag
 |     |-- #3755 Circular (P1) --> #3756 Bore (P1)
 |     |-- #3757 Contour, #3758 Trace/Engrave/Chamfer
 |-- K1 #4025 region: create path_kernel/, region.rs, MaterialSide; around_part moves in
       |-- K4 order and link: move ring_set/ring_links in (pure move), split order;
       |      then #3934 (Band, the air plan joins) as a separate fix
       |-- K5 entry and exit: move ring_entry/air_entry in; #3759; #3397; Exit type
             |-- K3 #4026 load control: None/Trochoidal/ConstantEngagement;
             |      #3933 (resumes), #3932, #3849 (PR #4008 stays its own fix)
             |-- K6 the strategies: #3754 (Adaptive fallback, 3D levels), #3760 (P7)
K7 #4027 grows with each step from K1 on.
```

Dependencies: K4 and K5 need K1 (the region type). K3 needs K4 (load
control reads the ordered set). #3934 needs the K4 move. #3933 needs K3.
The P1 steps #3752, #3755 and #3756 need only #3750 (Decision 5).

### Why

- The maintainer's words (#4023): "Well we need a unifying approach to this
  planning and it calls for a motion planning kernel that dictates how
  motion can be implemented."
- #3741: "The only thing that differs and inside and outsided strategy is
  actually where the tool starts.." and "Where it should have just followed
  the contour keeping its plan not lifting the tool."
- #3743 result 2 found eleven ring planners, ten linkers and ten entry
  choosers. The defects of #3741, #3755, #3756 and #3934 come from those
  copies. One contract with checks closes the class, not one case.

### Dropped (other choices)

- The kernel under `machining2d/`: the 3D Pocket levels use it too.
- Load control as its own stage with its own entries (trochoidal as a
  strategy): #4023 says it changes a guide curve; a second entry rule is
  the defect class.
- The #4023 call order (load before order): the engagement depends on the
  cut order, so the load stage would have to guess it.
- A new material side field on the Pocket and 2D Offset pages now: no
  user asked for it; the kernel keeps the values apart inside.
- Wait with the P1 steps for K1-K4: P1 defects would wait for a refactor.

### Cost to change later

The stage types are internal to `camcraft-motion`. The project file does
not change in K0-K5, except where a later step adds a user field. A
different stage split changes the K7 checks and the recipes, not the
operations' pages.

### Open questions for the maintainer

1. **#3934 Outside start shape.** Confirm: one band ring shape (stock edge
   fronts that blend into the part shape) for Inside and Outside, with the
   Adaptive join law moved into the shared link step. Recommended: yes.
2. **Inside on a band changes too.** Its outer passes then follow the
   stock edge (a consequence of "the start side changes only the start").
   Recommended: accept it.
3. **P1 fixes before the kernel.** Let #3752, #3755 and #3756 go on the
   #3750 code at once, and move the files later. Recommended: yes.
4. **Module place.** `crates/motion/src/path_kernel/`. Recommended: yes.
5. **Material side field.** No new page field on Pocket and 2D Offset now;
   Contour keeps "Side". Recommended: no new field.
6. **Trochoidal is load control, not a strategy.** Adaptive Trochoidal and
   the trochoidal slot become a centre line plus trochoidal load control,
   with the shared entry and links. Recommended: yes.
7. **Offset rings and Adaptive stay two names.** Offset rings = rings
   spaced by the load law (#3933); Adaptive = each cut read on the stock.
   Recommended: keep both.
8. **Above the load limit: name or prevent.** A strategy without load
   control names the cut (status line); a strategy with load control
   re-plans it, and names what it cannot re-plan. Recommended: yes.
9. **Exit.** Exit is part of stage 6 (lead-out and retract), not a stage of
   its own. Recommended: yes.
