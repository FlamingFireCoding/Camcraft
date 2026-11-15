## #1993 - The lathe envelope lies along the spindle axis: a lathe view frame for the camera

**What was decided.** The cause is in the camera, not in the envelope.
`crates/post/src/machine/envelope.rs` already gives a lathe a cylinder
about world Z (the Z travel sets its ends, the largest X limit its
radius), and the turning paths use world Z as the spindle axis too. But
`Camera::basis_right` in `crates/ui/src/viewport.rs` always crossed the
view with world +Z, so world Z could never lie across the screen: every
cylinder about Z stood vertical.

The fix gives the camera an orbit frame, `crate::view_frame::ViewFrame`
(`Mill` = identity, `Lathe`). Yaw and pitch are read in the frame; the
eye and the right/up/forward basis map back to the world through one
proper rotation (local x = world -Z, local y = world +Y, local z = world
+X). `mvp`, `ray_through`, `dolly_cursor`, `pan`, the traced-stock eye and
the view cube all read that basis, so draw and pick stay on one matrix.
`Camera::toward` converts the cube's world direction into the frame.
`pan` now uses the Z component of the right vector (it was always zero
before).

The frame comes from the machine data: `view_frame::sync` runs at the
top of the viewport panel each frame and reads
`camcraft_app::machine::kind_of` (the active setup's machine). A change
of frame (a lathe mounted, or a mill again) takes the frame's home view.
Fit and "nothing to frame" keep the current frame.

The screen rule for a lathe: the right-handed program frame of ISO 841
stays (the world, WCS, toolpaths and posted G-code do not change). The
home view puts world +Z (the spindle axis, away from the chuck) to the
right and world +X (the cross-slide, radial) up, slightly raised toward
+X; world +Y then points at the viewer. No hand is hard-coded per
machine: the frame is a rotation of the program frame, so a machine
file with an inverted axis changes only its posted words, as before.

**Why.** Rotating the envelope alone would separate it from the stock
and the turning paths, which also use world Z. A camera frame turns
all of them together, and it is the base that #2019 (the gizmo) needs.

**Considered and dropped.**
- Draw the cylinder about world X: wrong, it would disagree with the
  turning paths and the travel check (`crates/app/src/machine/lathe_stroke.rs`).
- A general free up vector (quaternion camera): larger change; every
  existing yaw/pitch law (view cube roll snap #622, Iso button) would
  need a new form. The frame keeps them unchanged in local terms.
- Mill-turn in the lathe frame: its milling operations use the milling
  convention today. It stays `Mill` until #2019 decides it.

**Cost to change later.** `ViewFrame` is one enum with two rotations;
a new frame (mill-turn, a front-turret hand) is one more arm in
`to_world` / `to_local` / `home` and in `for_kind`.
