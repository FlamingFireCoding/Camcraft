//! The bottom bar's laws (#1198): the camera buttons, the ribbon size and
//! the combined Move and Rotate handles (#2624).
//! Provenance: our own tests over our own bar.
//! Gate map: covers ui::egui_app::viewport_bar.

use super::*;

#[test]
fn camera_buttons_flip_the_viewer_and_latch_a_tool() {
    // Projection: flips perspective <-> orthographic, the View
    // door's radios' true twin.
    let mut state = State::new();
    let p0 = state.cam.projection;
    act(&mut state, Nav::Projection);
    assert_ne!(state.cam.projection, p0, "the projection must flip");
    assert!(state.status.contains("orthographic"), "{:?}", state.status);
    act(&mut state, Nav::Projection);
    assert_eq!(state.cam.projection, p0, "one more press flips back");
    // The tool buttons latch once on and once off, and say so.
    for (nav, tool, word) in [
        (Nav::Orbit, DragMode::Orbit, "orbit"),
        (Nav::Pan, DragMode::Pan, "pan"),
        (Nav::Zoom, DragMode::Zoom, "zoom"),
    ] {
        let mut state = State::new();
        act(&mut state, nav);
        assert_eq!(state.nav_tool, Some(tool));
        assert!(state.status.contains(word), "{:?}", state.status);
        // Latching a different tool replaces the first.
        act(&mut state, if matches!(nav, Nav::Zoom) { Nav::Orbit } else { Nav::Zoom });
        assert_ne!(state.nav_tool, Some(tool));
        // The second press on the same tool latches off.
        act(&mut state, nav);
        assert_eq!(state.nav_tool, Some(tool));
        act(&mut state, nav);
        assert_eq!(state.nav_tool, None);
        assert_eq!(state.status, NAV_TOOL_CLEAR);
    }
}

#[test]
fn bar_fits_its_buttons() {
    assert_eq!(NAVS.len(), NAV_COUNT, "NAV_COUNT sizes the ribbon");
    let count = NAVS.len() as f32;
    let width = count * NAV_PX + (count - 1.0) * NAV_GAP + 2.0 * PAD;
    assert!((BAR_W - width).abs() < 1e-6, "the ribbon is as wide as its buttons");
    let pane = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(900.0, 600.0));
    let bar = bar_rect(pane);
    assert!((bar.width() - BAR_W).abs() < 1e-6);
    assert!(
        (bar.center().x - pane.center().x).abs() < 1e-3,
        "the ribbon is centred on the bottom edge"
    );
    assert!(
        (bar.max.y - (pane.max.y - BOTTOM_MARGIN)).abs() < 1e-3,
        "it rests on the bottom edge"
    );
    let kids = buttons(bar);
    assert_eq!(kids.len(), NAVS.len());
    assert!(kids.iter().all(|(_, r)| bar.contains(r.min) && bar.contains(r.max)));
    let last = kids.last().map(|(k, r)| (*k, *r));
    assert_eq!(
        last.map(|(k, _)| k),
        NAVS.last().copied(),
        "the last button is the last in the list"
    );
    assert!(
        last.is_some_and(|(_, r)| bar.contains(r.max)),
        "the last button sits inside the ribbon"
    );
}

#[test]
fn move_gizmo_shows_arrows_and_rings_together() {
    let mut state = State::new();
    act(&mut state, Nav::Rotate);
    assert!(state.rotate_handles && !state.move_handles);
    assert_eq!(state.status, rotate_label(true));
    act(&mut state, Nav::Move);
    assert!(state.move_handles && state.rotate_handles, "Move shows arrows and rings together");
    act(&mut state, Nav::Rotate);
    assert!(state.move_handles && state.rotate_handles, "Rotate keeps both handle sets while Move is on");
    assert!(state.handle_bar.turn_value, "Rotate selects the angle editor while Move is on");
    act(&mut state, Nav::Move);
    assert!(!state.move_handles && !state.rotate_handles, "Move off clears both handle sets");
    act(&mut state, Nav::Rotate);
    assert!(state.rotate_handles && !state.move_handles, "Rotate on puts Move off");
    act(&mut state, Nav::Rotate);
    assert!(!state.rotate_handles && !state.move_handles);
    assert_eq!(state.status, rotate_label(false));
    assert!(state.pending.is_empty(), "a mode is view state, never a command");
    // Rotate sits right after Move in the bar.
    let at = |k: Nav| NAVS.iter().position(|n| *n == k);
    assert_eq!(at(Nav::Rotate), at(Nav::Move).map(|i| i + 1));
}

