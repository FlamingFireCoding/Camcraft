## #2004 - Depth-stepping operations take no level at Top Z

**Decided:**

1. Top Z of a depth-stepping operation is the top of the material that
   the operation cuts. It is never a cut level. The shared level law
   (`camcraft_motion::machining2d::cut_levels`) starts at
   `top - stepdown` and ends at `bottom` exactly. `top 20, bottom 0,
   step 5` gives `[15, 10, 5, 0]`; `top 13, bottom 0, step 50` gives
   `[0]`. Each level below the first is the same float as before (the
   same subtraction chain), so a changed program loses only its first
   level.
2. One law: `cut_levels` is now `pub`, and the app's `ladder_planes`
   (the pocket region build) calls it. It no longer restates the loop.
   A refusal of the law (stepdown not positive, too many levels, NaN)
   gives the one bottom plane there, as before; the generator refuses by
   name later.
3. Top Z below the stock top (material above Top Z, round 2 finding):
   fixed in round 3. `Approach` gets the runtime field `material_top:
   Option<f64>` (default `None`, never in the project file). The app
   lays the setup's stock top over it at generation (`generate.rs`,
   `stock_top_z`, in the generator's frame), and the region planes read
   the same top (`area.rs`, `material_top`). The law
   `cut_levels_under(top, material_top, bottom, step)` keeps the level at
   Top Z when the material top stands over Top Z: that is the old ladder,
   byte for byte, so each bite stays at or under one stepdown. `None`
   or a material top at Top Z gives the new ladder. The stock top is the
   raw setup stock: an earlier facing operation is not read, so that
   case keeps the old (one air level) answer. That errs to the safe side.
   Face (`FaceParams::stock_top`) and the vault rim ring
   (`links.approach.stock_top`) follow the same law since the review of
   head `eacc74c78` (see the last section). The trace and trochoidal
   slot are on the remaining list.
   Dropped: a new field on each params struct (about 350 literals); an
   app shift of Top Z by one stepdown (float noise, and the plunge
   starts higher); a thread-local.
4b. Vault: a plane at Top Z is dropped. The rim ring's floor is the bowl
   at the radius, which is Top Z, so the rim ring cut air at Top Z under
   the old and the round-1 ladder; the first inner ring opened at that
   same plane. A ring with no plane under Top Z is not walked.
4c. A ring, a hole or an overhang whose only legal level was Top Z
   (a stepdown past the hole floor or past the slab top) is now refused
   by name ("nothing to machine"). Before, the operation posted one air
   pass at Top Z there. A level at a flat floor between ladder levels is
   a separate enhancement.
