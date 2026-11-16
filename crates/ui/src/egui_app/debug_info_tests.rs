//! Issue #157: the Debug info report. The panel is a dock tab since #2262;
//! its door and dock tests are in `tests/layout.rs`.
//! Provenance: our own tests over our own window.

use super::debug_info;
use crate::state::State;

#[test]
fn the_report_names_build_platform_gpu_and_document() {
    let state = State::new();
    let text = debug_info::report(&state);
    assert!(text.starts_with(&format!("CamCraft {}", env!("CARGO_PKG_VERSION"))), "{text}");
    for want in [
        format!("Build time: {}", crate::build_info::BUILT_UTC),
        format!("Source commit: {}", crate::build_info::SOURCE_SHA),
        format!("Platform: {} {}", std::env::consts::OS, std::env::consts::ARCH),
        "GPU: none (headless)".to_owned(),
        "Project: unsaved".to_owned(),
        "Units: ".to_owned(),
        "Machine: ".to_owned(),
        "Document: ".to_owned(),
        "Status: ".to_owned(),
    ] {
        assert!(text.contains(&want), "missing {want:?} in:\n{text}");
    }
}

#[test]
fn the_report_gives_the_frame_rate_or_says_idle() {
    // #2262: a fresh state has drawn no frame, so the panel says idle.
    let state = State::new();
    let text = debug_info::report(&state);
    assert!(text.contains("Frame rate: idle"), "{text}");
}
