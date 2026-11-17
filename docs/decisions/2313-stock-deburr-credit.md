## #2313 - A stock-source Deburr breaks the stock's edges, not a part's

**Decided.** A Deburr whose source is the setup's stock (the stock itself,
or picks off it) now obeys the same rule as a stock-source 2D Chamfer
(#2308):

- It is a stock-kept kind (`keep_law::is_stock_kept_kind`). At generation
  the solid law (#1631) keeps its cut out of every part of the setup, and
  the post's live re-judge (`kept_rejudge`) names a part added after
  generation.
- The program sim gives its bevel no edge-break credit (#303) over a part's
  columns (`stock_sim::breaks::breaks_stock_edge`).

One exception: a stock that IS a part. This is a "from model" stock whose
source is a judged part (not a fixture, not raw-stock-only) and whose facets
equal that part's facets in place, vertex for vertex within 1e-6
(`stock_sim::breaks::stock_part_of`). There the edges are the part's own
edges, as on a part source. The op keeps the #303 credit, and that one part
is spared from the solid law. All other parts of the setup stay kept.

**Why.** Before this fix a stock-source Deburr kept the credit over every
part's columns and had no part law. A cone that cut 1 mm into a second part
posted. Deburr on the stock of a from-model setup is a common legal edge
break, so a blanket refusal would refuse a legal program. The facet
equality test is the strict reading: when it fails, the op is judged as a
stock-edge break.

**Dropped.**

- Refuse every stock-source Deburr. This refuses the legal from-model edge
  break.
- Exempt a stock by its bounding box ("looks like the part"). A box that
  only matches the part's extents does not have the part's edges, and a
  wrong match forgives a gouge.
- Exempt a from-model stock by its source name only. A from-model stock
  that is moved off its source does not break the part's edges.

**Stricter than before (not covered by an existing test).**

- A stock-source Deburr on a box stock, or on a from-model stock moved off
  its source, now takes the solid law. If its cone reaches a part, it is a
  gouge. The workaround is to deburr with the part as source.

**The refusal (review fix).** A stock-source Deburr whose solid-law check
hits a part gets its own finding, `Warning::StockDeburrGouge`: "move N cuts
past the finish surface (stock-source deburr breaks into a part; use the
part as source)". The stock-source Chamfer keeps its #1637 finding. When
the sim's gouge verdict blocks the post, the post line also names the op
and the part through the live judge: "S1 breaks into part 'other' (...)".
A part added after generation keeps the #2308 text ("S1 now cuts part
'other' ... regenerate it"), now with the Deburr way out.

**Not covered.**

- On a turned stock that the sim cannot judge, only the live re-judge acts
  for a part added after generation. A convicted op there is refused by its
  own cached finding, which names the op and the way out, not the part.
- When the stock IS the part, the sim credits the bevel over the columns of
  all parts, not only the source part. The solid law (generation) and the
  live re-judge (post) catch a cut into a second part; the sim is not a
  second net for that part.
- The 1e-6 vertex test reads the facet order of the stock copy. A future
  re-mesh of the copy would remove the exemption (stricter, so safe).

**Cost to change.** Small: the exemption is one function
(`stock_part_of`), read by `breaks_stock_edge` and `keep_law::parts_in_stock`
/ `kept_models`.
