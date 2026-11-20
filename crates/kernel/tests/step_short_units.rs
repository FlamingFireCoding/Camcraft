//! Provenance: our authored unit-box corpus and the ISO 10303 short-name registry.
//! Gate map (M12.1): covers camcraft_kernel::solidcore_step.

mod step_util;

use camcraft_kernel::solidcore_impl::SolidcoreKernel;
use camcraft_kernel::solidcore_step::import_step;
use camcraft_kernel::{KernelError, Measure};
use step_util::{CASE_TIMEOUT, bounded_cancel};

const INCH: &str = include_str!("../../../testdata/models/short_units_inch.step");
const METRE: &str = include_str!("../../../testdata/models/short_units_metre.step");

fn import(text: &str) -> Result<(Vec<Measure>, Vec<f64>), KernelError> {
    let bytes = text.as_bytes().to_vec();
    bounded_cancel("short-name units", CASE_TIMEOUT, move |stop| {
        let mut kernel = SolidcoreKernel::default();
        import_step(&mut kernel, &bytes, stop).map(|model| {
            (
                model.solids.iter().map(|solid| solid.measure).collect(),
                model.length_units_mm,
            )
        })
    })
}

fn assert_scale(solids: &[Measure], units: &[f64], size: f64) {
    assert_eq!(units, [size], "the declared unit must survive import");
    assert_eq!(solids.len(), 1, "the unit cube must remain one solid");
    let measure = solids[0];
    assert!(
        (measure.volume / size.powi(3) - 1.0).abs() < 1e-8,
        "wrong volume: {measure:?}, expected a {size} mm cube"
    );
    for axis in 0..3 {
        assert!(measure.min[axis].abs() < 1e-8, "wrong minimum: {measure:?}");
        assert!(
            (measure.max[axis] / size - 1.0).abs() < 1e-8,
            "wrong maximum: {measure:?}, expected {size} mm"
        );
    }
}

fn correct_scale(text: &str, size: f64) {
    let (solids, units) = import(text).expect("short-name unit entities must import");
    assert_scale(&solids, &units, size);
}

#[test]
fn short_name_inch_import_keeps_scale() {
    correct_scale(INCH, 25.4);
}

#[test]
fn short_name_metre_import_keeps_scale() {
    correct_scale(METRE, 1000.0);
}

#[test]
fn short_context_only_import_keeps_scale() {
    for (text, size) in [(INCH, 25.4), (METRE, 1000.0)] {
        correct_scale(&text.replace("SUNT(", "SI_UNIT("), size);
    }
}

#[test]
fn short_si_unit_only_import_keeps_scale() {
    for (text, size) in [(INCH, 25.4), (METRE, 1000.0)] {
        correct_scale(&text.replace("GUAC(", "GLOBAL_UNIT_ASSIGNED_CONTEXT("), size);
    }
}

#[test]
fn long_name_context_controls_import_with_correct_scale() {
    for (text, size) in [(INCH, 25.4), (METRE, 1000.0)] {
        let control = text
            .replace("GUAC(", "GLOBAL_UNIT_ASSIGNED_CONTEXT(")
            .replace("SUNT(", "SI_UNIT(");
        let (solids, units) = import(&control).expect("long-name control must import");
        assert_scale(&solids, &units, size);
    }
}
