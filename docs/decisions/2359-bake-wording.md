## #2359 - Describe Bake as table coordinates without controller rotation

**Decided.** Preserve Bake's existing posting behavior. Generated CL already
contains Setup/table coordinates. Bake posts those coordinates relative to
the work offset origin without applying the WCS yaw again or emitting
controller rotation. The operator must not enter a rotation on the control.
Correct the `PostOptions` and `WcsRotationOutput` documentation and the UI
tooltip to describe this behavior. Name the choice
`Bake into XY (table coordinates)` to make the posted frame visible.

**Why.** `post::wcs_rotate::apply` deliberately leaves Bake records in the
Setup/table frame. The existing
`bake_keeps_setup_table_coordinates_without_g68` test pins nonzero-yaw move,
arc and drilling coordinates. Rotating those coordinates again would move
the physical cut. Control mode instead transforms records into WCS
coordinates and emits the controller rotation that restores the table path.

**Dropped.** Changing Bake's algorithm to match the old wording would change
correct posted coordinates. Removing the mode or changing its stored key
would require unnecessary compatibility work. The `bake` and `control`
keys, their defaults and the generated programs remain unchanged.

**Cost to change later.** Wording changes are small. Changing either mode's
coordinate contract requires new posting and generation evidence for moves,
arcs and cycles, plus an explicit migration and operator guidance.
