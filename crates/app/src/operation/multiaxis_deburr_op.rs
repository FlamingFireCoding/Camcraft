//! The Multi-Axis Deburr operation's seed, tool doors and kernel
//! numbers (#1894): the 3D Deburr's fields read onto
//! [`crate::multiaxis::AxisDeburrParams`]. Written from our own design
//! notes and machine-tool handbook chamfer geometry.

use crate::finishing3d::DeburrParams;
use crate::machining2d::GenError;
use crate::multiaxis::{AxisDeburrParams, DeburrAxisMode, Shank};

use super::kinds::Carried;
use super::types::OpParams;

/// A fresh Multi-Axis Deburr: the Deburr seed's numbers, `Axis5`, no
/// lean, a 30-degree tilt cap and the holder kept off the part.
pub fn seed_multiaxis_deburr(c: &Carried) -> OpParams {
    OpParams::MultiAxisDeburr {
        width: 0.5,
        clearance_z: c.clearance,
        top_z: c.top,
        bottom_z: c.bottom,
        feed: c.feed,
        plunge_feed: c.plunge,
        exclude_edges: Vec::new(),
        min_edge_angle_deg: 40.0,
        exclude_holes: false,
        hole_min_diameter: 0.0,
        hole_max_diameter: 10.0,
        unreachable: crate::finishing3d::UnreachableEdges::SkipWithNote,
        sizing: crate::finishing3d::DeburrBreak::default(),
        axis_mode: DeburrAxisMode::Axis5,
        lead_deg: 0.0,
        lag_deg: 0.0,
        max_tilt_deg: 30.0,
        shank_avoid: true,
    }
}

/// The chamfer mill a tilted ride needs, refused by name otherwise.
/// Returns the cone's usable diameter: the cone's top, or where the
/// flute ends on the cone when the flute is shorter than the cone.
pub fn cone_diameter(tool: &crate::tools::Tool, tool_name: &str) -> Result<f64, GenError> {
    if tool.kind != crate::tools::ToolKind::Taper {
        return Err(GenError::Invalid(format!(
            "Multi-Axis Deburr lays a cone flank on the break - it needs a chamfer mill, '{}' is a {}",
            tool_name, tool.kind
        )));
    }
    if !(tool.tip_angle.is_finite() && tool.tip_angle > 0.0 && tool.tip_angle < 180.0) {
        return Err(GenError::Invalid(format!(
            "Multi-Axis Deburr needs a cone tool: '{}' has an included angle of {}, which rules no chamfer",
            tool_name, tool.tip_angle
        )));
    }
    let half = 0.5 * tool.tip_angle.to_radians();
    let d = tool.diameter.min(2.0 * tool.flute_len * half.tan());
    if !(d.is_finite() && d > 0.0) {
        return Err(GenError::Invalid(format!(
            "Multi-Axis Deburr: '{}' has no cone flank (diameter {} mm, flute {} mm)",
            tool_name, tool.diameter, tool.flute_len
        )));
    }
    Ok(d)
}

/// The kernel's numbers from the operation's fields, the resolved
/// window and the tool. `None` when `params` is another kind.
pub fn kernel_params(
    params: &OpParams,
    window: (f64, f64, f64),
    tip_angle_deg: f64,
    cone_d: f64,
    shank: Shank,
) -> Option<AxisDeburrParams> {
    let OpParams::MultiAxisDeburr {
        width,
        feed,
        plunge_feed,
        min_edge_angle_deg,
        exclude_holes,
        hole_min_diameter,
        hole_max_diameter,
        unreachable,
        sizing,
        axis_mode,
        lead_deg,
        lag_deg,
        max_tilt_deg,
        shank_avoid,
        ..
    } = params
    else {
        return None;
    };
    let (clearance_z, top_z, bottom_z) = window;
    Some(AxisDeburrParams {
        base: DeburrParams {
            width: *width,
            tip_angle_deg,
            clearance_z,
            top_z,
            bottom_z,
            feed: *feed,
            plunge_feed: *plunge_feed,
            min_edge_angle_deg: *min_edge_angle_deg,
            exclude_holes: *exclude_holes,
            hole_min_diameter: *hole_min_diameter,
            hole_max_diameter: *hole_max_diameter,
            unreachable: *unreachable,
            sizing: *sizing,
        },
        mode: *axis_mode,
        lead_deg: *lead_deg,
        lag_deg: *lag_deg,
        max_tilt_deg: *max_tilt_deg,
        tool_diameter: cone_d,
        shank: shank_avoid.then_some(shank),
    })
}
