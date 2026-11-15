//! Issue #1696: the status strip's Report door opens a window that fills
//! GitHub's new-issue form in (title, body, label) and copies a picture
//! of the document to the clipboard.
//! Provenance: our own tests over our own window.

use egui::accesskit::Role;
use egui::Vec2;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

use super::{debug_info, panes, report, theme};
use crate::state::State;

fn frame(ui: &mut egui::Ui, st: &mut State) {
    let pal = theme::current();
    panes::status(ui, st, &pal, None);
    debug_info::window(ui.ctx(), st);
    report::window(ui.ctx(), st);
}

fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size(Vec2::new(900.0, 500.0))
        .build_ui_state(frame, State::new())
}

#[test]
fn url_encode_keeps_safe_words_and_plains_the_rest() {
    assert_eq!(report::url_encode("Contour-1_2.3~ok"), "Contour-1_2.3~ok");
    assert_eq!(report::url_encode("a b"), "a+b");
    // A reserved byte and a non-ASCII byte both go through as UTF-8 %XX.
    assert_eq!(report::url_encode("a?b#c"), "a%3Fb%23c");
    assert_eq!(report::url_encode("é"), "%C3%A9");
}

#[test]
fn the_url_prefills_the_blank_template_with_title_body_and_label() {
    let url = report::issue_url(report::ReportKind::Bug, "Crash on post", "line one\nline two");
    assert!(url.starts_with("https://github.com/NeoTech/Camcraft/issues/new?template=blank-issue.md"), "{url}");
    assert!(url.contains("&title=Crash+on+post"), "{url}");
    assert!(url.contains("&body=line+one%0Aline+two"), "{url}");
    assert!(url.ends_with("&labels=bug"), "{url}");
    let feat = report::issue_url(report::ReportKind::Feature, "", "");
    assert!(feat.ends_with("&labels=enhancement"), "{feat}");
}

#[test]
fn the_body_carries_the_questions_and_the_debug_report() {
    let state = State::new();
    let body = report::report_body(&state);
    assert!(body.contains("**What happened**"), "{body}");
    assert!(body.contains("**Steps to reproduce**"), "{body}");
    // Same fields, one source: the Debug info window's own report.
    assert!(body.contains(&debug_info::report(&state)), "the body must embed the debug report");
    assert!(body.contains("Copy screenshot"), "the body tells the reporter to paste the picture");
}

#[test]
fn the_door_opens_the_window_and_a_kind_button_files_the_url() {
    let mut h = harness();
    h.run();
    assert!(h.query_by_label("Report a bug").is_none(), "closed on a fresh state");
    h.get_by_label("Report an issue").click();
    h.run();
    assert!(h.state().report_open);
    h.get_by_label("Request a feature").click();
    h.step(); // the click lands this frame; its output is this frame's
    let opened = h.output().platform_output.commands.iter().any(|c| {
        matches!(c, egui::OutputCommand::OpenUrl(u)
            if u.url.starts_with("https://github.com/NeoTech/Camcraft/issues/new")
            && u.url.ends_with("&labels=enhancement"))
    });
    assert!(opened, "the kind button opened the prefilled new-issue URL");
    h.run();
    assert!(h.state().status.starts_with("Request a feature:"), "status: {}", h.state().status);
    // The door closes it again. The window's title row reads "Report an
    // issue" too, so only the button answers (the Debug info window's law).
    h.get_by(|n| n.role() == Role::Button && n.label().as_deref() == Some("Report an issue")).click();
    h.run();
    assert!(!h.state().report_open);
    assert!(h.query_by_label("Report a bug").is_none(), "the door closed it");
}

#[test]
fn the_title_field_travels_into_the_url() {
    let mut h = harness();
    h.get_by_label("Report an issue").click();
    h.run();
    let s = h.state_mut();
    s.report_title = "Pocket leaves a witness line".to_owned();
    h.run();
    h.get_by_label("Report a bug").click();
    h.step();
    let opened = h.output().platform_output.commands.iter().any(|c| {
        matches!(c, egui::OutputCommand::OpenUrl(u) if u.url.contains("&title=Pocket+leaves+a+witness+line"))
    });
    assert!(opened, "the typed title rides into the URL");
}

#[test]
fn copy_screenshot_says_the_word_on_an_empty_document() {
    // The headless State has no model: the raster finds nothing to frame,
    // so the copy must refuse in words, not fail silent (Prime Directive).
    let mut h = harness();
    h.get_by_label("Report an issue").click();
    h.run();
    h.get_by_label("Copy screenshot").click();
    h.run();
    assert!(h.state().status.starts_with("no picture to copy"), "status: {}", h.state().status);
    let copied = h
        .output()
        .platform_output
        .commands
        .iter()
        .any(|c| matches!(c, egui::OutputCommand::CopyImage(_)));
    assert!(!copied, "nothing was copied from an empty document");
}

#[test]
fn the_screenshot_paints_the_document() {
    use camcraft_app::commands::ImportModel;
    use camcraft_app::geometry::{cube, Point3};
    let mut state = State::new();
    state
        .history
        .execute(&state.world, ImportModel::from_mesh("cube", cube(Point3::ZERO, Point3::new(10.0, 10.0, 10.0))));
    let img = report::screenshot_image(&state).expect("a cube frames a camera");
    assert_eq!(img.size, [800, 600], "the agreed picture size");
    let bg = egui::Color32::from_rgb(0xF2, 0xF2, 0xF2);
    assert!(
        img.pixels.iter().any(|p| *p != bg),
        "the cube paints over the background"
    );
}
