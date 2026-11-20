//! Provenance: our authored unit-box corpus and the ISO 10303 short-name registry.
//! Gate map (M12.1): covers camcraft_kernel::solidcore_step.

mod step_util;

use camcraft_kernel::solidcore_impl::SolidcoreKernel;
use camcraft_kernel::solidcore_step::{StepModel, import_step, import_step_model};
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

/// The import must be refused, and the refusal must contain `named`.
fn refused(text: &str, named: &str) {
    let err = import(text).expect_err("a length unit read at 1 mm per unit must be refused");
    let msg = err.to_string();
    assert!(msg.contains(named), "the refusal must name `{named}`: {msg}");
}

/// #2919: a length unit, but no context that the reader can find states it.
#[test]
fn length_unit_without_unit_context_is_refused() {
    for text in [INCH, METRE] {
        let no_context = text.replace("GUAC((#12, #13, #14))", "");
        assert!(!no_context.contains("GUAC("), "the fixture must lose its unit context");
        refused(&no_context, "length unit #12 is in the file");
    }
}

/// #2919 review: a unit context that names a unit missing from the file.
#[test]
fn unit_context_naming_missing_unit_is_refused() {
    refused(
        &INCH.replace("GUAC((#12, #13, #14))", "GUAC((#12, #13, #14, #777))"),
        "unit context #11 names unit #777",
    );
}

/// #2919 review: the solid's own context states no length unit, while a
/// different context of the file states one.
#[test]
fn solid_in_unitless_context_is_refused() {
    let text = INCH
        .replace(
            "#10 = ADVANCED_BREP_SHAPE_REPRESENTATION('', (#16), #11);",
            "#10 = ADVANCED_BREP_SHAPE_REPRESENTATION('', (#16), #902);\n\
             #902 = ( GEOMETRIC_REPRESENTATION_CONTEXT(3) REPRESENTATION_CONTEXT('', '') );",
        );
    assert!(text.contains("#902 ="), "the fixture must move the solid to a context with no unit");
    refused(&text, "context #902 states no length unit");
}

/// #2919 review (P1, case 1): the file states a length unit, but no
/// representation lists the solid, so the reader reads it at 1 mm per unit.
#[test]
fn solid_listed_by_no_representation_is_refused() {
    let text = INCH.replace(
        "#10 = ADVANCED_BREP_SHAPE_REPRESENTATION('', (#16), #11);",
        "#10 = ADVANCED_BREP_SHAPE_REPRESENTATION('', (), #11);",
    );
    assert!(text.contains("('', (), #11)"), "the fixture must unlist the solid");
    refused(&text, "no representation gives it a context");
}

/// The unit cube as a surface model that the topology reader splits into
/// three pieces: closed shell #17 (a solid) and open shells #903 and #904
/// (two sheets), with representation #10 in context `ctx`.
fn split_model(ctx: &str) -> String {
    let mut text = INCH.replace(
        "#16 = MANIFOLD_SOLID_BREP('', #17);",
        "#16 = SHELL_BASED_SURFACE_MODEL('', (#17, #903, #904));\n\
         #903 = OPEN_SHELL('', (#18));\n\
         #904 = OPEN_SHELL('', (#25));",
    );
    if ctx != "#11" {
        text = text.replace(
            "#10 = ADVANCED_BREP_SHAPE_REPRESENTATION('', (#16), #11);",
            &format!(
                "#10 = ADVANCED_BREP_SHAPE_REPRESENTATION('', (#16), {ctx});\n\
                 {ctx} = ( GEOMETRIC_REPRESENTATION_CONTEXT(3) REPRESENTATION_CONTEXT('', '') );"
            ),
        );
    }
    assert!(text.contains("#904 = OPEN_SHELL"), "the fixture must hold a split surface model");
    text
}

fn import_model(text: &str) -> Result<StepModel, KernelError> {
    let bytes = text.as_bytes().to_vec();
    bounded_cancel("short-name units, model", CASE_TIMEOUT, move |stop| {
        let mut kernel = SolidcoreKernel::default();
        import_step_model(&mut kernel, &bytes, stop)
    })
}

/// #2919 review (P1, case 2): each piece split from a surface model sits in
/// the model's context. A unitless context refuses every piece by name.
#[test]
fn split_surface_model_pieces_in_unitless_context_are_refused() {
    let text = split_model("#902");
    assert!(text.contains("#902 = ("), "the fixture must move the model to a context with no unit");
    let err = import_model(&text).expect_err("pieces read at 1 mm per unit must be refused");
    let msg = err.to_string();
    assert!(msg.contains("3 of 3"), "each of the three pieces must be refused: {msg}");
    assert!(msg.contains("first #17"), "the solid piece must be refused: {msg}");
    assert!(msg.contains("context #902 states no length unit"), "the refusal must name the context: {msg}");
}

/// Control for the case above: the same split model in the inch context
/// keeps each piece, at the inch scale.
#[test]
fn split_surface_model_pieces_in_inch_context_keep_scale() {
    let model = import_model(&split_model("#11")).expect("the split model in the inch context must import");
    assert_eq!(model.length_units_mm, [25.4], "the declared unit must survive import");
    assert_eq!(model.solids.len(), 1, "closed shell #17 must stay one solid");
    assert_eq!(model.sheets.len(), 2, "open shells #903 and #904 must stay two sheets");
    let measure = model.solids[0].measure;
    assert!((measure.volume / 25.4f64.powi(3) - 1.0).abs() < 1e-8, "wrong volume: {measure:?}");
    for axis in 0..3 {
        assert!((measure.max[axis] / 25.4 - 1.0).abs() < 1e-8, "wrong maximum: {measure:?}");
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
