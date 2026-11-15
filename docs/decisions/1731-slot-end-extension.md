## #1731 - Slot end extensions

**Decided.** The Slot operation has two new parameters, `extend_start`
(past A) and `extend_end` (past B), in mm, zero by default. The pass runs
between `A - extend_start * u` and `B + extend_end * u`, where `u` is the
unit direction from A to B. The values must be finite and not negative;
the command refuses other values at creation. The project file writes a
key only when its value is not zero, and the parameter digest appends the
pair only when one is not zero. Thus a slot with no extension keeps its
file text, its stamp and its output.

**The gouge law judges the extended pass.** The Slot's machine area (the
region that the verifier reads) is the stadium of the extended ends: the
channel that the user asks for. The part law is not changed. On a part
source, the part's solid section judges the path; on a stock source, the
setup's parts judge it (#1607). An extension that enters a part that the
job keeps is a blocking Gouge, and the post refuses the program.

**The pick room judges the authored slot.** On a Selection source, the
#1504 room law (`SlotBound::judge`) reads the authored A, B and width, not
the extended ends. An extension goes past the picks because the user
asks for it, and the part law then judges where it goes.

**Considered and dropped.**
- One `extend` value for both ends: the maintainer asked for an
  extension "in the direction of its length", and a slot that runs out
  through one edge only needs a different value at each end.
- Keep the region as the authored stadium: the verifier would then
  convict every extended move, also in air, and the parameter could not
  be used.
- Move the extension into `SlotParams` in motion: the generator is a
  pure function of two endpoints; the app moves the endpoints with
  `machining2d::slot_extended_ends`, and the generator does not change.
- Extend the Trochoidal slot too: not asked for; it can take the same
  helper later.

**Cost to change later.** Small: two additive fields with serde defaults.
