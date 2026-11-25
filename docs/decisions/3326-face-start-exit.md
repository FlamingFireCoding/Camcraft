## #3326 - Face takes a start pick and an exit pick

**What was decided.**

1. **The capability.** Face answers `PickCapability::Raster`. It takes a start pick and an exit pick from the operation-level `picks` of PR #3871.
2. **The start pick.** The raster starts at the corner of the face region's bounding box that is nearest the pick. The edge row at that corner is cut first, away from the corner. The rows then step to the far edge. The pick sets the first row direction, so the Climb switch does not move a picked corner.
3. **The exit pick with a start pick.** The exit pick names the X side where the last row ends. The row count decides that side: an even count ends on the side of the start corner, an odd count ends across from it. A side that the count cannot reach is refused by name. The refusal names the row count and the start corner as the cause, and it marks the exit pick in the viewport.
4. **The exit pick alone.** The planner is free to choose the start corner, so it chooses the one that ends the last row at the corner nearest the exit pick. This exit is always reached.
5. **An exit beside the first row.** With a start pick, the last row is at the edge across from the start. An exit pick that is nearer the first row is met on its side only. The path says so in a note.
6. **A tie.** A pick at equal distance from two sides takes the low side.
7. **The rows never change.** The pick orders the rows of the facer's own band. The row positions, the overrun, the holes law, the "Do not touch" law, the level ladder and the air entry read the same rows. From a high Y corner the rows are the same band in the opposite order, not a mirrored band.
8. **The turned frame.** A Face laid along an open boundary works in a turned frame. The picks are stated in the part's frame and are turned into the laid frame before the corner is read.
9. **A tilted (3+2) Face refuses a pick by name**, as the tilted 2D Contour does. The pick is model X and Y, and the tilted plane has a different X and Y.
10. **The page.** The Face Geometry tab shows a Start point group and an Exit point group with typed X and Y, a switch and a Clear button. Apply commits the draft through `EditOperationParams`.
11. **Where the code is.** The planning rule is a new module, `motion::machining2d::face_raster`. `face_picked` is the new entry point. `face_over_kept` calls it with no pick.

**Why.**

- The issue states the start rule ("the corner of the face region nearest the pick; the row direction and the step direction follow from that corner") and the exit rule ("the last row ends at the side nearest the exit pick; if the row count and the start corner make that impossible, the exit is refused by name").
- The issue is silent about an exit pick without a start pick. The refusal in the issue names the start corner as a cause. Without a start pick the corner is the planner's choice, so the smallest honest rule is to choose the corner that reaches the exit.
- A plan with no pick is the automatic raster with the same numbers in the same order. Every shipped Face golden is byte-identical.
- The reversal plan of the arc turns is computed on the ascending rows and then put in cut order. The blend law reads positive strides, so its numbers do not change.
- The review of the Wire Contour exit (#3332) found that a stated pick must be visible on the page. The Face page follows that.

**Considered and dropped.**

- A new field in `FaceParams`: dropped. Each test that builds a `FaceParams` must change, and the picks are already one shared type.
- Mirror the region in Y to start at the high edge: dropped. The band snaps its last row to the high edge, so a mirrored band cuts different rows. The holes, the keep-out closure and the arc senses must be mirrored too.
- With an exit pick alone, keep the automatic start corner and refuse half of the exits: dropped. The planner can reach each exit, so a refusal has no cause that the user stated.
- Judge the exit pick by its nearest corner when a start pick is stated: dropped. The issue says "side". The last row is always at the edge across from the start, so a corner rule refuses each exit beside the first row.
- A viewport Pick button on the Face page: not in this change. The viewport pick of `start_exit_pick` writes to the 2D Contour only, and PR #3978 changes that file. The typed rows do not depend on it.
- A pick on a tilted Face, projected into the tilted plane: not in this change. It is refused by name.

**Cost to change later.**

Low. The rule is one function, `face_raster::plan`, with its own tests. A project file stores the picks only, so a different rule changes the path and not the file format.
