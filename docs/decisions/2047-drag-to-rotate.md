## #2047 - Drag-to-rotate rings in the viewport bar (#1198 slice c)

**Decided.**
- The bottom bar gets a Rotate button next to Move. Move and Rotate are
  mutually exclusive: turning one on turns the other off and cancels any
  live drag of the other.
- The selected part shows three rings (one in the plane square to each
  shown axis) round its bounding-box centre, at a constant 60 px radius on
  screen. A press within 6 px of a ring grabs it; the nearest ring wins.
- The angle is where the cursor ray meets the ring plane, measured about
  the centre with `atan2`, unwrapped so a drag can pass 180 degrees. When
  the ring stands edge-on to the eye the press starts from the ring point
  under the cursor and a grazing ray holds the last angle.
- The release queues one UI verb `CommandKind::TurnModelBy { name, axis,
  angle_rad }`. The drain composes the turn about the centre with the
  shown pose, maps it back through the active setup's alignment, and runs
  the existing `MoveModel`. So a dragged turn and a typed turn are the
  same undo step, the same staleness and the same following stock.
- A turn below 1e-9 rad queues nothing (no rotation noise, #2177).
  Escape cancels with no step. A bad axis, a non-finite angle or a
  missing part is refused by name on the status line.

**Why.** Rings are the usual CAD convention for a turn about an axis and
do not collide with the Move mode's balls. A constant screen size keeps
the rings grabbable at every zoom. Reusing `MoveModel` keeps one undo law
and one staleness law (`a_moved_part_marks_generated_ops_stale_and_undo_clears_it`
already covers a pure rotation).

**Dropped.**
- A new app command (`TurnModel`): no new behaviour in the document, so no
  new MCP tool either. MCP `move_model` already sets the rotation.
- Angle snapping (15-degree steps with a modifier): not asked for; can be
  added in `turn_drag::motion` without a format change.
- Screen-space angle (atan2 of the cursor about the projected centre): it
  is wrong when the ring is seen at a slant.

**Cost to change.** Small. The law is in `crates/ui/src/turn_drag.rs`; the
paint in `crates/ui/src/egui_app/turn_handles.rs`.
