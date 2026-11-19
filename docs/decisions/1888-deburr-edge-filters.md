## #1888 - Deburr: picked edges, excluded edges, minimum edge angle, hole filter

**What was decided**

1. **Picked chains narrow the ride; they do not bound a region.** A Deburr
   whose source is a selection breaks only the exposed edges on the picked
   edge chains. `OpParams::Deburr` stays in the `selection_drives_region`
   list of mesh-driven kinds. The picks are read by a new module,
   `operation/deburr_picks.rs`, the way Multi-Axis Contour reads its drive
   chains. A source that is a whole model, the stock, or an empty selection
   keeps the automatic detection.
2. **A pick selects among exposed edges only.** A picked chain that lies on
   a concave or flat meeting is not ridden. The path has a note for it. If
   no pick names an exposed edge, generation refuses by name.
3. **A pick is not filtered.** The minimum edge angle and the hole filter
   judge the automatic edge set only. An excluded chain is removed in both
   modes.
4. **A pick that is not an edge chain is refused by name**, at generation
   and at the MCP pick door (`kind_door`). `exclude_edges` holds chains
   only; `is_sane` refuses a face, a patch or a vertex there.
5. **New operations get 40 degrees; old files get 0.** The seed sets
   `min_edge_angle_deg` to 40.0. A project file without the key reads 0.0
   (every crease), which is the ride the file stored. Each new key is absent
   from the file at its rest value, so an old file saves unchanged. The
   operation stamp digests a filter only off its rest value, so an old
   operation keeps its stamp and its cached path.
6. **The hole filter matches the bore's own cylinder.** A recognized hole
   (`drilling::recognize`, `recognize_cross`) whose diameter is in the range
   removes each exposed edge with both ends on its cylinder, within 0.05 mm,
   inside the wall's span. The seed range is 0 to 10 mm, with the filter off.
7. **The picked and the excluded chains share the job's `drive_chains`
   slot.** The source's chains come first; the pick count of the source
   says where the list splits. The WCS yaw then turns both lists with the
   facets, with no new slot.
8. **The UI moves picks into the exclusion list.** The Deburr page has the
   button "Exclude the picked edges". It adds the edge picks of the
   Selection block to the draft's `exclude_edges`. Apply writes them with
   the other parameters, as one `EditOperationParams` command.

**Why**

- The issue text says to remove Deburr from the `!selection_drives_region`
  guard. That guard sends the picks to `selection_region`, which makes a 2D
  area from them and refuses an open run. A picked edge is a path, not an
  area. The Multi-Axis Contour pattern gives the behaviour that the issue
  asks for ("`SrcRef::Selection` drives manual edge chains").
- A vertical cone over a concave corner cuts the two faces. A pick must
  not make that path (Prime Directive).
- A silent change of the edges that an old project breaks is a change of
  posted G-code. The 0.0 default for old files prevents it.
- Before this change, a Deburr with a selection source ignored the picks
  and rode every edge of the picked models. That was a silent wrong answer
  for a user who picked one edge.

**Considered and dropped**

- Removing Deburr from the guard as written: refused open chains and gave
  no path.
- 40.0 for a file without the key: changes the path of each stored Deburr
  on a mesh with shallow creases.
- Applying the minimum angle and the hole filter to picked chains: a pick
  is an explicit instruction.
- A separate job slot and a new `WcsYaw::enter` parameter for the excluded
  chains: more plumbing for the same result.
- A new command for the exclusion list: the list is an operation
  parameter, and `EditOperationParams` already makes it undoable. MCP sets
  it through `edit_operation`.
- A pick mode in the viewport that writes to the exclusion list directly:
  larger UI work; the button uses the existing Selection picks.

**Limits**

- A stepped hole (counterbore, chamfered mouth) is recognized by its narrow
  bore. The wider ring's rim is not on that cylinder and stays in the ride.
- A rim of an open mesh has no bend, so the minimum angle does not remove
  it.
- A behaviour change: a Deburr whose stored source is a selection of faces
  now refuses by name. Before, it rode every edge of the picked models.

**Cost to change later**

- The 0.0 file default is a file contract. A change needs a migration.
- The shared `drive_chains` slot is local to `job.rs`, `generate.rs` and
  `deburr_picks.rs`. A separate slot is a small refactor.
