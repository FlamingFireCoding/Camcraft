//! Gate map: covers camcraft_motion::break_through.
//! The value-only helper is usable without an app world or document.

use camcraft_motion::break_through::{BELOW_STOCK_NOTE, Broke, judge};
use camcraft_motion::cl::Toolpath;
use camcraft_motion::geometry::Point3;
use camcraft_motion::units::Unit;

#[test]
fn caller_tolerance_controls_the_stock_bottom_warning() {
    let b = Some(Broke { run: 1.0, tip: Point3::new(0.0, 0.0, -0.005) });
    let below = |tolerance| {
        judge(Toolpath::new(Vec::new()), b, Some(0.0), false, Unit::Millimeters, tolerance)
            .notes.iter().any(|note| note.starts_with(BELOW_STOCK_NOTE))
    };
    assert!(!below(0.01), "a tip inside the caller's tolerance needs no warning");
    assert!(!below(0.005), "the exact tolerance boundary needs no warning");
    assert!(below(0.001), "a tip beyond the caller's tolerance needs a warning");
}
