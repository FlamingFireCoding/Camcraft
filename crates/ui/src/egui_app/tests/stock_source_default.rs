//! #3261: Stock "From model" applies the part that Source shows, with no
//! change to the picker first. Structural (AccessKit labels), no pixels.
//! Provenance: our own page law.

use super::*;
use camcraft_app::commands::ImportModel;
use camcraft_app::geometry::{Point3, cube};
use camcraft_app::stock::StockShape;

/// Imports `names` as small boxes, opens the Stock page, picks From model
/// and clicks Apply with no change to Source.
fn apply_from_model(names: &[&str]) -> Harness<'static, ShellUi> {
    let mut h = shell_harness();
    {
        let st = &mut h.state_mut().state;
        for (i, name) in names.iter().enumerate() {
            let x = i as f64 * 50.0;
            let part = cube(Point3::new(x, 0.0, 0.0), Point3::new(x + 20.0, 10.0, 5.0));
            st.history.execute(&st.world, ImportModel::from_mesh(name, part));
        }
        st.doc_version += 1;
        st.ui_target = UiTarget::Stock;
    }
    h.run_steps(2);
    h.get_by_label("From model").click();
    h.run_steps(2);
    h.get_by_label("Apply").click();
    h.run_steps(2);
    h
}

fn applied_source(h: &Harness<'static, ShellUi>) -> Option<String> {
    let world = &h.state().state.world;
    let active = camcraft_app::setup::active(world);
    let name = camcraft_app::setup::stock_entity_name(world, active.as_deref());
    let entity = model::find_by_name(world, &name)?;
    match model::stock_def_of(world, &entity) {
        Some(StockShape::FromModel { source }) => Some(source),
        _ => None,
    }
}

#[test]
fn from_model_apply_takes_the_only_part_shown() {
    let h = apply_from_model(&["box"]);
    assert_eq!(applied_source(&h).as_deref(), Some("box"), "the stock comes from the one part");
    assert_eq!(h.state().state.stock_ui.source, "box", "the draft holds the shown part");
}

#[test]
fn from_model_apply_takes_the_part_shown_of_several() {
    let h = apply_from_model(&["first", "second"]);
    let shown = h.state().state.stock_ui.source.clone();
    assert!(!shown.is_empty(), "Source shows a part");
    assert_eq!(applied_source(&h), Some(shown), "the applied source is the shown part");
}
