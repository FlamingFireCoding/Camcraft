//! #3034: the recorded QA part `q02-3d-mold.step` (a mould base with an
//! ellipsoid boss whose B-spline face ends in a pole) imports as a closed
//! solid and slices closed at the reported height. Before the fix the
//! import refused it: "face Some(10) stands 5.089e-1 mm off its surface".
//!
//! Fixture: `testdata/models/q02_3d_mold.step`, the recorded QA file of
//! issue #3034, byte for byte (347101 bytes). Our own part, made in
//! SolidCraft for QA; no outside material.
//!
//! Gate map (M4.4): covers app::geometry::import::step,
//! app::geometry::import (parse_model), camcraft_geometry::geometry::slice.

use std::path::Path;

use camcraft_app::geometry::import::{self, Format};
use camcraft_app::geometry::{Point3, slice};

/// SolidCraft's volume of the part, mm^3 (from the QA run of #3034).
const SOLIDCRAFT_VOLUME: f64 = 97165.69;

#[test]
fn the_recorded_q02_mould_imports_closed_and_slices_at_the_reported_height() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/models/q02_3d_mold.step");
    let bytes = std::fs::read(path).expect("fixture");
    let mesh = import::parse_model(&bytes, Format::Step)
        .unwrap_or_else(|e| panic!("the q02 mould must import: {e}"));
    // Signed volume (divergence theorem): closed and wound outward.
    let o = Point3::new(0.0, 0.0, 0.0);
    let volume: f64 = mesh.facets.iter().map(|t| (t.a - o).dot((t.b - o).cross(t.c - o)) / 6.0).sum();
    assert!((volume - SOLIDCRAFT_VOLUME).abs() < 10.0, "volume {volume} vs SolidCraft {SOLIDCRAFT_VOLUME}");
    // z = 19.6875 is the height the issue reports; the others cross the
    // boss flank and its cap. Areas measured on the fixed tree.
    for (z, want) in [(18.5, 1468.32), (19.6875, 1382.27), (20.0, 1371.70), (25.0, 932.50), (29.0, 225.77)] {
        let region = slice::slice(&mesh.facets, z).unwrap_or_else(|e| panic!("z = {z}: {e}"));
        let area = region.area();
        assert!((area - want).abs() < 0.5, "z = {z}: section {area} vs {want}");
    }
}
