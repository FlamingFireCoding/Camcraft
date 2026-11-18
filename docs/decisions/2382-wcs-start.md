## #2382 - Every operation starts from the setup's WCS, not the world origin

**Decided.** The generators keep their seed at the frame's zero on the
clearance plane. After generation, and before the yaw exit, the fixture
routing and the verify laws, `generate.rs` calls
`Toolpath::seat_approach`. When the first move is a rapid that leaves that
seed, its start moves to the setup's WCS origin (X/Y) on the same plane.
A 3+2 path reads the origin through `TiltedFrame::to_local`. The origin
comes from `posting::wcs_context` (the `setup::wcs_entity` transform), the
same value the post subtracts.

**Why.** The opening rapid started at (0, 0, clearance) of the document.
That is the world origin. The viewport drew it, the simulation used it to
link the operations, and the CL `START` line and the post's `StartX/Y/Z`
read it. The posted move blocks were already measured from G54, because a
post writes the end of each move only. One rewrite at the single point
where every milling kind passes is smaller and safer than a new parameter
in about 35 generators. The rapid's end does not change, so no posted
block moves.

**Dropped.**
- A WCS seed parameter in every generator: a large diff in the motion
  crate for the same result, and each new generator could forget it.
- A change of the zero-length seed rapid that the approach law (#690)
  states when the cut starts at the seed: it has no extent, so moving its
  start would invent a rapid that the generator did not plan.
- A change of hole, plasma, sinker, lathe, wire and rotary paths: they
  seed at their own first point already, and `seat_approach` leaves a
  path alone when its first move does not leave the frame's zero.

**Cost to change later.** Low. A generator that seeds at a WCS point by
itself makes the rewrite a no-op for that kind.
