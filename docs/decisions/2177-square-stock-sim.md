## #2177 - A stock in a vise is simulated: no locate noise, quarter turns modelled

**Decided.**
- `MoveModel` (and so `LocatePart`) keeps the stock's own rotation when the
  move does not turn the part (`turns()` false, the same 1e-12 tolerance the
  square refit already uses). Before, the rigid follow wrote
  `next * prev^-1 * before`: float noise of the locate solve (about 5e-14)
  became a stored stock rotation, and the sim refused the stock.
- One law, `stock_follow::sim_frame`, gives the stock as the sim models it.
  `stock_sim::stock_grid`, `stock_sim_held::first_body` / `first_field` and
  `stock_view::stock_facets` all read it. An unturned stock is unchanged.
  A box turned by a quarter-turn multiple (within 1e-9) is its unturned twin
  with the sides swapped. An upright cylinder (axis up or down) is an
  unturned one. A from-model stock is read from its placed mesh at any turn.
  An oblique box and a laid cylinder are refused by name, in plain words.
- The UI says a simulation refusal once (`State::say_once`) and in plain
  words: "simulation shows the uncut stock: <reason>". The text "the field
  view" and "the sheet stands" is gone from the status line.

**Why.** The maintainer saw no simulated stock and the refusal twice for a
block in a vise. The cause was the noise, not a real turn. A quarter-turned
box is the same solid as its twin, so the sim can judge it exactly. A
tolerance in the refusal alone (without the source fix) would leave noise in
saved projects and in the stock that a re-fit keeps.

**Dropped.**
- Only a tolerance in the two refusals: it hides the noise but does not
  model a true quarter turn of a from-model or box stock.
- An oriented lattice for oblique stock: new geometry, effort:max, not
  needed for the reported scene.

**Effect on existing projects.** A project whose stock carries noise or a
quarter turn was "not judged" by the stock sim. It is now judged: a gouge
that the sim finds can now block the post. No posted G-code changes.

**Cost to change.** `sim_frame` is one function; a future oriented lattice
replaces its refusal arm.
