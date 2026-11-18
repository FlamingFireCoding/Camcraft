# 2019 - lathe world gizmo

Issue #2019, stacked on #2023 (the lathe view frame).

## Decision: mill-turn machines

- **Chosen:** a mill-turn machine keeps the milling frame (Z up), as #2023
  does. Its gizmo shows X, Y and Z. It hides Y only when its machine file
  states axes and none is a linear Y. An unstated axis list hides nothing.
- **Why:** the issue recommends it. A mill-turn mills with live tools, so
  the milling view is the one that reads the part. The issue asks Y to show
  when the machine has a Y axis.
- **Dropped:** a lathe frame for mill-turn (it would turn milling views on
  their side); a user switch for the frame (more UI, no request).

## Decision: lathe gizmo

- A lathe shows Z and X. It shows Y only when its file states a linear Y.
- The gizmo reuses the camera frame of #2023 (`ViewFrame`). The orientation
  has one derivation. This change only filters which axes are drawn
  (`view_frame::axes_shown`).
- View cube face names are not changed: the cube names world directions
  and #2023 keeps them true in the lathe frame.
- Display only: the WCS, the toolpaths and the posted program do not read
  this.