#[test]
fn move_glyph_shows_a_part_body_while_pan_shows_only_arrows() {
    let has_body = |nav| {
        let ctx = egui::Context::default();
        let state = State::new();
        let rect = egui::Rect::from_min_size(egui::pos2(40.0, 40.0), egui::vec2(NAV_PX, NAV_PX));
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            draw(ui, rect.expand(6.0), &[(nav, rect)], &state, None);
        });
        let body = out.shapes.iter().any(|shape| {
            matches!(&shape.shape, egui::Shape::Rect(r)
                if r.stroke.width > 1.0
                    && r.rect.width() < NAV_PX * 0.65
                    && r.rect.height() < NAV_PX * 0.65)
        });
        out.textures_delta.clear();
        body
    };
    assert!(has_body(Nav::Move), "Move shows the part that its handles move");
    assert!(!has_body(Nav::Pan), "Pan remains a camera navigation glyph");
}

#[test]
fn expanded_bar_contains_controls_without_touching_buttons() {
    let pane = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(900.0, 600.0));
    let base = bar_rect(pane);
    assert_eq!(panel_rect(pane, false), base);
    let panel = panel_rect(pane, true);
    assert!(panel.contains(base.min) && panel.contains(base.max));
    assert!(panel.min.y < base.min.y, "expanded bar grows upward");
    let ctrl = controls_rect(panel);
    assert!(panel.contains(ctrl.min) && panel.contains(ctrl.max));
    for (_, btn) in buttons(base) {
        assert!(!ctrl.intersects(btn), "controls {ctrl:?} must not intersect nav button {btn:?}");
    }
    let ctx = egui::Context::default();
    let mut state = State::new();
    state.move_handles = true;
    let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
        controls::draw(ui, &mut state, ctrl);
    });
    out.textures_delta.clear();
}

#[test]
fn typed_move_and_turn_respect_units_and_refuse_bad_input() {
    let mut state = State::new();
    let mesh = camcraft_app::geometry::cube(
        camcraft_app::geometry::Point3::new(0.0, 0.0, 0.0),
        camcraft_app::geometry::Point3::new(20.0, 20.0, 20.0),
    );
    state.history.execute(&state.world, camcraft_app::commands::ImportModel::from_mesh("part", mesh));
    state.pending.push_back(crate::state::CommandKind::Select(Some("part".into())));
    state.drain_actions();
    state.pending.push_back(crate::state::CommandKind::SetUnits(camcraft_app::units::Unit::Inches));
    state.drain_actions();
    state.history.mark_clean();
    // Typing 0.5 in inch mode moves 12.7 mm exactly along Z.
    state.handle_bar.axis = 2;
    state.handle_bar.move_draft = "0.5".into();
    controls::apply_move_text(&mut state);
    state.drain_actions();
    let e = camcraft_app::model::find_by_name(&state.world, "part").expect("part");
    let pos = camcraft_app::model::transform_of(&state.world, &e).unwrap_or_default().pos;
    assert!((pos.z - 12.7).abs() < 1e-12, "0.5 in = 12.7 mm, got {}", pos.z);
    assert!(state.history.is_dirty());
    // Zero and non-finite inputs queue nothing.
    state.history.mark_clean();
    state.handle_bar.move_draft = "0".into();
    controls::apply_move_text(&mut state);
    state.handle_bar.rotate_draft = "NaN".into();
    controls::apply_turn_text(&mut state);
    state.drain_actions();
    assert!(!state.history.is_dirty());
    assert!(state.status.contains("part not turned"), "{}", state.status);
}

