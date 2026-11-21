//! Adaptive trochoid shape checks from the project tests for issues #1509 and #2990.
//! Gate map: covers camcraft_motion::machining2d (adaptive_troch, adaptive_troch_lanes, adaptive_troch_clean), camcraft_motion::quality.

#[path = "common/adaptive_troch.rs"]
mod fixtures;

use fixtures::*;
use camcraft_motion::machining2d::PocketParams;

#[test]
fn wide_rectangular_pocket_is_cleared_by_parallel_lanes() {
    let started = std::time::Instant::now();
    let p = PocketParams { bottom_z: -3.0, ..params() };
    let r = rect(0.0, 0.0, 30.0, 24.0);
    check_cleared("wide 30x24 D6", &r, &p);
    // A D10 tool needs a short side above 32.36 mm to keep parallel lanes.
    check_cleared("wide 44x36 D10", &rect(0.0, 0.0, 44.0, 36.0), &PocketParams { tool_diameter: 10.0, ..p });
    assert!(started.elapsed().as_secs_f64() <= 90.0, "wide pocket debug fixture exceeds 90 seconds: {:?}", started.elapsed());
}

#[test]
fn corner_cleanup_reduces_34x16_rest_to_at_most_hsm_rest() {
    let p = PocketParams { bottom_z: -3.0, ..params() };
    check_cleared("face 34x16 D6", &face(), &p);
}
