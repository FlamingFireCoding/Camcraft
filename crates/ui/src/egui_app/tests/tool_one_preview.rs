//! #2017: the tool page has one preview - the side view - and it draws the
//! holder shape the collision check reads; the 3D view is a placement aid
//! that shows only while a Pick button is armed. Provenance: our own page
//! law; the test holder is our own geometry.

use super::tool_body::{TOOL, editor, prism_stl, shank_top, temp_stl};
use egui::{Color32, Shape};

use camcraft_app::tool_body;
use super::*;

/// Background rects of the previews: the side view is 260 x 260, the 3D view 260 x 200.
fn previews(harness: &Harness<'static, ShellUi>) -> (usize, usize) {
    let (mut side, mut view3d) = (0, 0);
    fn walk(s: &Shape, side: &mut usize, view3d: &mut usize) {
        match s {
            Shape::Vec(v) => v.iter().for_each(|s| walk(s, side, view3d)),
            Shape::Rect(r) if r.fill == Color32::from_rgb(24, 26, 31) => {
                if (r.rect.width() - 260.0).abs() < 0.5 && (r.rect.height() - 260.0).abs() < 0.5 {
                    *side += 1;
                } else if (r.rect.width() - 260.0).abs() < 0.5 && (r.rect.height() - 200.0).abs() < 0.5 {
                    *view3d += 1;
                }
            }
            _ => {}
        }
    }
    for c in &harness.output().shapes {
        walk(&c.shape, &mut side, &mut view3d);
    }
    (side, view3d)
}

#[test]
fn the_tool_page_shows_one_preview_and_no_idle_3d_view() {
    let mut harness = editor();
    let path = temp_stl("one");
    prism_stl(&path, 15.0, 0.0, 30.0, 24);
    harness.state_mut().state.import_tool_body_from(&path);
    harness.run();
    harness.run();
    assert_eq!(previews(&harness), (1, 0), "one side view, no 3D view at rest");
    harness.get_by_label("Pick tip").click();
    harness.run();
    harness.run();
    assert_eq!(previews(&harness), (1, 1), "the armed pick shows the 3D placement view");
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn the_side_preview_bands_are_the_collision_stack_of_an_explicit_body() {
    let mut harness = editor();
    let path = temp_stl("bands");
    prism_stl(&path, 15.0, 0.0, 30.0, 24);
    let top = shank_top(&harness.state().state);
    {
        let st = &mut harness.state_mut().state;
        st.import_tool_body_from(&path);
        st.tool_body_ui.offset = top + 5.0;
    }
    harness.run();
    harness.get_by_label("Apply body").click();
    harness.run();
    harness.run();
    let st = &mut harness.state_mut().state;
    let e = model::find_by_name(&st.world, TOOL).expect("tool");
    let tool = model::tool_of(&st.world, &e).expect("tool record");
    let (_, profile) = tool_body::judged(&st.world, &e, tool.clone()).expect("body fits");
    let swept: Vec<(f64, f64, f64)> = tool_body::stack(&tool.geom(), profile.as_ref())
        .bands
        .iter()
        .map(|b| (b.above, b.len, b.r))
        .collect();
    assert!(swept.len() > 1, "the body gives bands: {swept:?}");
    assert_eq!(super::super::tool_body_page::envelope(st), Some(swept), "one shape for check and preview");
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn without_a_body_the_side_preview_has_no_envelope_bands() {
    let mut harness = editor();
    assert_eq!(super::super::tool_body_page::envelope(&mut harness.state_mut().state), None);
}
