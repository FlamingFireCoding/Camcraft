//! #1749: the footprint a whole-model 3D Pocket (and its stamped twin,
//! Multi-Axis Rough) clears - the silhouette grown past its open edges
//! (`machining3d::open_sides`), the setup's other parts kept out.
//!
//! Provenance: the "tool centre on boundary" containment public CAM
//! documentation describes; our own design, no vendor material consulted.

use super::*;

/// `region` as the ladder reads it: grown past the silhouette when it is
/// the model's own source-built silhouette (`source_built`, an untilted
/// whole-model source); otherwise the picked or controlled boundary,
/// exactly as given. `neighbours` are the setup's other parts
/// (`keep_law::neighbour_parts`): the growth never reaches into them.
pub(super) fn open_footprint(
    region: Region,
    source_built: bool,
    tilted: bool,
    src: &SrcRef,
    neighbours: &[Triangle],
    r: f64,
) -> Result<Region, GenError> {
    if !source_built || tilted || !matches!(src, SrcRef::Entity(_)) {
        return Ok(region);
    }
    let keep_out = if neighbours.is_empty() {
        Region::empty()
    } else {
        slice::project(neighbours, FOOTPRINT_SAMPLES).map_err(from_slice)?
    };
    machining3d::open_sides(&region, r, &keep_out)
}
