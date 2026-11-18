//! Structural twin of the two Refresh buttons of #2380 (the setup's
//! Machine page and the post picker): each is a named, enabled button,
//! and a press reads the shipped folders again. No position, size or
//! colour is asserted.
//!
//! Provenance: our own structural assertion of the folder lists (#2380).

use super::*;
use crate::egui_app::machine_page::REFRESH_MACHINES;
use crate::egui_app::pages::REFRESH_POSTS;
use crate::state::SetupPage;
use egui::accesskit::Role;

/// The shell at the app's default height, with one setup and `page` shown.
fn setup_page(page: SetupPage) -> Harness<'static, ShellUi> {
    let mut h = Harness::builder()
        .with_size(Vec2::new(1280.0, 800.0))
        .build_ui_state(frame, ShellUi::seeded());
    h.state_mut().state.pending.push_back(CommandKind::CreateSetup { name: None, datum_from: None });
    h.run_steps(2);
    crate::actions::target(&mut h.state_mut().state, UiTarget::SetupNode("Setup1".into(), page));
    h.run_steps(3);
    h
}

/// The button named `name` is an enabled button, and a press reads the
/// folders again (the process read count goes up).
fn assert_refresh(h: &mut Harness<'static, ShellUi>, name: &str) {
    let button = h.get_by_role_and_label(Role::Button, name);
    assert!(!button.accesskit_node().is_disabled(), "{name:?} is enabled");
    // Read once, so the count below cannot rise from a first use alone.
    let _ = camcraft_app::posting::machine_scan();
    let before = camcraft_app::posting::scan_reads();
    button.click();
    h.run();
    assert!(camcraft_app::posting::scan_reads() > before, "a press of {name:?} reads the folders again");
}

#[test]
fn the_machine_page_refresh_button_rereads_the_folder() {
    let _guard = snapshot_guard();
    let mut h = setup_page(SetupPage::Machine);
    assert_refresh(&mut h, REFRESH_MACHINES);
}

#[test]
fn the_post_picker_refresh_button_rereads_the_folder() {
    let _guard = snapshot_guard();
    let mut h = setup_page(SetupPage::Post);
    assert_refresh(&mut h, REFRESH_POSTS);
}
