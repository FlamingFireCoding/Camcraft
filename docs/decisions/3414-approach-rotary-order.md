## #3414 - The approach rotates before it descends

**What was decided.** The first five-axis approach splits into two moves:
a zero-travel Rapid whose axis is the only change (the rotary index -
the machine swings at a standstill, tip fixed, on the hop plane or at
the tip it already holds), then the Plunge/cut descend under the pose
already stated. Applied in `curve.rs` (every chain's approach) and in
`stamp.rs` (every posed kind, at every rapid-to-cut re-entry where the
pose changes). No new `MoveKind`.

**Why.** A pose change riding the move that reaches first contact
turns the rotaries while the tool descends - the ball sweeps through
unknown air and through the rim it approaches.

**What was considered and dropped.**
- A short travel rapid along the hop plane under the new pose: it
  drags the tip sideways while the head turns, and it blurs the law
  that a travel rapid inherits its pose.
- A new `MoveKind::Rotate`: the post already solves a zero-travel
  segment as a pure rotary reorientation (`g93.rs`'s `ds_tip == 0`
  path), and the builders already keep a zero-travel move whose pose
  changed - no new verb is needed.

**Cost to change later.** The law tests (`rapids_hold_the_pose_they_inherit`,
`rapid_pose_contract`, the air-link ratchet, `balls_are_the_plan`) read
the swing-free stream through `no_swings`/`sans_swings` helpers. If a
real rotate verb ever lands, those helpers collapse into one place.
