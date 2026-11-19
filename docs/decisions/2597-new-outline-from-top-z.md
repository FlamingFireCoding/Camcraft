## #2597 - Whole-model 2D Contour: a new, larger outline starts at Top Z

**Decided.** In a whole-model 2D Contour, a ring of a deeper band whose tool
sweep reaches outside every sweep cut so far gets every flat level above its
first level. So it is cut from Top Z down, one stepdown at a time, and not
first at the level where its outline appeared. The check is in
`crates/motion/src/machining2d/contour_whole_reach.rs` (`reach_up`). A ring
whose sweep stays inside the cut sweeps (an outline that does not grow) is
unchanged. If such a ring cannot be proven clear of the part at the added
levels (a slice error, or material over it), the operation is refused by
name, with no toolpath, so nothing posts.

**Why.** The contour has no stock outline. The material outside the part can
stand up to Top Z. A first pass at the new outline's level buried the tool by
the full height above it (20 mm on the #2597 part with stepdown 5). The
user's stepdown is the axial limit of the operation, so this is not a new
choice and not a new parameter: each pass is now at most one stepdown under
the pass before it on the same ring. The radial load of the added passes is
not more than that of the first band's passes: each pass engages at most the
tool width, as any contour pass does.

**Dropped.**
- Refuse every growing outline. Safe, but it refuses common parts (a boss
  on a plate) that can be cut correctly.
- A new operation parameter ("start new outlines at Top Z" on/off). The
  other value would post the buried pass again, which the Prime Directive
  forbids. So there is no choice to give.
- Read the stock outline to start each ring where the stock really starts.
  The contour generator gets no stock region today; that is a larger change
  (app and motion), and the Top Z rule is safe without it. It costs air
  passes where the stock is narrow. A follow-up can add the stock region.

**Cost of the rule.** A wall that grows a little at each level (a chamfer)
re-walks the new ring from Top Z at each band: more passes, mostly in air.
A strip narrower than 0.05 mm is taken as offset noise and does not count as
growth. A level at Top Z itself is never added: it removes no material, and
its lead-in crossed the lead-out of the ring above it on the chamfered island
of `contour_whole_step` case 1 (a `Crossing` warning blocked the post).
