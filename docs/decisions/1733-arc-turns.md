## #1733 - Arc turns at corners with a radius on 2D Pocket, 2D Adaptive and 2D Offset

**What was decided.**
- The existing "arc turns at corners" option (`blend_turns`, M11.1) gets a
  radius, `blend_radius`. No radius means the tool radius, which is what
  every earlier file blended at. So an old file and a new default
  operation keep their output byte for byte.
- The radius is bounded: at least 0.01 mm (a smaller arc is not a move a
  control can cut) and at most the tool radius. A larger arc would leave
  more corner rest than the sharp pass does. The upper bound is checked in
  the generator, which knows the tool; the lower bound is checked at the
  command door (`named_refusal`), even when the option is off, so a stored
  nonsense radius cannot come back at a later turn-on.
- 2D Adaptive and 2D Offset get the option and the radius. 2D Adaptive's
  Trochoidal method refuses the option by name: its loops are arcs
  already, and the option is never silently dropped.
- The stamp digest takes the two new values only when they are set, so
  the digests of existing operations do not change (no false Stale).
- The project file writes `blend_radius` only when set, with no version
  step. An old reader ignores the key.
- The MCP `edit_operation` door accepted `blend_turns: true` but refused
  `blend_turns: false` as an unknown field, because the flag is written
  only when ON. A `false` leaf is now kept when the same key set to
  `true` survives the typed round-trip. A misspelled key is still refused.
- The UI seeds 0.5 mm when the user first gives a radius.

**Why.** The maintainer asked for arc turns at corners as an option on
pocket-type operations, with the default output unchanged. The corner rest
of a smaller arc is stated, as before, by the fillet tally.

**Considered and dropped.**
- A radius larger than the tool radius with a rest pass: that is a new
  corner-rest strategy, not this option.
- Slot: open PR #1745 changes Slot; it is left to a follow-up so the two
  PRs do not conflict. Face and 2D Contour also go to the follow-up.
- A pinning test of 2D Adaptive on the large island fixture: one debug run
  takes about 15 s, so its test uses a small rectangle.

**Cost to change later.** The radius is a plain optional field; a wider
bound or more kinds add arms to the same door and generator call.
