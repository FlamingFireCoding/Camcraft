//! #2958: the Theme door's "Line width" and "Anti-aliasing" groups. A
//! pick is a per-user view setting: it goes out to the view file at once,
//! survives a restart, and Ctrl+Z does not take it back. A choice of the
//! gizmo names never resets it. An anti-aliasing count that the graphics
//! device does not offer is a disabled row, and a queued pick of it is
//! refused by name and not kept.
//! Provenance: our own tests over our own shell; no HSMWorks material.
//!
//! Gate map (M4.4): covers ui::egui_app::theme_drop (the quality rows),
//! ui::state (state_view), app::view through the State handoff.

use std::path::PathBuf;

use camcraft_app::view::{self, Antialias, FILE_NAME, LineWidth};
use egui::accesskit::Role;
use egui_kittest::kittest::NodeT;

use super::*;

fn temp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("camcraft-quality-ui-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("make the test folder");
    std::fs::canonicalize(&d).expect("canonical folder")
}

fn open_theme_door(h: &mut egui_kittest::Harness<'_, ShellUi>) {
    h.get_by_role_and_label(Role::Button, "Theme").click();
    h.run_steps(2);
    assert!(h.state().state.ribbon_theme_open, "pressing the door opens its list");
}

#[test]
fn the_defaults_are_one_and_a_half_points_and_four_samples() {
    let state = State::new();
    assert_eq!((state.line_width, state.antialias), (LineWidth::Normal, Antialias::X4));
    assert_eq!(state.samples_offered, vec![1, 4], "headless keeps the WebGPU guaranteed counts");
}

#[test]
fn a_line_width_pick_is_kept_survives_a_restart_and_undo_leaves_it() {
    let file = temp_dir("width").join(FILE_NAME);
    let mut h = shell_harness();
    h.state_mut().state.view_store = Some(file.clone());
    h.run();
    open_theme_door(&mut h);
    for caption in ["Line width", "Anti-aliasing"] {
        assert!(h.query_by_label(caption).is_some(), "the door shows the '{caption}' group");
    }
    // The seed document is already unsaved; the pick must not change that.
    let dirty = h.state().state.history.is_dirty();
    h.get_by_role_and_label(Role::MenuItem, "2 pt").click();
    h.run_steps(2);
    assert!(!h.state().state.ribbon_theme_open, "a choice closes the list");
    assert_eq!(h.state().state.line_width, LineWidth::Bold);
    assert_eq!(view::load(&file).0.line_width, LineWidth::Bold, "the pick goes out to the file at once");
    assert_eq!(h.state().state.history.is_dirty(), dirty, "a view setting is not an unsaved change");
    h.state_mut().state.pending.push_back(CommandKind::Undo);
    h.run_steps(3);
    assert_eq!(h.state().state.line_width, LineWidth::Bold, "undo leaves the width");
    // The gizmo-name switch writes the same file: it keeps the width.
    h.state_mut().state.set_gizmo_names(false);
    assert_eq!(view::load(&file).0.line_width, LineWidth::Bold, "another view setting never resets it");
    let mut restarted = State::new();
    restarted.load_view_from(&file);
    assert_eq!(restarted.line_width, LineWidth::Bold, "the width survives a restart");
}

#[test]
fn an_offered_antialias_pick_is_kept_and_an_unoffered_one_is_a_disabled_row() {
    let file = temp_dir("aa").join(FILE_NAME);
    let mut h = shell_harness();
    h.state_mut().state.view_store = Some(file.clone());
    h.run();
    open_theme_door(&mut h);
    let two = h.get_by_role_and_label(Role::MenuItem, "2x");
    assert!(two.accesskit_node().is_disabled(), "2x is not offered on WebGPU's guaranteed counts");
    assert!(!h.get_by_role_and_label(Role::MenuItem, "Off").accesskit_node().is_disabled());
    h.get_by_role_and_label(Role::MenuItem, "Off").click();
    h.run_steps(2);
    assert_eq!(h.state().state.antialias, Antialias::Off);
    assert_eq!(view::load(&file).0.antialias, Antialias::Off, "kept at once");
    // A device that offers 2x enables the row.
    h.state_mut().state.samples_offered = vec![1, 2, 4];
    open_theme_door(&mut h);
    assert!(!h.get_by_role_and_label(Role::MenuItem, "2x").accesskit_node().is_disabled(), "an offered 2x is live");
}

#[test]
fn a_queued_pick_of_an_unoffered_count_is_refused_by_name_and_not_kept() {
    let file = temp_dir("refuse").join(FILE_NAME);
    let mut state = State::new();
    state.view_store = Some(file.clone());
    state.pending.push_back(CommandKind::SelectAntialias(Antialias::X2));
    state.drain_actions();
    assert_eq!(state.antialias, Antialias::X4, "the refused pick is not worn");
    assert!(state.status.contains("anti-aliasing 2x not selected"), "refused by name: {}", state.status);
    assert!(!file.exists(), "nothing is written for a refusal");
    state.samples_offered = vec![1, 2, 4];
    state.pending.push_back(CommandKind::SelectAntialias(Antialias::X2));
    state.drain_actions();
    assert_eq!(state.antialias, Antialias::X2, "an offered count is kept");
    assert_eq!(view::load(&file).0.antialias, Antialias::X2);
}
