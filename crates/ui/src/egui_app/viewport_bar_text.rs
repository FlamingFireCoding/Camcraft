//! The viewport bar's plain-English labels and hover tooltips (#2914).
//!
//! Provenance: original text, labels and descriptions for CamCraft's
//! viewport bar controls; no vendor material.

use super::Nav;
use crate::state::{DisplayMode, DragMode, State};
use crate::viewport::ProjectionMode;

/// The plain-English state line of the pick-through button (#692).
#[must_use]
pub(crate) fn pick_through_label(on: bool) -> &'static str {
    if on { "Pick through: on" } else { "Pick through: off" }
}

/// The plain-English state line of the Rotate button (#2047).
#[must_use]
pub(crate) fn rotate_label(on: bool) -> &'static str {
    if on {
        "Rotate: drag a ring on the selected part"
    } else {
        "Rotate: off"
    }
}

/// The plain-English state line of the Move button (#1198 b).
#[must_use]
pub(crate) fn move_label(on: bool) -> &'static str {
    if on {
        "Move: drag a ball to move or a ring to rotate the selected part"
    } else {
        "Move: off"
    }
}

/// The status line the projection button flips the viewer between.
#[must_use]
pub(crate) fn projection_label(mode: ProjectionMode) -> &'static str {
    match mode {
        ProjectionMode::Perspective => "projection: perspective",
        ProjectionMode::Orthographic => "projection: orthographic",
    }
}

/// The status line a latched nav tool wears on the status bar.
#[must_use]
pub(crate) fn nav_tool_label(tool: DragMode) -> &'static str {
    match tool {
        DragMode::Orbit => "nav tool: orbit",
        DragMode::Pan => "nav tool: pan",
        DragMode::Zoom => "nav tool: zoom",
    }
}

/// The status line when a nav tool latches off.
pub(crate) const NAV_TOOL_CLEAR: &str = "nav tool: off";

/// The base plain-English label for each of the viewport controls.
#[must_use]
pub(crate) fn label(kind: Nav) -> &'static str {
    match kind {
        Nav::Frame => "Frame all",
        Nav::Iso => "Isometric view",
        Nav::Wire => "Display mode",
        Nav::Paths => "Show all toolpaths",
        Nav::Axes => "World axes",
        Nav::PickThrough => "Pick through",
        Nav::Move => "Move mode",
        Nav::Rotate => "Rotate mode",
        Nav::Projection => "Projection",
        Nav::Orbit => "Orbit tool",
        Nav::Pan => "Pan tool",
        Nav::Zoom => "Zoom tool",
    }
}

/// The descriptive hover tooltip for each of the viewport controls (#2914),
/// reflecting the current state of toggle controls.
#[must_use]
pub(crate) fn tooltip(kind: Nav, state: &State) -> String {
    match kind {
        Nav::Frame => "Frame all (F): fit all objects in view".to_owned(),
        Nav::Iso => "Isometric view: orient camera to isometric view".to_owned(),
        Nav::Wire => match state.display {
            DisplayMode::Wireframe => {
                "Display mode: wireframe (click for shaded with edges)".to_owned()
            }
            _ => {
                "Display mode: shaded with edges (click for wireframe)".to_owned()
            }
        },
        Nav::Paths => {
            if state.show_all_paths {
                "Show all toolpaths: on".to_owned()
            } else {
                "Show all toolpaths: off".to_owned()
            }
        }
        Nav::Axes => {
            if state.show_world_axes {
                "World axes: on".to_owned()
            } else {
                "World axes: off".to_owned()
            }
        }
        Nav::PickThrough => {
            if state.pick_through {
                "Pick through: on (picks reach far side)".to_owned()
            } else {
                "Pick through: off (picks stop at front face)".to_owned()
            }
        }
        Nav::Move => {
            if state.move_handles {
                "Move mode: on (drag handles to move part)".to_owned()
            } else {
                "Move mode: off (show move handles)".to_owned()
            }
        }
        Nav::Rotate => {
            if state.rotate_handles {
                "Rotate mode: on (drag rings to rotate part)".to_owned()
            } else {
                "Rotate mode: off (show rotation rings)".to_owned()
            }
        }
        Nav::Projection => match state.cam.projection {
            ProjectionMode::Perspective => {
                "Projection: perspective (P, click for orthographic)".to_owned()
            }
            ProjectionMode::Orthographic => {
                "Projection: orthographic (P, click for perspective)".to_owned()
            }
        },
        Nav::Orbit => {
            if state.nav_tool == Some(DragMode::Orbit) {
                "Orbit tool: active (drag to orbit)".to_owned()
            } else {
                "Orbit tool: click to latch orbit drag".to_owned()
            }
        }
        Nav::Pan => {
            if state.nav_tool == Some(DragMode::Pan) {
                "Pan tool: active (drag to pan)".to_owned()
            } else {
                "Pan tool: click to latch pan drag".to_owned()
            }
        }
        Nav::Zoom => {
            if state.nav_tool == Some(DragMode::Zoom) {
                "Zoom tool: active (drag to zoom)".to_owned()
            } else {
                "Zoom tool: click to latch zoom drag".to_owned()
            }
        }
    }
}