#[test]
fn nav_buttons_have_distinct_labels_and_descriptive_tooltips() {
    let mut state = State::new();
    assert_eq!(NAVS.len(), NAV_COUNT);
    for &kind in &NAVS {
        let lab = label(kind);
        assert!(!lab.is_empty(), "label for {:?} must not be empty", kind);
        let tip = tooltip(kind, &state);
        assert!(!tip.is_empty(), "tooltip for {:?} must not be empty", kind);
    }

    // Toggle states reflect in tooltips:
    // Wireframe toggle
    state.display = DisplayMode::ShadedEdges;
    assert!(tooltip(Nav::Wire, &state).contains("shaded with edges"));
    state.display = DisplayMode::Wireframe;
    assert!(tooltip(Nav::Wire, &state).contains("wireframe"));

    // Toolpaths toggle
    state.show_all_paths = false;
    assert!(tooltip(Nav::Paths, &state).contains("off"));
    state.show_all_paths = true;
    assert!(tooltip(Nav::Paths, &state).contains("on"));

    // World axes toggle
    state.show_world_axes = false;
    assert!(tooltip(Nav::Axes, &state).contains("off"));
    state.show_world_axes = true;
    assert!(tooltip(Nav::Axes, &state).contains("on"));

    // Pick through toggle
    state.pick_through = false;
    assert!(tooltip(Nav::PickThrough, &state).contains("off"));
    state.pick_through = true;
    assert!(tooltip(Nav::PickThrough, &state).contains("on"));

    // Move handles toggle
    state.move_handles = false;
    assert!(tooltip(Nav::Move, &state).contains("off"));
    state.move_handles = true;
    assert!(tooltip(Nav::Move, &state).contains("on"));

    // Rotate handles toggle
    state.rotate_handles = false;
    assert!(tooltip(Nav::Rotate, &state).contains("off"));
    state.rotate_handles = true;
    assert!(tooltip(Nav::Rotate, &state).contains("on"));

    // Projection toggle
    state.cam.projection = ProjectionMode::Perspective;
    assert!(tooltip(Nav::Projection, &state).contains("perspective"));
    state.cam.projection = ProjectionMode::Orthographic;
    assert!(tooltip(Nav::Projection, &state).contains("orthographic"));

    // Nav tools
    state.nav_tool = None;
    assert!(tooltip(Nav::Orbit, &state).contains("click to latch"));
    assert!(tooltip(Nav::Pan, &state).contains("click to latch"));
    assert!(tooltip(Nav::Zoom, &state).contains("click to latch"));
    state.nav_tool = Some(DragMode::Orbit);
    assert!(tooltip(Nav::Orbit, &state).contains("active"));
    state.nav_tool = Some(DragMode::Pan);
    assert!(tooltip(Nav::Pan, &state).contains("active"));
    state.nav_tool = Some(DragMode::Zoom);
    assert!(tooltip(Nav::Zoom, &state).contains("active"));
}

#[test]
fn nav_buttons_attach_tooltips_on_hover() {
    use egui_kittest::kittest::Queryable;
    use egui_kittest::Harness;

    let state = State::new();
    let pane = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(900.0, 600.0));
    let bar = bar_rect(pane);
    let btns = buttons(bar);
    let mut h = Harness::builder()
        .with_size(egui::vec2(900.0, 600.0))
        .build_ui(move |ui| {
            draw(ui, bar, &btns, &state, None);
        });

    h.ctx.all_styles_mut(|s| {
        s.interaction.tooltip_delay = 0.0;
        s.interaction.show_tooltips_only_when_still = false;
    });

    h.run_steps(2);

    fn walk(shape: &egui::Shape, out: &mut Vec<String>) {
        match shape {
            egui::Shape::Text(t) => out.push(t.galley.text().to_owned()),
            egui::Shape::Vec(v) => v.iter().for_each(|s| walk(s, out)),
            _ => {}
        }
    }

    let all_buttons = buttons(bar);

    // Verify all 12 buttons have accessible labels
    for &(kind, btn_rect) in &all_buttons {
        let lab = label(kind);
        let node = h.get_by_label(lab);
        assert_eq!(node.rect(), btn_rect, "node for {:?} must match button rect", kind);
    }

    // Verify hover tooltip is displayed when hovering over a button
    h.hover_at(all_buttons[0].1.center());
    h.run_steps(2);

    let mut painted = Vec::new();
    for shape in &h.output().shapes {
        walk(&shape.shape, &mut painted);
    }

    assert!(
        painted.iter().any(|s| s.contains("Frame all")),
        "hovering over Frame button must paint a tooltip containing 'Frame all', got: {:?}",
        painted
    );
}