4d. Bore contour, helix transition (`one_descent`, case "bore lead none
   Ramp helix transition"): a real rapid into stock, fixed in round 3.
   Hole radius 10, tool diameter 6, top 0, stepdown 2, levels [-2, -4,
   -6]. The ring at -2 sweeps only the band r 4..10; the core r < 4 stays
   at Z 0. The level transition rapided to Z -2 at r 6.55, and its disc
   (r 3.55..9.55) stood 0.45 mm into the uncut core over Z -2..0. The
   old ladder started the first transition at Top Z, so it opened the
   column from the top. Now a transition whose column no earlier descent
   opened (`BoreStep::opened`, `Step::opened`) starts at the top of the
   cut plus the plunge height (Z 2), and later transitions reuse the
   opened column at the plane above. Verify is not loosened. Edge not
   exercised: a helix that changes its radius between levels.
4. Circular, vault mode: the rim ring starts one stepdown under Top Z.
   An inner ring keeps the floor of the ring before it as its first level
   (`levels_from` in `circular.rs`). That plane is not a repeat: the
   strip inside the previous ring's band has stood at the stock top until
   then, and it opens at that plane. To drop it would deepen that ring's
   first cut by one stepdown.
5. Turning face: its station ladder is anchored on the facing plane, not
   on `cut_levels`. A rung exactly at the top of the work (the span an
   exact multiple of the stepdown) has its whole ball band above the work.
   That rung is dropped. A rung at a Top Z below the work's top has
   material over it and stays.
6. Face with no Top Z (the single pass at Cut Z) stays one pass. A span
   of zero stays the one plane at Bottom Z. A stepdown of zero stays
   refused.
7. The last step keeps the remainder (`[15, 10, 5, 2]` for bottom 2).
   Equal steps are a separate choice, filed as a follow-up issue.
8. Rotary wrap rough, a station window that holds only air under the
   wrap top (round 4): the refusal "nothing to cut" is correct. With no
   stock, the wrap top is the skin maximum (rho 2.5 on the test bar). In
   the windows v 0..8 (foot, rho 2.0) and v 19..28 (cone top and barrel)
   the old ladder `[2.5, 1.5, 0.5]` had open area only at 2.5, where the
   tip stands on the stock skin. That program cut only air. The test now
   pins both windows as refusals by name, and asserts the station law on
   windows with material (v 0..12 and v 10..28). Dropped: to keep an air
   level so that the window does not refuse - it posts a program that
   removes nothing. Not changed: the ladder does not reach the material
   between rho 2.0 and 2.5 on the foot (levels 1.5 and 0.5 are in the
   model). That was so before this change too.
9. Trace, engrave and trochoidal slot (round 5): `TraceParams` and
   `TrochoidParams` get their own field `material_top: Option<f64>`. The
   app fills it from the same `lk.approach.material_top` as the other
   operations. The trochoid lanes of 2D Adaptive (Trochoidal) copy it from
   their pocket's links. Dropped: a full `Links` field on the two param
   types - the two generators read no other link, and a `Links` field
   promises controls that they do not have.

10. Round 7, the red quick gate on `185a69f1e`. Each failure, by class:
    (a) the expectation names only the lost air level at Top Z, so the
    expectation changes and every other assert stays; (b) a real
    regression, fixed in the code.
    - (b) `blind_floor::random_stepdowns_and_floors_never_cut_below_a_floor`
      case 4 (floor 16.108 over the first level): the contour depth
      ladder (`contour_depth::ladder`) searched a ring's floor only under
      a level that the ring walked. The old ladder always walked Top Z
      first (in air), so a blind hole shallower than one stepdown got its
      floor; now its first level failed and the ring cut nothing. Fix:
      when the first level fails and the ring is clear above Top Z, the
      floor search runs up to Top Z. New pin:
      `a_blind_hole_shallower_than_one_stepdown_is_cut_at_its_floor`.
      This corrects item 4c for contour rings: a ring with a floor between
      Top Z and the first level is cut at that floor, not refused.
    - (b) `area_open::a_containment_that_puts_the_tool_in_the_part_is_refused_by_name`
      and, in `contour_outside_hole`,
      `outside_on_a_picked_hole_wall_is_refused_by_name_at_every_depth`
      and `inside_on_a_picked_block_top_is_refused_by_name`: with no air
      level, a wrong-side picked ring walks no level, so the generic
      "every contour ring cuts into the part at its first level" refusal
      came first and hid the #1927 named refusal. Fix: `picked_law` runs
      before the empty-ladder check (`contour.rs`). The named text, the
      side advice and "part top Z 10.000" are unchanged, and the four
      depth cases (full, patch, selection heights, top over the part)
      still refuse.
    - (a) `blind_floor::a_blind_hole_ring_ends_at_its_floor_and_the_outer_wall_goes_to_the_bottom`,
      `a_through_hole_ring_keeps_every_level`: the stock top is Top Z 20,
      so the level at 20 goes. The floor, the skip notes and the deepest
      move asserts are unchanged.
    - (a) `contour_outside_hole::inside_and_auto_on_the_hole_wall_cut_every_level_clean`,
      `outside_on_an_outer_wall_and_on_a_boss_stays_legal`: the part top
      is Top Z 10 (no stock), so the level at 10 goes.
    - (a) `drill_prior::a_stock_facing_allowance_is_faced_and_the_drill_after_it_stands`:
      the fifth pass had its nose tip at the stock end Z 22 (item 5). The
      four passes remove the full 2 mm: the test now also asserts the last
      pass at the finished end, the first bite one stepdown, and each bite
      at most one stepdown.
    - (a) `post_engine::every_golden_is_a_legal_post_input`,
      `slot_fullwidth_centerline.cl` with 9 records: the golden still
      feeds the full row at Z -3 and Z -6 (bottom -6). Only the record
      floor of 10 failed. That golden's floor is 8, and the test now
      asserts its two full-depth rows.
    - (a) `explain_tormach::tormach_explains_every_conformance_golden_without_changing_the_program`:
      the `explain_*.nc` goldens were not written again in round 6. Each
      one loses the same lines as its plain twin (188 lines in all).
    - (a) `first_router_part::first_router_part_project_loads_verifies_and_posts_to_grblhal`:
      `examples/first-router-part.camproj` holds cached toolpaths of the
      old ladder. It is now written from the test's command build under
      `WRITE_GOLDEN=1` (as #1917's decision says). The diff removes 37
      moves and changes the stamp; the verification gate stays silent.
11. Round 8, the 18 red app tests after round 7. Each one is class (a):
    the test kept the old air level at Top Z. No code changed in this
    round, and no golden changed.

    | Test | Why (a) |
    |---|---|
    | `operations::an_open_run_contours_on_its_air_side` | part top 5 is Top Z; levels `[2.5, 0]` |
    | `operations::a_mixed_pick_list_cuts_boundary_then_run...` | each kind loses its level at Top Z; plunges 6 to 4 |
    | `operations::contour_outside_forces_outside_on_bore` | still refused; the #1927 named text comes first (item 10b) |
    | `project_roundtrip::side_choice_round_trips_and_a_hand_edited_inside_refuses_at_generate` | still refused; asserts the #1927 named text and the side advice |
    | `predrilled_entry` (2 tests) | stock top 20 is Top Z; plunges `[16, 12]`; no descent onto Z 20 |
    | `remachining` (2 tests) | the unmasked marker moves from Z 20 to Z 15; the masked asserts also forbid Z 15 |
    | `stitch_cross` (2 tests) | stepdown 10 = the span, so the ladder is `[0]`; the ring across two boxes is still cut at Z 0 as the one-block reference; the self-touching body is still refused by name |
    | `turning_face_approach::every_facing_retract_lifts_z...` | the pass at the stock end Z 16 is air (item 5); passes `[15.5, 15]` |
    | `turning_face_seed::a_seeded_facing_faces_the_allowance_not_the_part` | passes `[49.5 .. 48]`; the full 2 mm is removed |
    | `wall_arc_entry` (6 tests) | stock top 20 is Top Z; one level and one fan fewer; the second level now plunges onto Z 15; each entry's last rapid ends at Z 20 or above (new assert) |
12. Round 10, the floor review of head `68b5315c8`. Rest Adaptive
    (`crates/motion/src/machining2d/adaptive/rest/mod.rs`, from `main`)
    called `cut_levels`. In a rest operation the stock that the earlier
    operations left stands over the rest Top Z. A plain `cut_levels`
    dropped the level at Top Z, and the band Z-2..Z-4 of the
    `rest_adaptive` fixtures had no cut and no refusal.
    - Chosen: `cut_levels_under(top_z, material_top.or(Some(INFINITY)),
      ...)`. With no material top stated, the rest ladder keeps the plane
      at Top Z (the pre-#2004 ladder). A material top that is stated and
      is at or under Top Z still drops it, as for the other callers.
    - Dropped: to read the top from `RestStock`. The rest stock holds
      floors (proved depths), not a top Z, so it cannot state one. The
      safe default "material over Top Z" can only add a level that cuts
      air; it never leaves material.
    - Pinned by `adaptive::rest::tests::the_first_rest_level_at_top_z_is_cut_without_a_stated_material_top`
      and the 4 `rest_adaptive` tests of the review.
    - The merge of `main` regenerated `examples/first-router-part.camproj`
      with `WRITE_GOLDEN=1` (main moved the stamp salt to
      `camcraft-op-stamp-v3`); the stamps were not merged by hand.
13. Round 10, the tests that `main` added and that keep the old air
    level at Top Z. Each one is class (a): the stock top is Top Z.
    - `rest_pocket::the_rest_pass_cuts_only_the_corners_posts_and_matches_its_golden`:
      stock top 20 is Top Z, stepdown 2. `golden/pocket_rest_corners.cl`
      is regenerated (126 to 85 moves): the Z 20 level is gone, the
      levels Z 18 and Z 16 are the same paths.
    - `first_4axis_part` (3 tests): the stock box touches the bar, so
      the stock top is the skin Z0.375 and Top Z. The pass at the skin
      is gone. `examples/first-4axis-part.nc` and `.camproj` are
      regenerated with `WRITE_GOLDEN=1`. The test now asserts one plunge
      and one pass at the flat (Z0.36). The guide page
      `site/content/guide/getting-started/first-4axis-part.md` quotes
      the new program and says why no pass runs at the skin.

**Why:** a level at the material top removes nothing and doubles the cycle
time of a one-level cut (the maintainer's report on v0.21.0). The fix at the
shared law reaches each caller at once.

**Dropped:**

- A per-operation "skip the first level" flag: more surface, and the same
  answer everywhere.
- To drop the inner vault ring's opening plane too (the issue's first
  reading of `cut_levels(prev, ...)`): it makes the cut deeper, not shorter.
- Finishing 3D rings (`finishing3d/ladder.rs`): these are surface rings
  whose `top_z` is the tip Z of the highest ring, not a material top. Not
  changed.

**Cost to change later:** one function (`cut_levels`) and the turning face
rung count. Posted G-code of each depth-stepping operation changes (one level
fewer) - this is under the safety floor.

## Merge with #1559 (PR #2129): two fields on `Approach`

Taken unattended by the orchestrating session for the absent maintainer,
at the merge-time refresh of PR #2013 (2026-11-16).

**Chosen:** two separate fields on `camcraft_motion::approach::Approach`.

- `material_top` keeps the #1559 meaning and readers exactly as `main`
  has them: the carried stock over one region, set only where the plain
  entry dives into it, read by `entry_top()`, `carries_stock()` and
  `top_stock::carried_note`.
- This PR's field is renamed `stock_top`: the top of the setup's stock in
  the generator's frame, set for every operation in `generate.rs`, read
  only by the level law `cut_levels_under` (and its callers in the
  ladders, `TraceParams::stock_top`, `TrochoidParams::stock_top`,
  `area.rs`). The entry never reads it.
- `contour_bore.rs::bore_entry`: main's `p.entry_top()` on the first
  engagement, and this PR's `fresh` arm (an unopened column starts over
  the top of the cut). The `fresh` arm also reads `p.entry_top()`: where
  #1559 carries stock, the unopened column has that stock over it, so the
  descent starts over it. With no carried stock, `entry_top()` is the top
  of the cut, so this PR's path is unchanged.
- `generate.rs`: this PR's `lk.approach.stock_top = ...` line and main's
  `carried_top!` / `carry_need!` macros, both kept.

**Why:** one field with the two meanings sets a carried entry on every
operation. #2129 avoids that on purpose (a finish that enters through a
cleared bore keeps its path byte for byte).

**Dropped:**

- One field with the #1559 meaning, read also by the level law: the law
  then sees the stock top only on carried operations. Where stock stands
  over Top Z and no carry is set (a slot, a trace, a tilted frame), the
  law reads no stock over Top Z and drops the level at Top Z. That level
  cuts stock, so the change loses a cut, not only an air level.
- One field with this PR's meaning: every operation gets the carried
  entry.

## Review of head `eacc74c78`: Face and the vault rim ring read the stock top

Taken unattended by the issue worker for the absent maintainer, at the
fix of the safety-tier review of head `eacc74c78` (2026-11-17).

**Chosen:** Face and the vault rim ring follow item 3 (stock over Top Z
keeps the level at Top Z). This replaces the sentence of item 3 that
Face and the vault keep `cut_levels` (stated at head `eacc74c78`).

- `FaceParams` gets the runtime field `stock_top: Option<f64>`. The app
  sets it from `lk.approach.stock_top` (`generate.rs`). The staged Face
  takes its levels from `cut_levels_under(top_z, stock_top, cut_z,
  stepdown)`. A stated value that is not finite is refused in `validate`:
  a NaN must not read as "no stock over" and drop the level at Top Z.
  The field is never saved in the project file. Each test literal sets
  `None` (the #2004 law).
- Vault (`circular.rs`): the rim ring takes `cut_levels_under` with
  `p.links.approach.stock_top`. When stock stands over Top Z,
  `under_top` is not applied, so no plane at Top Z is dropped. That is
  the vault ladder of `main`. Without stock over Top Z, items 4 and 4b
  do not change.
- The test `one_descent::the_bore_contour_under_carried_stock_never_rapids_into_it`
  pins the `fresh` arm of `bore_entry` under carried stock (the
  reviewer's open edge). `main` fails its case `lead none Ramp helix
  transition` with `RapidInStock`; this branch passes it. The cases
  `lead linear Ramp planar/retract` give `RapidInStock` on `main` and on
  this branch alike. They do not pass through the `fresh` arm. The test
  skips them by name and #2301 tracks them.

**Why:** with stock 2 mm over Top Z and a stepdown of 3 mm, the Face of
the review cut `[15, 12]` from a stock top 20: a first bite of 5 mm. A
first bite deeper than one stepdown is not permitted.

**Dropped:**

- A refusal or a warning when stock stands over Top Z: the old ladder
  cuts that stock safely, so a refusal would stop a legal program.
- A second `face` entry point with the stock top as a parameter: the
  field follows the pattern of `TraceParams` and `TrochoidParams`.

## #2004 - Review of head `7cb6472cb`: stock caps in the cutting frame

**Chosen:** generation and area planning share `stock_top_frame.rs`'s
stock-cap calculation. For an indexed operation it transforms every corner
of the setup-stock and material bounds into the cutting frame and takes
the highest local Z. Upright operations retain their existing reading.
B90 and translated oblique Face pins cover generation; area-plane and
material-bound pins cover the shared calculation.

The two opening-rapid tests from `main` now expect the first cut at Bottom
Z -1 for their Top Z 0, stepdown 1 fixture. Their opening rapid, plunge,
feed and chained-path assertions remain in place.

**Why:** subtracting `origin.z` alone misses the stock cap after rotation.
The review's B90 Face saw 20 instead of 40 and silently took a 5 mm first
bite with a 3 mm stepdown. Generation and area planning must use the same
local cap so they retain the same cutting planes.

**Dropped:** retaining the setup-frame Z shift, or calculating a separate
cap for each caller; the former misses indexed stock, and the latter can
let generation and area planning disagree.

**Cost to change later:** update the shared cap function and its indexed
pins together. A changed cap can change posted depth levels and remains
under the safety floor.
