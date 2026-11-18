## #2287 - Accept part alignment on verified non-mill machines

**Chosen:** Replace the blanket `Kind != Mill` refusal in `setup_align::generation_problem`
with a per-machine capability check (`verified_for_machine`) matching each machine kind
to its proved operation kinds:
- `Kind::Lathe`: all 12 turning kinds (`Facing`, `ProfileRough`, `ProfileFinish`,
  `GrooveRough`, `GrooveFinish`, `Part`, `Threading`, `Chamfering`, `TaperTurning`,
  `MultiPassTurning`, `FormTurning`, `BoringTurning`) and `Drill` (axial and cross).
- `Kind::MillTurn`: the 17 verified mill kinds plus the 12 turning kinds; `Probe`,
  5-axis/rotary kinds, and `tilt().is_some()` stay refused.
- `Kind::Plasma`: `PlasmaProfile`.
- `Kind::WireEdm`: `WireContour` (2-axis, taper, and 4-axis ruled); wired
  `operation::wire_ruled` picks through `setup_align::pose_in`.
- `Kind::SinkerEdm`: `SinkerBurn` and `SinkerSideBurn`; wired
  `operation::sinker_faces`, `ElectrodeFromFaces::new_in`, `mcp::sinker_tools`, and
  `pick::align::outward_normal` through `setup_align::pose_in` and `facets_in`.
- `Kind::Waterjet`: keeps the machine-kind refusal because Waterjet has no operation
  kinds or post yet to prove against a baked mesh.

Updated `an_aligned_setup_on_an_edm_names_the_edm_kind_it_is_refused_on` in
`crates/app/tests/setup_align.rs` so foreign milling `Contour1` on EDM asserts the
per-machine aligned-setup refusal while `Waterjet` owns the machine-level refusal.

**Why:** Most non-mill generators already consume `OpFrame::material_of` (`facets_in`)
and `part_aabb_in`, so blocking them only hid two remaining document-frame reads:
4-axis wire profile picks in `wire_ruled.rs` and cavity electrode derivation in
`sinker_faces.rs`. Single-setup part alignment rotates the part onto the spindle,
table, or guide plane independently of multi-setup carried stock (`stock_sim_carry`
remains refused on lathe/mill-turn per #321). Each machine/kind pair is enabled only
alongside a turned-by-two-axes vs baked-mesh CL and NC equivalence proof (`#2276`).

**Dropped:** Unlocking `Waterjet` without operations or a post to prove it against,
or unlocking `Probe`, 5-axis/rotary, or tilted ops on `MillTurn` ahead of #2285/#2286.

**Cost to change later:** Enabling `Waterjet` when its operations ship requires one
arm in `verified_for_machine` plus a turned-vs-baked proof test. Any future non-mill
operation that reads picks or facets outside `OpFrame` must go through
`setup_align::{pose_in, facets_in}` rather than `model::transform_of` or `world_facets`.