#[test]
#[ignore = "evidence generator: opt-in with CAMCRAFT_EVIDENCE_SCREENSHOTS_OUT"]
fn generate_evidence_screenshots() {
    let Some(out_dir) = std::env::var_os("CAMCRAFT_EVIDENCE_SCREENSHOTS_OUT")
        .map(std::path::PathBuf::from)
    else {
        return;
    };
    let _ = std::fs::create_dir_all(&out_dir);
    let pane = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(600.0, 120.0));
    let bar = bar_rect(pane);
    let all_buttons = buttons(bar);

    let render_shot = |name: &str, mut setup: Box<dyn FnMut(&mut State)>, hover: Option<usize>| {
        let mut state = State::new();
        setup(&mut state);
        let btns = all_buttons.clone();
        let hover_pos = hover.map(|idx| btns[idx].1.center());
        let mut h = egui_kittest::Harness::builder()
            .with_size(egui::vec2(600.0, 120.0))
            .build_ui(move |ui| {
                ui.painter().rect_filled(pane, 0.0, egui::Color32::from_rgb(0x17, 0x1A, 0x1F));
                draw(ui, bar, &btns, &state, hover_pos);
            });

        h.ctx.all_styles_mut(|s| {
            s.interaction.tooltip_delay = 0.0;
            s.interaction.show_tooltips_only_when_still = false;
        });

        h.run_steps(2);
        if let Some(pos) = hover_pos {
            h.hover_at(pos);
            h.run_steps(3);
        }

        h.render()
            .expect("render")
            .save(out_dir.join(name))
            .expect("save image");
    };

    // 00: At rest (baseline)
    render_shot("00_at_rest.png", Box::new(|_| {}), None);

    // 01: Frame all
    render_shot("01_frame_all.png", Box::new(|_| {}), Some(0));

    // 02: Isometric view
    render_shot("02_iso_view.png", Box::new(|_| {}), Some(1));

    // 03 & 04: Display mode (Shaded vs Wireframe)
    render_shot(
        "03_wire_shaded.png",
        Box::new(|st| st.display = DisplayMode::ShadedEdges),
        Some(2),
    );
    render_shot(
        "04_wire_wireframe.png",
        Box::new(|st| st.display = DisplayMode::Wireframe),
        Some(2),
    );

    // 05 & 06: Show all toolpaths (Off vs On)
    render_shot(
        "05_paths_off.png",
        Box::new(|st| st.show_all_paths = false),
        Some(3),
    );
    render_shot(
        "06_paths_on.png",
        Box::new(|st| st.show_all_paths = true),
        Some(3),
    );

    // 07 & 08: World axes (Off vs On)
    render_shot(
        "07_axes_off.png",
        Box::new(|st| st.show_world_axes = false),
        Some(4),
    );
    render_shot(
        "08_axes_on.png",
        Box::new(|st| st.show_world_axes = true),
        Some(4),
    );

    // 09 & 10: Pick through (Off vs On)
    render_shot(
        "09_pick_through_off.png",
        Box::new(|st| st.pick_through = false),
        Some(5),
    );
    render_shot(
        "10_pick_through_on.png",
        Box::new(|st| st.pick_through = true),
        Some(5),
    );

    // 11 & 12: Move mode (Off vs On)
    render_shot(
        "11_move_mode_off.png",
        Box::new(|st| st.move_handles = false),
        Some(6),
    );
    render_shot(
        "12_move_mode_on.png",
        Box::new(|st| st.move_handles = true),
        Some(6),
    );

    // 13 & 14: Rotate mode (Off vs On)
    render_shot(
        "13_rotate_mode_off.png",
        Box::new(|st| st.rotate_handles = false),
        Some(7),
    );
    render_shot(
        "14_rotate_mode_on.png",
        Box::new(|st| st.rotate_handles = true),
        Some(7),
    );

    // 15 & 16: Projection (Perspective vs Orthographic)
    render_shot(
        "15_projection_perspective.png",
        Box::new(|st| st.cam.projection = ProjectionMode::Perspective),
        Some(8),
    );
    render_shot(
        "16_projection_ortho.png",
        Box::new(|st| st.cam.projection = ProjectionMode::Orthographic),
        Some(8),
    );

    // 17 & 18: Orbit tool (Unlatched vs Latched Active)
    render_shot(
        "17_orbit_latched_off.png",
        Box::new(|st| st.nav_tool = None),
        Some(9),
    );
    render_shot(
        "18_orbit_latched_on.png",
        Box::new(|st| st.nav_tool = Some(DragMode::Orbit)),
        Some(9),
    );

    // 19 & 20: Pan tool (Unlatched vs Latched Active)
    render_shot(
        "19_pan_latched_off.png",
        Box::new(|st| st.nav_tool = None),
        Some(10),
    );
    render_shot(
        "20_pan_latched_on.png",
        Box::new(|st| st.nav_tool = Some(DragMode::Pan)),
        Some(10),
    );

    // 21 & 22: Zoom tool (Unlatched vs Latched Active)
    render_shot(
        "21_zoom_latched_off.png",
        Box::new(|st| st.nav_tool = None),
        Some(11),
    );
    render_shot(
        "22_zoom_latched_on.png",
        Box::new(|st| st.nav_tool = Some(DragMode::Zoom)),
        Some(11),
    );
}
