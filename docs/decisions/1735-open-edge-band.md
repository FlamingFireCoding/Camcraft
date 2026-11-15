## #1735 - Two open edges are the walls of a band that the stock outline closes

**Decided:** A pocket-family operation (2D Offset, 2D Pocket, 2D Adaptive) whose selection holds two open edges of a part cuts the band between them. The definition is exact:

- Picked open runs that share an end are welded into one wall. Exactly two walls, A and B, must remain. Each wall is level, and both walls are at one height (tolerance 0.001 mm).
- Each wall is run on straight along the tangent at each of its ends.
- S is the stock outline in XY, grown by the extension `e` with sharp corners.
- `band(e) = (side of A that holds B) ∩ (side of B that holds A) ∩ S`.
- `band(0)` is the region of the selection. The pocket region law then removes the standing material of the part at every cut plane, as it does for a closed pick.
- `band(e)` is a lane. The area law clips its result to the lane after the extension step. Thus the extension moves the two open ends out of the stock along the walls. The extension does not widen the band, and the tool does not go round the end of a wall.

A selection that does not give one clear band is refused by name: no stock; a wall that is not level; walls at different heights; walls at more than 45 degrees to each other; walls that do not run alongside each other; walls that cross or touch when run on; walls that run together inside the stock; a stock outline that does not cover the space between the walls; a stock outline that closes the band in more than one piece; more than two walls; open edges mixed with closed boundaries; open edges of the stock; three edges at one point. One open edge keeps the old refusal, word for word. Tool containment On or Outside is refused for a band, because the walls are the limit of the tool.

**Why:** The maintainer reported it (issue 1735): two edges of a slot must give a path up to the stock edge, and "Extend past" must extend the reach out of the stock in the direction of the edges. An open slot or open pocket is bounded by its walls and by the stock in general 2.5D machining practice. The definition uses only half-plane splits and region intersections, so each result has one reading. No check was loosened: the region still loses the part's standing material, the operation still answers to the section gouge law and the solid law against the other parts of the setup, and a gouge still blocks the post. The test `a_band_pass_that_cuts_another_part_blocks_the_post` pins that.

**Considered and dropped:**

- Close the band with the shortest walk along the stock outline between the wall ends. Dropped: the walk has two directions at each end, and a wall that stops before the stock edge needs a guess for the gap.
- Run each wall out with Face's `extend_run_to_footprint` and union the two strips. Dropped: that law asks which side of one run holds the material; a band needs the side that holds the other wall.
- Cast rays along the wall normals to find the opposite wall. Dropped: it fails for walls of different lengths and for tessellated curved walls.
- Let the general extension step (`offset_sharp` of the area) do the "Extend past". Dropped: it grows the area sideways where no part material stands, so the tool would go round the end of a wall. The lane clip prevents that.
- A polygon made of the four wall ends. Dropped: it stops at the wall ends and does not reach the stock edge.
- A wedge between walls that meet inside the stock (one open end). Dropped for now: it is refused by name. It can be allowed later without a change to the two-open-end case.
- A separate module in `camcraft-geometry` for the pure law. Dropped: the law calls `sides_of_chain` (`app::operation::boundary_turn`) and the file reads the world (the stock outline and the picked runs).

**Limits of the decision:** The 45 degree limit is a choice, not a derived value. With extension 0 the tool centre stops one tool radius inside the stock edge (the pocket family keeps the tool inside its region); an extension of one radius or more clears the stock edge. A positive boundary offset cannot widen a band, because the lane clips it; a negative offset narrows the band. A blind slot picked as one U-shaped run (three edges) stays refused as one open run.

**Cost to change later:** Small. The law is one function (`open_band::band`) with unit tests, one hook in the selection walk and one clip in `boundary::resolve`. A change of the 45 degree limit or of a refusal is local to `open_band.rs`. A change of the default reach at the open ends (for example tool centre on the stock edge) would change the generated paths of band selections only; no golden holds one.
