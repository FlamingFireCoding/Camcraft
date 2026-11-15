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
3. Top Z below the stock top (material above Top Z): the meaning is
   unchanged. Material above Top Z is outside the operation, as it was
   before. The first level now cuts `stock top - Top Z + stepdown`; before
   it cut `stock top - Top Z` at Top Z itself. Neither ladder kept each
   level under one stepdown in that case. No new refusal is added here:
   the 2D generators do not read the stock, and the stock simulation and
   the verify still judge the posted program.
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
