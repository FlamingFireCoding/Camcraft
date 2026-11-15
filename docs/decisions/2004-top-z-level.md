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
   Callers with no `links` (face, vault, trace, trochoidal slot) keep
   `cut_levels`: face and vault have their own law (items 4, 4b), and
   the trace and trochoidal slot are on the remaining list.
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
