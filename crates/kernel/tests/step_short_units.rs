//! Provenance: our authored unit-box corpus and the ISO 10303 short-name registry.
//! Gate map (M12.1): covers camcraft_kernel::solidcore_step.

mod step_util;

use camcraft_kernel::solidcore_impl::SolidcoreKernel;
use camcraft_kernel::solidcore_step::import_step;
use camcraft_kernel::{KernelError, Measure};
use step_util::{CASE_TIMEOUT, bounded_cancel};

const INCH: &str = include_str!("../../../testdata/models/short_units_inch.step");
const METRE: &str = include_str!("../../../testdata/models/short_units_metre.step");
const CONE: &str = include_str!("../../../testdata/models/short_units_degree_cone.step");

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

/// The frustum of `short_units_degree_cone.step` (#2973): base radius 10 mm,
/// height 10 mm, semi-angle 1 degree. Its volume and top radius depend on the
/// angle, so an angle read in radians cannot pass.
fn assert_degree_cone(text: &str) {
    let (solids, units) = import(text).expect("the degree cone must import");
    assert_eq!(units, [1.0], "the file states millimetres");
    assert_eq!(solids.len(), 1, "the frustum must import as one solid");
    let m = solids[0];
    let (r0, h) = (10.0_f64, 10.0_f64);
    let r1 = r0 + h * 1.0_f64.to_radians().tan();
    let volume = std::f64::consts::PI * h / 3.0 * (r0 * r0 + r0 * r1 + r1 * r1);
    assert!(
        (m.volume / volume - 1.0).abs() < 1e-6,
        "wrong volume: {m:?}, expected {volume} mm^3 for a 1 degree cone"
    );
    assert!((m.max[0] - r1).abs() < 1e-6, "wrong top radius: {m:?}, expected {r1}");
    assert!((m.max[2] - h).abs() < 1e-9 && m.min[2].abs() < 1e-9, "wrong height: {m:?}");
}

#[test]
fn short_name_degree_cone_keeps_its_angle() {
    assert_degree_cone(CONE);
}

#[test]
fn long_name_degree_cone_control_keeps_its_angle() {
    let control = CONE
        .replace("PLANUN(", "PLANE_ANGLE_UNIT(")
        .replace("GUAC(", "GLOBAL_UNIT_ASSIGNED_CONTEXT(")
        .replace("GC(", "GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT(")
        .replace("SUNT(", "SI_UNIT(");
    assert_degree_cone(&control);
}

#[test]
fn unmarked_angle_unit_is_refused_by_name() {
    // The degree unit #13 without its plane angle marker: the reader cannot
    // know that it is an angle unit, and must not read angles in radians.
    let text = CONE.replace("NMDUNT(*) PLANUN() );", "NMDUNT(*) );");
    assert_ne!(text, CONE, "the fixture must have the marker to remove");
    match import(&text) {
        Err(KernelError::InvalidInput { detail, .. }) => assert!(
            detail.contains("#13") && detail.contains("plane angle"),
            "the refusal must name the unit: {detail}"
        ),
        other => panic!("an unmarked angle unit must be refused, got {other:?}"),
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
