## #1727 - Edges as boundaries: any pick mode, and two separate chains bound the area between them

**Decided.**
- An armed boundary row takes faces and edges whatever the toolbar's pick
  mode. Before, it took the toolbar's kinds, so in Faces mode (the usual
  one) no edge could be picked as a boundary.
- The open boundary may hold two or more separate open chains. Each chain
  splits the stock as one chain already did; each keeps the side that holds
  more of the operation's own area; the area kept is where every chain's
  side agrees (between the chains). The open ends run on to the stock as a
  single chain's do. The Open - side B role keeps everything beyond them.
  Face lays its rows along the first chain.
- Refused by name, never guessed: chains that touch or cross; a chain with
  the operation's area evenly on both sides (which side is "between" is a
  guess); a Contour given two chains (it rides one); an include / exclude
  made of separate chains (the Apply door, as for one open chain).
- The old refusal "the boundary picks spell N separate open runs" is gone:
  it is exactly the case the maintainer asked to work. The pick-shape test
  in `area_picks.rs` that pinned it now pins two chains.

**Why.** The maintainer's report: a face selection must be able to be
bounded by two edges, the cut staying between them. The law reuses the
single chain's side construction and region intersection; no new geometry
kernel.

**Considered and dropped.**
- Two separate Open entries (one per edge): the page and the file hold one
  open boundary; a second would change the project file format.
- Per-chain sides (A/B per edge): one side choice for the whole boundary is
  what the page offers; "between" is the case asked for.
- `set_operation_geometry` taking boundary edges: that tool sets the
  operation's source. The boundary is the area controls, set over MCP by
  `set_operation_area` (`open.picks` takes edge chain ids, already did); the
  issue's "Done means" names the first tool, the pin is on the second.
- Drawing the separate chains in the viewport's boundary overlay: the
  overlay draws one open chain; the page lists the entry as
  "2 edges, 2 open chains". Left for a follow-up if wanted.

**Cost to change later.** Small: the law is `camcraft_geometry::boundary_chains`
and one arm in `boundary::resolve`; nothing new is saved.

## PR #1756 review follow-up

**Chosen.** Keep chain grouping, distance checks and region splitting in
`camcraft-geometry`, with geometry unit tests beside them. The app passes
regions and point chains through that interface and maps domain errors into
its operation errors. World lookup and undoable commands stay in app.

**Why.** These calculations have no ECS or project dependency. Keeping the
new algorithms in app was considered and dropped because it violates the
existing domain ownership rule.

**Evidence choice.** Capture the actual egui shell used by the structural
pin for the Boundary row in Faces mode, including its two-chain entry.
Review images live under `docs/reviews/1756`, outside release snapshot
goldens. A painted-text assertion pins the entry without a pixel comparison;
hand-drawn mockups and updating release snapshot PNGs were dropped because
they would not show the reviewed behavior or fit the PR snapshot rule.
